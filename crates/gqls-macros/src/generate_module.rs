use gqlparser::gqls::prelude::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use stringcase::snake_case;

use crate::common::{my_crate, parse_gqls_file, expand_element};

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

  let ast = parse_gqls_file(&filename)?;

    // Fragments used in the output
  let mut graph_functions: Vec<proc_macro2::TokenStream> = Default::default();
  let mut elements: Vec<proc_macro2::TokenStream> = Default::default();
  let mut nodes: Vec<proc_macro2::TokenStream> = Default::default();
  let mut edges: Vec<proc_macro2::TokenStream> = Default::default();

  // Generate the elements
  for (identifier, properties_definition) in ast.elements.iter()
  {
    let element_trait_name =
      syn::Ident::new(&stringcase::pascal_case(identifier), Span::call_site());

    // Fields
    let mut property_names = Vec::<syn::Ident>::default();
    let mut property_setters = Vec::<syn::Ident>::default();
    let mut property_names_string = Vec::<String>::default();
    let mut property_types = Vec::<TokenStream>::default();

    for field in properties_definition.properties.iter()
    {
      let snaked_case_name = snake_case(field.0);
      property_names.push(syn::Ident::new(
        &snaked_case_name,
        proc_macro2::Span::call_site(),
      ));
      property_names_string.push(field.0.to_owned());
      property_setters.push(format_ident!("set_{}", snaked_case_name));
      property_types.push(property_type(field.1)?);
    }

    elements.push(quote! {
      /// Trait for the element #element_trait_name
      pub trait #element_trait_name: Element
      {
        #(
          // Retrieve property #arg_names from the database
          fn #property_names(&self) -> Result<#property_types>
          {
            let mut builder = gqb::Builder::default();
            builder.use_graph(self.graph_name().clone());
            let var = match self.element_type() {
              #my_crate::ElementType::Node => builder.match_node(labels![#identifier], value_map!()),
              #my_crate::ElementType::Edge => builder.match_edge(None, labels![#identifier], value_map!(), None),
            };
            builder.where_statement(eb::equal(eb::function_call("id", (var,)), self.element_key().into()));
            builder.return_property(var, vec![#property_names_string], #property_names_string);
            let r = self.query_interface().execute_builder(builder)?.unwrap();
            let val: #property_types = r.value(0,0)?.try_into()?;
            Ok(val.to_owned())
          }
          fn #property_setters(&self, value: impl Into<#property_types>) -> Result<()>
          {
            let mut builder = gqb::Builder::default();
            builder.use_graph(self.graph_name().clone());
            let var = match self.element_type() {
              #my_crate::ElementType::Node => builder.match_node(labels![#identifier], value_map!()),
              #my_crate::ElementType::Edge => builder.match_edge(None, labels![#identifier], value_map!(), None),
            };
            builder.where_statement(eb::equal(eb::function_call("id", (var,)), self.element_key().into()));
            builder.set_assignment(var, vec![#property_names_string], value.into().into());
            self.query_interface().execute_builder(builder)?;
            Ok(())
          }
        )*
      }
    });
  }

  // Generate the fragments for nodes
  for node in ast.nodes
  {
    let node_struct_name =
      syn::Ident::new(&stringcase::pascal_case(&node.label), Span::call_site());
    let create_node_function_name =
      format_ident!("create_{}", stringcase::snake_case(&node.label));
    let match_node_function_name =
      format_ident!("match_{}", stringcase::snake_case(&node.label));
    // Fields
    let mut arg_names = Vec::<syn::Ident>::default();
    let mut arg_names_string = Vec::<String>::default();
    let mut arg_types = Vec::<TokenStream>::default();

    let expended_element = expand_element(&ast.elements, node.label)?;

    for field in expended_element.properties
    {
      arg_names.push(syn::Ident::new(
        &snake_case(&field.0),
        proc_macro2::Span::call_site(),
      ));
      arg_names_string.push(field.0.to_owned());
      arg_types.push(property_type(&field.1)?);
    }
    let elements = expended_element
      .labels
      .iter()
      .map(|x| syn::Ident::new(&stringcase::pascal_case(x), proc_macro2::Span::call_site()));
    let labels = &expended_element.labels;
    let labels = quote! {labels![#(#labels),*]};

    // Node structure
    nodes.push(quote::quote! {
      pub struct #node_struct_name
      {
        pub(super) key: graphcore::Key,
        pub(super) interface: Box<dyn QueryInterface>,
        pub(super) graph_name: String,
      }

      impl Element for #node_struct_name
      {
        fn query_interface(&self) -> &dyn QueryInterface
        {
          use std::ops::Deref;
          self.interface.deref()
        }
        fn graph_name(&self) -> &String
        {
          &self.graph_name
        }
        fn element_key(&self) -> graphcore::Key
        {
          self.key
        }
        fn element_type(&self) -> #my_crate::ElementType
        {
          #my_crate::ElementType::Node
        }
      }
      impl Node for #node_struct_name
      {
        fn from_key(key: graphcore::Key, interface: Box<dyn QueryInterface>, graph_name: impl Into<String>) -> Self
        {
          Self {
            key,
            interface,
            graph_name: graph_name.into()
          }
        }
        fn labels() -> Vec<String>
        {
          #labels
        }
      }

      #(
        impl elements::#elements for #node_struct_name
        {}
      )*
    });

    graph_functions.push(quote::quote! {
      /// Create a new node,
      pub fn #create_node_function_name(&self, #(#arg_names: impl Into<#arg_types>),*) -> Result<nodes::#node_struct_name>
      {
        let mut builder = gqb::Builder::default();
        builder.use_graph(self.graph_name.clone());
        let variable = builder.create_node(#labels, value_map!(#(#arg_names_string => #arg_names.into()),*));
        builder.return_expression(eb::function_call("id", (variable,)), "id");
        let r = self.interface.execute_builder(builder)?.unwrap();
        let key: &graphcore::Key = r.get(0,0)?;
        Ok(nodes::#node_struct_name {
          key: key.to_owned(),
          interface: self.interface.clone_interface(),
          graph_name: self.graph_name.clone(),
        })
      }
      pub fn #match_node_function_name(&self) -> Result<Vec<nodes::#node_struct_name>>
      {
        let mut builder = gqb::Builder::default();
        builder.use_graph(self.graph_name.clone());
        let variable = builder.match_node(#labels, value_map!());
        builder.return_expression(eb::function_call("id", (variable,)), "id");
        let t = self.interface.execute_builder(builder)?.unwrap();
        let mut res = Vec::<nodes::#node_struct_name>::default();
        for r in 0..t.rows()
        {
          let key: &graphcore::Key = t.get(r,0)?;
          res.push(nodes::#node_struct_name {
            key: key.to_owned(),
            interface: self.interface.clone_interface(),
            graph_name: self.graph_name.clone(),
          });
        }
        Ok(res)
      }
    });
  }

  // Generate the fragments for edges
  let mut generated_edges = Vec::<String>::new();
  for edge in ast.edges
  {
    let node_source_name =
      syn::Ident::new(&stringcase::pascal_case(&edge.source), Span::call_site());
    let create_edge_function_name =
      format_ident!("create_{}", stringcase::snake_case(&edge.label));
    let match_edge_function_name =
      format_ident!("match_{}", stringcase::snake_case(&edge.label));
    let node_destination_name = syn::Ident::new(
      &stringcase::pascal_case(&edge.destination),
      Span::call_site(),
    );
    let edge_struct_name =
      syn::Ident::new(&stringcase::pascal_case(&edge.label), Span::call_site());
    let into_edge_trait_name = format_ident!("Into{}", stringcase::pascal_case(&edge.label));

    if !generated_edges.contains(&edge.label)
    {
      generated_edges.push(edge.label.clone());

      // Fields
      let mut arg_names = Vec::<syn::Ident>::default();
      let mut arg_names_string = Vec::<String>::default();
      let mut arg_types = Vec::<TokenStream>::default();

      let expended_element = expand_element(&ast.elements, edge.label)?;

      for field in expended_element.properties
      {
        arg_names.push(syn::Ident::new(
          &snake_case(&field.0),
          proc_macro2::Span::call_site(),
        ));
        arg_names_string.push(field.0.to_owned());
        arg_types.push(property_type(&field.1)?);
      }
      let elements = expended_element
        .labels
        .iter()
        .map(|x| syn::Ident::new(&stringcase::pascal_case(x), proc_macro2::Span::call_site()));
      let labels = &expended_element.labels;
      let labels = quote! {labels![#(#labels),*]};

      // Define edge structure
      edges.push(quote::quote! {
        pub struct #edge_struct_name<TSource, TDestination>
          where (TSource, TDestination): #into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          pub(super) source: graphcore::Key,
          pub(super) destination: graphcore::Key,
          pub(super) key: graphcore::Key,
          pub(super) interface: Box<dyn QueryInterface>,
          pub(super) graph_name: String,
          pub(super) source_ghost: std::marker::PhantomData<TSource>,
          pub(super) destination_ghost: std::marker::PhantomData<TDestination>,
        }
        impl<TSource, TDestination> #edge_struct_name<TSource, TDestination>
          where (TSource, TDestination): #into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          /// Access the source of the edge
          pub fn source(&self) -> TSource
          {
            TSource::from_key(self.source, self.interface.clone_interface(), self.graph_name.clone())
          }
          /// Access the destination of the edge
          pub fn destination(&self) -> TDestination
          {
            TDestination::from_key(self.destination, self.interface.clone_interface(), self.graph_name.clone())
          }
        }
        impl<TSource, TDestination> Element for #edge_struct_name<TSource, TDestination>
          where (TSource, TDestination): #into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          fn query_interface(&self) -> &dyn QueryInterface
          {
            use std::ops::Deref;
            self.interface.deref()
          }
          fn graph_name(&self) -> &String
          {
            &self.graph_name
          }
          fn element_key(&self) -> graphcore::Key
          {
            self.key
          }
          fn element_type(&self) -> #my_crate::ElementType
          {
            #my_crate::ElementType::Edge
          }
        }
        impl<TSource, TDestination> Edge for #edge_struct_name<TSource, TDestination>
          where (TSource, TDestination): #into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          fn labels() -> Vec<String>
          {
            #labels
          }
        }
        pub trait #into_edge_trait_name
        {
        }

        #(
          impl<TSource, TDestination> elements::#elements for #edge_struct_name<TSource, TDestination>
            where (TSource, TDestination): #into_edge_trait_name,
                  TSource: Node,
                  TDestination: Node,
          {}
        )*
      });

      graph_functions.push(quote::quote! {
        /// Create a new edge,
        pub fn #create_edge_function_name<TSource, TDestination>(&self, source: &TSource, destination: &TDestination, #(#arg_names: impl Into<#arg_types>),*)
          -> Result<edges::#edge_struct_name<TSource, TDestination>>
          where (TSource, TDestination): edges::#into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          let mut builder = gqb::Builder::default();
          builder.use_graph(self.graph_name.clone());
          let source_var = builder.match_node(TSource::labels(), value_map!());
          let destination_var = builder.match_node(TDestination::labels(), value_map!());
          builder.where_statement(
            eb::and(
              eb::equal(eb::function_call("id", (source_var,)), source.element_key().into()),
              eb::equal(eb::function_call("id", (destination_var,)), destination.element_key().into()),
            )
          );
          let variable = builder.create_edge(source_var, #labels, value_map!(#(#arg_names_string => #arg_names.into()),*), destination_var);
          builder.return_expression(eb::function_call("id", (variable,)), "id");
          let r = self.interface.execute_builder(builder)?.unwrap();
          let key: &graphcore::Key = r.value(0,0)?.try_into_ref()?;
          Ok(edges::#edge_struct_name {
            source: source.element_key(),
            destination: destination.element_key(),
            key: key.to_owned(),
            interface: self.interface.clone_interface(),
            graph_name: self.graph_name.clone(),
            source_ghost: Default::default(),
            destination_ghost: Default::default(),
          })
        }
        // Match
        pub fn #match_edge_function_name<TSource, TDestination>(&self, source: Option<TSource>, destination: Option<TDestination>)
          -> Result<Vec<edges::#edge_struct_name<TSource, TDestination>>>
          where (TSource, TDestination): edges::#into_edge_trait_name,
                TSource: Node,
                TDestination: Node,
        {
          let mut builder = gqb::Builder::default();
          builder.use_graph(self.graph_name.clone());

          let (source_var, destination_var) = match (source, destination)
          {
            (None, None) => (None, None),
            (Some(source), None) => {
              let source_var = builder.match_node(TSource::labels(), value_map!());
              builder.where_statement(
                eb::equal(eb::function_call("id", (source_var,)), source.element_key().into()),
              );
              (Some(source_var), None)
            }
            (None, Some(destination)) => {
              let destination_var = builder.match_node(TDestination::labels(), value_map!());
              builder.where_statement(
                eb::equal(eb::function_call("id", (destination_var,)), destination.element_key().into()),
              );
              (None, Some(destination_var))
            }
            (Some(source), Some(destination)) => {
              let source_var = builder.match_node(TSource::labels(), value_map!());
              let destination_var = builder.match_node(TDestination::labels(), value_map!());
              builder.where_statement(
                eb::and(
                  eb::equal(eb::function_call("id", (source_var,)), source.element_key().into()),
                  eb::equal(eb::function_call("id", (destination_var,)), destination.element_key().into()),
                )
              );
              (Some(source_var), Some(destination_var))
            }
          };
          let (variable, _) = builder.match_path(source_var, #labels, value_map!(), destination_var);
          builder.return_variable(variable, "path");
          
          let t = self.interface.execute_builder(builder)?.unwrap();
          let mut results = Vec::<edges::#edge_struct_name<TSource, TDestination>>::default();
          for r in 0..t.rows()
          {
            let path: &graphcore::SinglePath = t.get(r,0)?;
            results.push(edges::#edge_struct_name {
              source: path.source().key(),
              destination: path.destination().key(),
              key: path.key(),
              interface: self.interface.clone_interface(),
              graph_name: self.graph_name.clone(),
              source_ghost: Default::default(),
              destination_ghost: Default::default(),
            });
          }
          Ok(results)
        }
      });
    }
    edges.push(quote::quote! {
      impl<TSource, TDestination> #into_edge_trait_name for (TSource, TDestination)
        where TSource: elements::#node_source_name, TDestination: elements::#node_destination_name
      {}
    });
  }

  // Assemble the output
  Ok(quote::quote! {

    /// Module with easy to use API generated from #filename
    pub mod #ident {
      use gqb::expression_builder as eb;
      use #my_crate::{gqb, anyhow, graphcore::*, QueryInterface, Element, Node, Edge};

      type Result<T, E = anyhow::Error> = std::result::Result<T,E>;
      /// Elements
      pub mod elements {
        use super::*;
        #(#elements)*
      }
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
        graph_name: String,
      }
      impl Graph
      {
        /// Create a new instance of the graph API for the given interface
        pub fn new<TQueryInterface>(interface: TQueryInterface, graph_name: impl Into<String>) -> Result<Self>
          where TQueryInterface: QueryInterface + 'static
        {
          let graph_name = graph_name.into();
          let mut builder = gqb::Builder::default();
          builder.create_graph(&graph_name, true);
          let _ = interface.execute_builder(builder)?;
          Ok(
            Self
            {
              interface: Box::new(interface),
              graph_name,
            }
          )
        }
        #(#graph_functions)*
        /// Delete a node
        pub fn delete_node<TNode: Node>(&self, node: TNode) -> Result<()>
        {
          let mut builder = gqb::Builder::default();
          builder.use_graph(self.graph_name.clone());
          let var = builder.match_node(TNode::labels(), value_map!());
          builder.where_statement(eb::equal(eb::function_call("id", (var,)), node.element_key().into()));
          builder.detach_delete(var);
          let _ = self.interface.execute_builder(builder)?;
          Ok(())
        }
        /// Delete an edge
        pub fn delete_edge<TEdge: Edge>(&self, edge: TEdge) -> Result<()>
        {
          let mut builder = gqb::Builder::default();
          builder.use_graph(self.graph_name.clone());
          let var = builder.match_edge(None, TEdge::labels(), value_map!(), None);
          builder.where_statement(eb::equal(eb::function_call("id", (var,)), edge.element_key().into()));
          builder.delete(var);
          let _ = self.interface.execute_builder(builder)?;
          Ok(())
        }
      }
    }
  })
}
