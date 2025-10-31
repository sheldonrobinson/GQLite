use gqlparser::gqls::prelude::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use stringcase::snake_case;

use crate::common::{expand_element, my_crate, parse_gqls_file};

pub(super) struct ParsedInput
{
  /// Identifier for the generated module.
  rune_module_name: syn::Ident,
  /// Identifier for the generated module.
  rust_module_name: syn::Path,
  /// File name with the GQL Schema.
  filename: syn::LitStr,
}

/// Implement the Parse trait from syn to parse the input
impl syn::parse::Parse for ParsedInput
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
  {
    let rune_module_name = input.parse()?;
    let _: syn::Token![,] = input.parse()?;
    let rust_module_name = input.parse()?;
    let _: syn::Token![,] = input.parse()?;
    let filename = input.parse()?;
    Ok(Self {
      rune_module_name,
      rust_module_name,
      filename,
    })
  }
}

fn property_arg_type(prop: &Property) -> Result<TokenStream, syn::Error>
{
  Ok(match prop
  {
    Property::Any => quote! { rune::Value },
    Property::Array(_) =>
    {
      let ct = property_arg_type(prop)?;

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
          LiteralBaseType::Integer => quote! {i64},
          LiteralBaseType::Float => quote! {f64},
          LiteralBaseType::String => quote! {Ref<str>},
          LiteralBaseType::TimeStamp => quote! {Ref<gqliterune::TimeStamp>},
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
      let ct = property_arg_type(opt)?;
      quote! { Option<#ct> }
    }
  })
}

fn property_return_type(prop: &Property) -> Result<TokenStream, syn::Error>
{
  Ok(match prop
  {
    Property::Any => quote! { rune::Value },
    Property::Array(_) =>
    {
      let ct = property_return_type(prop)?;

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
          LiteralBaseType::TimeStamp => quote! {gqliterune::TimeStamp},
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
      let ct = property_return_type(opt)?;
      quote! { Option<#ct> }
    }
  })
}

pub(super) fn generate_rune_module_impl(input: ParsedInput) -> Result<TokenStream, syn::Error>
{
  let ParsedInput {
    rune_module_name,
    rust_module_name,
    filename,
  } = input;

  let my_crate = my_crate();
  let rune_module_name_string = rune_module_name.to_string();
  let mut graph_functions = Vec::<TokenStream>::new();
  let mut graph_functions_declare = Vec::<TokenStream>::new();
  let mut nodes_declare = Vec::<TokenStream>::new();
  let mut nodes_structs = Vec::<TokenStream>::new();

  let ast = parse_gqls_file(&filename)?;

  // Generate nodes
  for node in ast.nodes
  {
    let node_struct_name =
      syn::Ident::new(&stringcase::pascal_case(&node.label), Span::call_site());
    let create_node_function_name = format_ident!("create_{}", stringcase::snake_case(&node.label));
    let match_node_function_name = format_ident!("match_{}", stringcase::snake_case(&node.label));

    let mut field_names = Vec::<syn::Ident>::default();
    let mut field_set_function_name = Vec::<syn::Ident>::default();
    let mut field_arg_types = Vec::<TokenStream>::default();
    let mut field_return_types = Vec::<TokenStream>::default();

    let expended_element = expand_element(&ast.elements, node.label)?;
    for field in expended_element.properties
    {
      field_names.push(syn::Ident::new(
        &snake_case(&field.0),
        proc_macro2::Span::call_site(),
      ));
      field_set_function_name.push(format_ident!("set_{}", snake_case(&field.0)));
      field_arg_types.push(property_arg_type(&field.1)?);
      field_return_types.push(property_return_type(&field.1)?);
    }

    nodes_structs.push(quote! {
      #[derive(Any)]
      #[rune(item = ::#rune_module_name::nodes)]
      pub struct #node_struct_name
      {
        pub(super) inner: #rust_module_name::nodes::#node_struct_name,
      }
      impl #node_struct_name
      {
        #(
          #[rune::function]
          fn #field_names(&self) -> Result<#field_return_types>
          {
            use #rust_module_name::elements::*;
            Ok(self.inner.#field_names()?.into())
          }
          #[rune::function]
          fn #field_set_function_name(&self, value: #field_arg_types) -> Result<()>
          {
            use #rust_module_name::elements::*;
            self.inner.#field_set_function_name(value.into_rust_argument())?;
            Ok(())
          }
        )*
      }
    });

    nodes_declare.push(quote! {
      m.ty::<nodes::#node_struct_name>()?;
      #(
        m.function_meta(nodes::#node_struct_name::#field_names)?;
        m.function_meta(nodes::#node_struct_name::#field_set_function_name)?;
      )*
    });

    graph_functions.push(quote! {
      #[rune::function]
      fn #create_node_function_name(&self, #(#field_names: #field_arg_types),*) -> Result<nodes::#node_struct_name>
      {
        let node = self.inner.#create_node_function_name(#(#field_names.into_rust_argument()),*)?;
        Ok(
          nodes::#node_struct_name {
            inner: node,
          }
        )
      }
    });
    graph_functions_declare.push(quote! {
        m.function_meta(Graph::#create_node_function_name)?;
    });
  }

  Ok(quote::quote! {
    mod #rune_module_name
    {
      use gqb::expression_builder as eb;
      use #my_crate::{gqb, anyhow, graphcore::*, QueryInterface, Element, Node, Edge, rune::*};

      mod nodes
      {
        use super::*;
        #(#nodes_structs)*
      }

      #[derive(Any)]
      #[rune(item = ::#rune_module_name)]
      pub struct Graph
      {
        inner: #rust_module_name::Graph,
      }

      impl Graph
      {
        #[rune::function(path = Self::new)]
        pub fn new(connection: gqliterune::Connection, graph_name: Ref<str>) -> Result<Graph>
        {
          Ok(
            Self {
              inner: #rust_module_name::Graph::new(connection.connection_clone(), graph_name.to_string())?
            }
          )
        }
        #(#graph_functions)*
      }

      /// Create the rune module
      fn rune_module() -> Result<rune::Module>
      {
        let mut m = rune::Module::with_crate(#rune_module_name_string)?;
        m.ty::<Graph>()?;
        m.function_meta(Graph::new)?;
        #(#graph_functions_declare)*

        Ok(m)
      }
      fn rune_nodes_module() -> Result<rune::Module>
      {
        let mut m = rune::Module::with_crate_item(#rune_module_name_string, vec!["nodes"])?;
        #(#nodes_declare)*

        Ok(m)
      }
      pub fn install(context: &mut Context) -> Result<()>
      {
        context.install(rune_module()?)?;
        context.install(rune_nodes_module()?)?;
        Ok(())
      }

    }

  })
}
