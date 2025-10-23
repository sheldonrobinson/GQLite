use std::{env, path::Path};

use gqlparser::gqls::prelude::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use stringcase::snake_case;

use crate::my_crate;

pub(super) struct ParsedInput
{
  /// Identifier for the generated module.
  ident: syn::Ident,
  /// File name with the GQL Schema.
  filename: syn::LitStr,
}

/// Implement the Parse trait from syn to parse the input
impl syn::parse::Parse for ParsedInput
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
  {
    let ident = input.parse()?;
    let _: syn::Token![,] = input.parse()?;
    let filename = input.parse()?;
    Ok(Self { ident, filename })
  }
}

fn property_type(prop: &Property) -> Result<TokenStream, syn::Error>
{
  Ok(match prop
  {
    Property::Any => quote! { graphcore::Value },
    Property::Array(_) =>
    {
      let ct = property_type(prop)?;

      quote! { Vec<#ct> }
    }
    Property::Literal(lit) =>
    {
      let r: Result<LiteralBaseType, _> = (*lit).try_into();
      if let Ok(r) = r
      {
        match r
        {
          LiteralBaseType::Boolean => quote! {bool},
          LiteralBaseType::Integer => quote! { i64},
          LiteralBaseType::Float => quote! {f64},
          LiteralBaseType::String => quote! {String},
          LiteralBaseType::TimeStamp => quote! {TimeStamp},
        }
      }
      else
      {
        quote! { graphcore::Value }
      }
    }
    Property::Map(_) =>
    {
      quote! {std::collections::HashMap<String, graphcore::Value>}
    }
    Property::Optional(opt) =>
    {
      let ct = property_type(opt)?;
      quote! { Option<#ct> }
    }
  })
}

pub(super) fn generate_module_impl(input: ParsedInput) -> Result<TokenStream, syn::Error>
{
  let ParsedInput { ident, filename } = input;

  let my_crate = my_crate();

  // Find gqls file, relative to CARGO_MANIFEST_DIR/src
  let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());

  let gqls_path = Path::new(&root).join("src/").join(filename.value());
  if !gqls_path.exists()
  {
    return Err(syn::Error::new(
      filename.span(),
      format!(
        "Cannot find: {:?} in src, make sure the path is relative to src.",
        filename.value()
      ),
    ));
  }

  let contents = std::fs::read_to_string(gqls_path);

  // Fragments used in the output
  let mut creation_functions: Vec<proc_macro2::TokenStream> = Default::default();
  let mut nodes: Vec<proc_macro2::TokenStream> = Default::default();
  let mut edges: Vec<proc_macro2::TokenStream> = Default::default();

  match contents
  {
    Ok(contents) =>
    {
      // Parse the file
      let ast = gqlparser::gqls::parse_schema(&contents).map_err(|e| {
        syn::Error::new(filename.span(), format!("Failed to parse schema: {:?}", e))
      })?;
      // Generate the fragments for nodes
      for node in ast.nodes
      {
        let node_struct_name = syn::Ident::new(
          &stringcase::pascal_case(&node.identifier),
          Span::call_site(),
        );
        let create_node_function_name =
          format_ident!("create_{}", stringcase::snake_case(&node.identifier));
        // Fields
        let mut arg_names = Vec::<syn::Ident>::default();
        let mut arg_names_string = Vec::<String>::default();
        let mut arg_types = Vec::<TokenStream>::default();

        for field in node.properties
        {
          arg_names.push(syn::Ident::new(
            &snake_case(&field.0),
            proc_macro2::Span::call_site(),
          ));
          arg_names_string.push(field.0.to_owned());
          arg_types.push(property_type(&field.1)?);
        }
        let labels = &node.labels;
        let labels = quote! {labels![#(#labels),*]};

        // Node structure
        nodes.push(quote::quote! {
          pub struct #node_struct_name
          {
            pub(super) key: graphcore::Key,
            pub(super) interface: Box<dyn QueryInterface>,
          }

          impl #node_struct_name
          {
            #(
              // Retrieve property #arg_names from the database
              pub fn #arg_names(&self) -> Result<#arg_types>
              {
                let mut builder = gqb::Builder::default();
                let var = builder.match_node(#labels, value_map!());
                builder.where_statement(eb::equal(eb::function_call("id", (var,)), self.key.into()));
                builder.return_property(var, vec![#arg_names_string], #arg_names_string);
                let r = self.interface.execute_builder(builder)?.unwrap();
                let val: #arg_types = r.value(0,0)?.try_into()?;
                Ok(val.to_owned())
              }
            )*
          }
        });

        creation_functions.push(quote::quote! {
          /// Create a new node,
          pub fn #create_node_function_name(&self, #(#arg_names: impl Into<#arg_types>),*) -> Result<nodes::#node_struct_name>
          {
            let mut builder = gqb::Builder::default();
            let variable = builder.create_node(#labels, value_map!(#(#arg_names_string => #arg_names.into()),*));
            builder.return_expression(eb::function_call("id", (variable,)), "id");
            let r = self.interface.execute_builder(builder)?.unwrap();
            let key: &graphcore::Key = r.value(0,0)?.try_into_ref()?;
            Ok(nodes::#node_struct_name {
              key: key.to_owned(),
              interface: self.interface.clone_interface(),
            })
          }
        });
      }
    }
    Err(e) => Err(syn::Error::new(
      filename.span(),
      format!("Error while reading file '{}': '{}'.", filename.value(), e),
    ))?,
  }

  Ok(quote::quote! {

    /// Module with easy to use API generated from #filename
    pub mod #ident {
      use gqb::expression_builder as eb;
      use #my_crate::{gqb, anyhow, graphcore::*, QueryInterface};

      type Result<T, E = anyhow::Error> = std::result::Result<T,E>;
      /// Nodes
      pub mod nodes {
        use super::*;
        #(#nodes)*
      }
      /// Edges
      pub mod edges {
        use super::*;
        #(#edges)*
      }
      /// Base class to access the graph definied in #filename
      pub struct Graph
      {
        interface: Box<dyn QueryInterface>,
      }
      impl Graph
      {
        /// Create a new instance of the graph API for the given interface
        pub fn new<TQueryInterface>(interface: TQueryInterface) -> Self
          where TQueryInterface: QueryInterface + 'static
        {
          Self
          {
            interface: Box::new(interface),
          }
        }
        #(#creation_functions)*
      }
    }
  })
}
