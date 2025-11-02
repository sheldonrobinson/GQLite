use crate::{Element, ElementType, Node};

/// GenericNode, avoid using in Rust code, but can be convenient for scripting bindings.
pub struct GenericNode
{
  key: graphcore::Key,
  query_interface: Box<dyn crate::QueryInterface>,
  graph_name: String,
  labels: Vec<String>,
}

impl GenericNode
{
  /// Create a new genric node from a given key.
  pub fn new(
    key: graphcore::Key,
    query_interface: Box<dyn crate::QueryInterface>,
    graph_name: String,
    labels: Vec<String>,
  ) -> Self
  {
    Self {
      key,
      query_interface,
      graph_name,
      labels,
    }
  }
  /// Unpack and consume
  pub fn unpack(
    self,
  ) -> (
    graphcore::Key,
    Box<dyn crate::QueryInterface>,
    String,
    Vec<String>,
  )
  {
    (self.key, self.query_interface, self.graph_name, self.labels)
  }
}

impl Element for GenericNode
{
  fn element_key(&self) -> graphcore::Key
  {
    self.key
  }
  fn element_type(&self) -> ElementType
  {
    ElementType::Node
  }
  fn graph_name(&self) -> &String
  {
    &self.graph_name
  }
  fn query_interface(&self) -> &dyn crate::QueryInterface
  {
    use std::ops::Deref;
    self.query_interface.deref()
  }
}

impl Node for GenericNode
{
  fn from_node(
    node: graphcore::Node,
    query_interface: Box<dyn crate::QueryInterface>,
    graph_name: impl Into<String>,
  ) -> Result<Self, anyhow::Error>
  {
    let (key, labels, _) = node.unpack();
    Ok(Self {
      key,
      query_interface,
      graph_name: graph_name.into(),
      labels,
    })
  }
  fn into_generic_node(self) -> GenericNode
  {
    self
  }
  fn labels(node: Option<&Self>) -> Vec<String>
  {
    node.map_or_else(Default::default, |n| n.labels.clone())
  }
}

impl Clone for GenericNode
{
  fn clone(&self) -> Self
  {
    Self {
      key: self.key,
      query_interface: self.query_interface.clone_interface(),
      graph_name: self.graph_name.clone(),
      labels: self.labels.clone(),
    }
  }
}
