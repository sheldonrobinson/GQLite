//! AST module

use indexmap::IndexMap;

use crate::gqls::prelude::*;

/// Abstract Syntax Tree of the schema
#[derive(Debug)]
pub struct Ast
{
  /// List of elements
  pub elements: IndexMap<String, PropertiesDefinition>,
  /// List of nodes
  pub nodes: Vec<Node>,
  /// List of edges
  pub edges: Vec<Edge>,
}

/// Represent an element
#[derive(Debug)]
pub struct PropertiesDefinition
{
  /// List of parament element
  pub parents: Vec<String>,
  /// Properties of the element
  pub properties: IndexMap<String, Property>,
}

/// Represent a node
#[derive(Debug)]
pub struct Node
{
  /// Main label of the node, used as identifer
  pub label: String,
}

/// Represent an edge
#[derive(Debug)]
pub struct Edge
{
  /// Identifier of the source
  pub source: String,
  /// Main label of the edge, used as identifer
  pub label: String,
  /// Identifier of the destination
  pub destination: String,
}
