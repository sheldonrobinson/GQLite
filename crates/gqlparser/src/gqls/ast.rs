//! AST module

use std::collections::HashMap;

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
  /// Labels of the node
  pub labels: Vec<String>,
  /// Properties of the node
  pub properties: HashMap<String, Property>,
}

/// Represent an edge
#[derive(Debug)]
pub struct Edge
{
  /// Label of the source
  pub source: String,
  /// Labels of the edge
  pub labels: Vec<String>,
  /// Label of the destination
  pub destination: String,
  /// Properties of the edge
  pub properties: HashMap<String, Property>,
}
