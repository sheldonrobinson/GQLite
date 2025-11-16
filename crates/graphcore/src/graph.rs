use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

use crate::prelude::*;

/// Uuid of a graph element (node, edge...).
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct Key
{
  pub(crate) uuid: u128,
}

impl Key
{
  /// New key from a given uuid
  pub fn new(uuid: u128) -> Self
  {
    Self { uuid }
  }
  /// Return the 128bits uuid value.
  pub fn uuid(&self) -> u128
  {
    self.uuid
  }
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
  /// Create a new node object
  pub fn new(key: Key, labels: Vec<String>, properties: value::ValueMap) -> Node
  {
    Self {
      key,
      labels,
      properties,
    }
  }
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
  /// Mutable labels
  pub fn labels_mut(&mut self) -> &mut Vec<String>
  {
    &mut self.labels
  }
  /// Edit labels
  pub fn labels_edit(&mut self, editor: impl FnOnce(Vec<String>) -> Vec<String>)
  {
    let mut tmp = Default::default();
    std::mem::swap(&mut self.labels, &mut tmp);
    self.labels = editor(tmp);
  }
  /// Properties.
  pub fn properties(&self) -> &value::ValueMap
  {
    &self.properties
  }
  /// Properties.
  pub fn take_properties(self) -> value::ValueMap
  {
    self.properties
  }
  /// Properties.
  pub fn properties_mut(&mut self) -> &mut value::ValueMap
  {
    &mut self.properties
  }
  /// Unpack Node in key, labels and properties.
  pub fn unpack(self) -> (Key, Vec<String>, value::ValueMap)
  {
    (self.key, self.labels, self.properties)
  }
  /// Convert into value map representation
  pub fn into_value_map(self) -> value::ValueMap
  {
    crate::value_map!("key" => self.key, "labels" => self.labels, "properties" => self.properties, "type" => "node")
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
  /// Labels for the Edge.
  pub(crate) labels: Vec<String>,
  /// Properties for the Edge.
  pub(crate) properties: value::ValueMap,
}

impl Edge
{
  /// Create a new node object
  pub fn new(key: Key, labels: Vec<String>, properties: value::ValueMap) -> Edge
  {
    Self {
      key,
      labels,
      properties,
    }
  }

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
  /// Mutable labels
  pub fn labels_mut(&mut self) -> &mut Vec<String>
  {
    &mut self.labels
  }
  /// Edit labels
  pub fn labels_edit(&mut self, editor: impl FnOnce(Vec<String>) -> Vec<String>)
  {
    let mut tmp = Default::default();
    std::mem::swap(&mut self.labels, &mut tmp);
    self.labels = editor(tmp);
  }
  /// Properties.
  pub fn properties(&self) -> &value::ValueMap
  {
    &self.properties
  }
  /// Properties.
  pub fn properties_mut(&mut self) -> &mut value::ValueMap
  {
    &mut self.properties
  }
  /// Properties.
  pub fn take_properties(self) -> value::ValueMap
  {
    self.properties
  }
  /// Unpack Edge in key, labels and properties.
  pub fn unpack(self) -> (Key, Vec<String>, value::ValueMap)
  {
    (self.key, self.labels, self.properties)
  }
  /// Convert into value map representation
  pub fn into_value_map(self) -> value::ValueMap
  {
    crate::value_map!("key" => self.key,  "labels" => self.labels, "properties" => self.properties, "type" => "edge")
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

/// SinglePath in the graph. A SinglePath contains an edge, source and destination.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "path")]
pub struct SinglePath
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

impl SinglePath
{
  /// Create new single path
  pub fn new(
    key: Key,
    source: Node,
    labels: Vec<String>,
    properties: value::ValueMap,
    destination: Node,
  ) -> SinglePath
  {
    SinglePath {
      key,
      source,
      destination,
      labels,
      properties,
    }
  }
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
  /// Convert into an Edge
  pub fn to_edge(&self) -> Edge
  {
    Edge {
      key: self.key,
      labels: self.labels.clone(),
      properties: self.properties.clone(),
    }
  }
  /// Convert into an Edge
  pub fn into_edge(self) -> Edge
  {
    Edge {
      key: self.key,
      labels: self.labels,
      properties: self.properties,
    }
  }
  /// Convert into value map representation
  pub fn into_value_map(self) -> value::ValueMap
  {
    crate::value_map!("key" => self.key, "source" => self.source, "labels" => self.labels, "properties" => self.properties, "destination" => self.destination, "type" => "path")
  }
}

impl std::fmt::Display for SinglePath
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{}-[:{} ", self.source, self.labels.join(":"))?;
    write!(f, "{}", self.properties.borrow())?;
    write!(f, "])->{}", self.destination)
  }
}

impl From<SinglePath> for Edge
{
  fn from(val: SinglePath) -> Self
  {
    Edge {
      key: val.key,
      labels: val.labels,
      properties: val.properties,
    }
  }
}

/// Convenient macro to create a vector of label, from &str.
#[macro_export]
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
