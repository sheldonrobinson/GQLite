use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum EdgeDirectivity
{
  Undirected,
  Directed,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct Key
{
  pub(crate) uuid: u128,
}

impl Serialize for Key
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_u128(self.uuid)
  }
}

impl<'de> Deserialize<'de> for Key
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self {
      uuid: u128::deserialize(deserializer)?,
    })
  }
}

impl Default for Key
{
  fn default() -> Self
  {
    Key {
      uuid: uuid::Uuid::new_v4().as_u128(),
    }
  }
}

impl From<&Key> for u128
{
  fn from(value: &Key) -> Self
  {
    value.uuid
  }
}

impl From<Key> for u128
{
  fn from(value: Key) -> Self
  {
    value.uuid
  }
}

/// Represent a Node in the graph
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "node")]
pub struct Node
{
  /// uuid for the Node.
  pub(crate) key: Key,
  /// Vector of labels.
  pub(crate) labels: Vec<String>,
  /// Properties.
  pub(crate) properties: value::ValueMap,
}

impl Node
{
  /// uuid for the Node.
  pub fn key(&self) -> Key
  {
    self.key
  }
  /// Vector of labels.
  pub fn labels(&self) -> &Vec<String>
  {
    &self.labels
  }
  /// Properties.
  pub fn properties(&self) -> &value::ValueMap
  {
    &self.properties
  }
  /// Unpack Node in key, labels and properties.
  pub fn unpack(self) -> (Key, Vec<String>, value::ValueMap)
  {
    (self.key, self.labels, self.properties)
  }
}

impl std::fmt::Display for Node
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    if self.labels.is_empty()
    {
      write!(f, "(")?;
    }
    else
    {
      write!(f, "(:{} ", self.labels.join(":"))?;
    }
    write!(f, "{}", self.properties.borrow())?;
    write!(f, ")")
  }
}

/// Directed edge of the graph.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "edge")]
pub struct Edge
{
  /// uuid for the Edge.
  pub(crate) key: Key,
  #[serde(skip_serializing)]
  /// source node for the Edge, this property is used internally by the engine, but is not exported in query results, and not part of the public API.
  pub(crate) source: Node,
  /// destination node for the Edge, this property is used internally by the engine, but is not exported in query results, and not part of the public API.
  #[serde(skip_serializing)]
  pub(crate) destination: Node,
  /// Labels for the Edge.
  pub(crate) labels: Vec<String>,
  /// Properties for the Edge.
  pub(crate) properties: value::ValueMap,
}

impl Edge
{
  /// uuid for the Node.
  pub fn key(&self) -> Key
  {
    self.key
  }
  /// Vector of labels.
  pub fn labels(&self) -> &Vec<String>
  {
    &self.labels
  }
  /// Properties.
  pub fn properties(&self) -> &value::ValueMap
  {
    &self.properties
  }
  /// Unpack Edge in key, labels and properties.
  pub fn unpack(self) -> (Key, Vec<String>, value::ValueMap)
  {
    (self.key, self.labels, self.properties)
  }
}

impl std::fmt::Display for Edge
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "[:{} ", self.labels.join(":"))?;
    write!(f, "{}", self.properties.borrow())?;
    write!(f, "])")
  }
}

impl Into<Path> for Edge
{
  fn into(self) -> Path
  {
    Path {
      key: self.key,
      source: self.source,
      destination: self.destination,
      labels: self.labels,
      properties: self.properties,
    }
  }
}

/// Path in the graph.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "path")]
pub struct Path
{
  /// uuid for the path.
  pub(crate) key: Key,
  /// source node for the path.
  pub(crate) source: Node,
  /// destination node for the path.
  pub(crate) destination: Node,
  /// Labels for the path.
  pub(crate) labels: Vec<String>,
  /// Properties for the path.
  pub(crate) properties: value::ValueMap,
}

impl Path
{
  /// uuid for the Node.
  pub fn key(&self) -> Key
  {
    self.key
  }
  /// uuid for the Node.
  pub fn source(&self) -> &Node
  {
    &self.source
  }
  /// uuid for the Node.
  pub fn destination(&self) -> &Node
  {
    &self.destination
  }
  /// Vector of labels.
  pub fn labels(&self) -> &Vec<String>
  {
    &self.labels
  }
  /// Properties.
  pub fn properties(&self) -> &value::ValueMap
  {
    &self.properties
  }
  /// Unpack Node in key, labels and properties.
  pub fn unpack(self) -> (Key, Node, Vec<String>, value::ValueMap, Node)
  {
    (
      self.key,
      self.source,
      self.labels,
      self.properties,
      self.destination,
    )
  }
}

impl std::fmt::Display for Path
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{}-[:{} ", self.source, self.labels.join(":"))?;
    write!(f, "{}", self.properties.borrow())?;
    write!(f, "])->{}", self.destination)
  }
}

#[cfg(test)]
macro_rules! labels {
  // match a list of expressions separated by comma:
  ($($str:expr),*) => (
    {
    // create a Vec with this list of expressions,
    // calling String::from on each:
    vec![$(String::from($str),)*] as Vec<String>
    }
  );
}

#[cfg(test)]
pub(crate) use labels;
