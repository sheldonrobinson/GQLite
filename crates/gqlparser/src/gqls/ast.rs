//! AST module

use indexmap::IndexMap;

use crate::gqls::prelude::*;

/// Abstract Syntax Tree of the schema
#[derive(Debug)]
pub struct Ast
{
  /// List of nodes
  pub nodes: Vec<Node>,
  /// List of edges
  pub edges: Vec<Edge>,
}

/// Represent a node
#[derive(Debug)]
pub struct Node
{
  /// Main label of the node, used as identifer
  pub identifier: String,
  /// Labels of the node
  pub labels: Vec<String>,
  /// Properties of the node
  pub properties: IndexMap<String, Property>,
}

/// Represent an edge
#[derive(Debug)]
pub struct Edge
{
  /// Main label of the edge, used as identifer
  pub identifer: String,
  /// Identifier of the source
  pub source: String,
  /// Labels of the edge
  pub labels: Vec<String>,
  ///   /// Identifier of the destination
  pub destination: String,
  /// Properties of the edge
  pub properties: IndexMap<String, Property>,
}
