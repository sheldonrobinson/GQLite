use std::collections::HashMap;

use gqlparser::gqls::{ast, prelude::*};
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
  let mut edges_declare = TokenStream::new();
  let mut edges_structs = TokenStream::new();
  let mut metadata_function = TokenStream::new();
  let mut key_function = TokenStream::new();

  let ast = parse_gqls_file(&filename)?;

  // Generate nodes
  for node in ast.nodes
  {
    let node_struct_name =
      syn::Ident::new(&stringcase::pascal_case(&node.label), Span::call_site());
    let node_metadata_const_name = format_ident!(
      "{}_METADATA",
      stringcase::snake_case(&node.label).to_uppercase()
    );
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

    let labels = &expended_element.labels;
    let labels_slice = quote! {&[#(#labels),*]};

    nodes_structs.push(quote! {
      #[derive(Any)]
      #[rune(item = ::#rune_module_name::nodes)]
      pub struct #node_struct_name
      {
        pub(super) inner: #rust_module_name::nodes::#node_struct_name,
      }
      impl #node_struct_name
      {
        pub(super) fn inner_ref(&self) -> &#rust_module_name::nodes::#node_struct_name
        {
          &self.inner
        }
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
      pub const #node_metadata_const_name: ElementMetadata = ElementMetadata {
        element_type: ElementType::Node,
        labels: #labels_slice,
      };
    });

    metadata_function.extend(quote! {
      nodes::#node_struct_name::HASH => Ok(&nodes::#node_metadata_const_name),
    });
    key_function.extend(quote! {
      nodes::#node_struct_name::HASH => {
        let node = value.borrow_ref::<nodes::#node_struct_name>()?;
        Ok(node.inner_ref().element_key())
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

  // Edges

  let mut edge_groups: HashMap<String, Vec<ast::Edge>> = HashMap::new();
  for edge in ast.edges
  {
    edge_groups
      .entry(edge.label.clone())
      .or_default()
      .push(edge);
  }

  for (label, edges) in edge_groups
  {
    let edge_struct_name = syn::Ident::new(&stringcase::pascal_case(&label), Span::call_site());
    let into_edge_trait_name = format_ident!("Into{}", stringcase::pascal_case(&label));
    let create_edge_function_name = format_ident!("create_{}", stringcase::snake_case(&label));
    let match_edge_function_name = format_ident!("match_{}", stringcase::snake_case(&label));

    let mut field_names = Vec::<syn::Ident>::default();
    let mut field_set_function_name = Vec::<syn::Ident>::default();
    let mut field_arg_types = Vec::<TokenStream>::default();
    let mut field_return_types = Vec::<TokenStream>::default();

    let expended_element = expand_element(&ast.elements, label)?;
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

    edges_structs.extend(quote! {
      impl #rust_module_name::edges::#into_edge_trait_name for (GenericNode, GenericNode)
      {}
      #[derive(Any)]
      #[rune(item = ::#rune_module_name::edges)]
      pub struct #edge_struct_name
      {
        pub(super) inner: #rust_module_name::edges::#edge_struct_name<GenericNode, GenericNode>,
      }
      impl #edge_struct_name
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

    edges_declare.extend(quote! {
      m.ty::<edges::#edge_struct_name>()?;
      #(
        m.function_meta(edges::#edge_struct_name::#field_names)?;
        m.function_meta(edges::#edge_struct_name::#field_set_function_name)?;
      )*
    });
    let source_labels: Vec<_> = edges.iter().map(|x| &x.source).collect();
    let destination_labels: Vec<_> = edges.iter().map(|x| &x.destination).collect();

    graph_functions.push(quote! {
      #[rune::function]
      fn #create_edge_function_name(&self, source: rune::Value, destination: rune::Value, #(#field_names: #field_arg_types),*) -> Result<edges::#edge_struct_name>
      {
        let source_metadata = get_metadata(source.type_hash())?;
        let destination_metadata = get_metadata(destination.type_hash())?;
          if !(
        #( (source_metadata.labels.contains(&#source_labels) && destination_metadata.labels.contains(&#destination_labels) )
        )||*)
          {
            return Err(anyhow::anyhow!("Invalid node combination for edge"))
          }
        let edge = self.inner.#create_edge_function_name(
          &edges::GenericNode
          {
            key: get_key(&source)?,
            query_interface: self.inner.query_interface().clone_interface(),
            graph_name: self.inner.graph_name().clone()
          },
          &edges::GenericNode {
            key: get_key(&destination)?,
            query_interface: self.inner.query_interface().clone_interface(),
            graph_name: self.inner.graph_name().clone()
          },
          #(#field_names.into_rust_argument()),*)?;

        Ok(
          edges::#edge_struct_name {
            inner: edge,
          }
        )
      }
    });
    graph_functions_declare.push(quote! {
        m.function_meta(Graph::#create_edge_function_name)?;
    });
  }

  Ok(quote::quote! {
    mod #rune_module_name
    {
      use gqb::expression_builder as eb;
      use #my_crate::{gqb, anyhow, graphcore::*, QueryInterface, Element, ElementType, Node, Edge, rune::*};

      mod nodes
      {
        use super::*;
        #(#nodes_structs)*
      }

      mod edges
      {
        use super::*;
        pub(super) struct GenericNode
        {
          pub key: graphcore::Key,
          pub query_interface: Box<dyn crate::QueryInterface>,
          pub graph_name: String,
        }

        impl Element for GenericNode {
            fn element_key(&self) -> graphcore::Key {
                self.key
            }
            fn element_type(&self) -> ElementType {
                ElementType::Node
            }
            fn graph_name(&self) -> &String {
                &self.graph_name
            }
            fn query_interface(&self) -> &dyn crate::QueryInterface {
                use std::ops::Deref;
                self.query_interface.deref()
            }
        }

        impl Node for GenericNode
        {
          fn from_key(
              key: graphcore::Key,
              query_interface: Box<dyn crate::QueryInterface>,
              graph_name: impl Into<String>,
            ) -> Self {
              GenericNode {
                key,
                query_interface,
                graph_name: graph_name.into()
              }
          }
          fn labels() -> Vec<String> {
              Default::default()
          }
        }
        #edges_structs
      }

      fn get_key(value: &rune::Value) -> Result<graphcore::Key> {
        use rune::FromValue;
        use Element;
        match value.type_hash() {
          #key_function
          _ => Err(anyhow::anyhow!("Unknown hash.")),
        }
      }

      fn get_metadata(hash: Hash) -> Result<&'static ElementMetadata> {
        match hash {
          #metadata_function
          _ => Err(anyhow::anyhow!("Unknown hash.")),
        }
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
