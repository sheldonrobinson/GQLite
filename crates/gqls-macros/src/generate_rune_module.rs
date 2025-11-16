use std::collections::HashMap;

use gqlparser::gqls::{ast, prelude::*};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use stringcase::snake_case;

use crate::common::{expand_element, my_crate, parse_gqls_file};

pub(super) struct ParsedInput
{
  /// visibility of the generated module
  pub visibility: syn::Visibility,
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
    let visibility: syn::Visibility = input.parse()?;
    let rune_module_name = input.parse()?;
    let _: syn::Token![,] = input.parse()?;
    let rust_module_name = input.parse()?;
    let _: syn::Token![,] = input.parse()?;
    let filename = input.parse()?;
    Ok(Self {
      visibility,
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
          LiteralBaseType::String => quote! {rune::Ref<str>},
          LiteralBaseType::TimeStamp => quote! {rune::Ref<gqliterune::TimeStamp>},
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
    visibility,
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
  let mut to_generic_node_function = TokenStream::new();
  let mut to_specific_node_function = TokenStream::new();

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
    let labels_vec = quote! {vec![#(#labels),*]};

    nodes_structs.push(quote! {
      #[derive(rune::Any)]
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
        #[rune::function]
        fn element_key(&self) -> gqliterune::Key
        {
          self.inner.element_key().into()
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
    to_generic_node_function.extend(quote! {
      nodes::#node_struct_name::HASH => {
        let node = value.borrow_ref::<nodes::#node_struct_name>()?;
        Ok(node.inner.clone().into_generic_node())
      }
    });
    to_specific_node_function.extend(quote! {
      if #my_crate::contains_all(node.labels(), &#labels_vec)
      {
        return Ok(
          rune::to_value(
            nodes::#node_struct_name {
              inner: #rust_module_name::nodes::#node_struct_name::from_node(
                node,
                query_interface,
                graph_name
              )?
            }
          )?
        )
      }
    });

    nodes_declare.push(quote! {
      m.ty::<nodes::#node_struct_name>()?;
      m.function_meta(nodes::#node_struct_name::element_key)?;
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
      #[rune::function]
      fn #match_node_function_name(&self) -> Result<Vec<nodes::#node_struct_name>>
      {
        let nodes = self.inner.#match_node_function_name()?;
        Ok(nodes.into_iter().map(|node|
          nodes::#node_struct_name {
            inner: node,
          }
        ).collect())
      }
    });
    graph_functions_declare.push(quote! {
        m.function_meta(Graph::#create_node_function_name)?;
        m.function_meta(Graph::#match_node_function_name)?;
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
      #[derive(rune::Any)]
      #[rune(item = ::#rune_module_name::edges)]
      pub struct #edge_struct_name
      {
        pub(super) inner: #rust_module_name::edges::#edge_struct_name<GenericNode, GenericNode>,
      }
      impl #edge_struct_name
      {
        #[rune::function]
        fn element_key(&self) -> gqliterune::Key
        {
          self.inner.element_key().into()
        }
        #[rune::function]
        fn source(&self) -> Result<rune::Value>
        {
          to_specific_node(self.inner.source().clone())
        }
        #[rune::function]
        fn destination(&self) -> Result<rune::Value>
        {
          to_specific_node(self.inner.destination().clone())
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
    });

    edges_declare.extend(quote! {
      m.ty::<edges::#edge_struct_name>()?;
      m.function_meta(edges::#edge_struct_name::element_key)?;
      m.function_meta(edges::#edge_struct_name::source)?;
      m.function_meta(edges::#edge_struct_name::destination)?;
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
          to_generic_node(&source)?,
          to_generic_node(&destination)?,
          #(#field_names.into_rust_argument()),*)?;

        Ok(
          edges::#edge_struct_name {
            inner: edge,
          }
        )
      }
      #[rune::function]
      fn #match_edge_function_name(&self, source: Option<rune::Value>, destination: Option<rune::Value>) -> Result<Vec<edges::#edge_struct_name>>
      {
        let edges = self.inner.#match_edge_function_name(
          source.map(|n| to_generic_node(&n)).transpose()?,
          destination.map(|n| to_generic_node(&n)).transpose()?
        )?;
        Ok(edges.into_iter().map(|edge|
          edges::#edge_struct_name
          {
            inner: edge,
          }
        ).collect())
      }
    });
    graph_functions_declare.push(quote! {
        m.function_meta(Graph::#create_edge_function_name)?;
        m.function_meta(Graph::#match_edge_function_name)?;
    });
  }

  Ok(quote::quote! {
    #visibility mod #rune_module_name
    {
      use gqb::expression_builder as eb;
      use #my_crate::{gqb, anyhow, graphcore::*, GenericNode, QueryInterface, Element, ElementType, Node, Edge, rune::*};
      use rune::TypeHash as _;

      type Result<T, E = anyhow::Error> = std::result::Result<T,E>;
      mod nodes
      {
        use super::*;
        #(#nodes_structs)*
      }

      mod edges
      {
        use super::*;
        #edges_structs
      }

      fn to_specific_node(generic_node: GenericNode)
        -> Result<rune::Value>
      {
        let (key, query_interface, graph_name, labels) = generic_node.unpack();
        let node = graphcore::Node::new(key, labels, graphcore::value_map!());
        #to_specific_node_function
        Err(anyhow::anyhow!("Unknown node with labels {:?}.", node.labels()))
      }

      fn to_generic_node(value: &rune::Value) -> Result<GenericNode> {
        use rune::FromValue;
        use Element;
        match value.type_hash() {
          #to_generic_node_function
          _ => Err(anyhow::anyhow!("Unknown hash.")),
        }
      }

      fn get_metadata(hash: rune::Hash) -> Result<&'static ElementMetadata> {
        match hash {
          #metadata_function
          _ => Err(anyhow::anyhow!("Unknown hash.")),
        }
      }

      #[derive(rune::Any)]
      #[rune(item = ::#rune_module_name)]
      pub struct Graph
      {
        inner: #rust_module_name::Graph,
      }

      impl Graph
      {
        pub fn new<TQueryInterface>(interface: TQueryInterface, graph_name: impl Into<String>) -> Result<Graph>
          where TQueryInterface: QueryInterface + 'static
        {
          Ok(
            Self {
              inner: #rust_module_name::Graph::new(interface, graph_name)?
            }
          )
        }
        #[rune::function(keep, path = Self::new)]
         fn new_rune(connection: gqliterune::Connection, graph_name: rune::Ref<str>) -> Result<Self>
        {
          Ok(
            Self {
              inner: #rust_module_name::Graph::new(connection.connection_clone(), graph_name.to_string())?
            }
          )
          // Ok(Self::new_rune(connection.connection_clone(), graph_name.to_string())?)
        }
        #(#graph_functions)*
      }

      /// Create the rune module
      fn rune_module() -> Result<rune::Module>
      {
        let mut m = rune::Module::with_crate(#rune_module_name_string)?;
        m.ty::<Graph>()?;
        m.function_meta(Graph::new_rune__meta)?;
        #(#graph_functions_declare)*
        Ok(m)
      }
      fn rune_nodes_module() -> Result<rune::Module>
      {
        let mut m = rune::Module::with_crate_item(#rune_module_name_string, vec!["nodes"])?;
        #(#nodes_declare)*
        Ok(m)
      }
      fn rune_edges_module() -> Result<rune::Module>
      {
        let mut m = rune::Module::with_crate_item(#rune_module_name_string, vec!["edges"])?;
        #edges_declare
        Ok(m)
      }
      pub fn install(context: &mut rune::Context) -> Result<()>
      {
        context.install(rune_module()?)?;
        context.install(rune_nodes_module()?)?;
        context.install(rune_edges_module()?)?;
        Ok(())
      }

    }

  })
}
