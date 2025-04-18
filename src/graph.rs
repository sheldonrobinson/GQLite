use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

/// Represent a value in a properties for a Node or an Edge.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value
{
  #[default]
  Invalid,
  Boolean(bool),
  Integer(i64),
  Float(f64),
  String(String),
  Array(Vec<Value>),
  Object(ValueObject),
  Node(Node),
  Edge(Edge),
}

pub type ValueObject = std::collections::HashMap<String, Value>;

fn value_object_display(obj: &ValueObject, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
{
  write!(f, "{{")?;
  obj.iter().enumerate().for_each(|(n, (k, v))| {
    if n == 0
    {
      write!(f, "{}: {}", k, v).unwrap();
    }
    else
    {
      write!(f, ", {}: {}", k, v).unwrap();
    }
  });
  write!(f, "}}")
}

impl Value
{
  /// Return an object from the value, or an empty object
  pub fn to_object_safe(&self) -> ValueObject
  {
    match self
    {
      Value::Object(o) => o.clone(),
      _ => ValueObject::new(),
    }
  }
  pub fn to_object(&self) -> Option<ValueObject>
  {
    match self
    {
      Value::Object(o) => Some(o.clone()),
      _ => None,
    }
  }
  pub fn to_node(&self) -> Option<Node>
  {
    match self
    {
      Value::Node(n) => Some(n.clone()),
      _ => None,
    }
  }
  pub(crate) fn access<'a>(&self, mut path: impl Iterator<Item = &'a String>) -> Value
  {
    match path.next()
    {
      Some(name) => match self
      {
        Value::Node(node) => match node.properties.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Invalid,
        },
        Value::Edge(edge) => match edge.properties.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Invalid,
        },
        Value::Object(obj) => match obj.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Invalid,
        },
        _ => Value::Invalid,
      },
      None => self.to_owned(),
    }
  }
}

impl std::fmt::Display for Value
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      Value::Invalid => write!(f, "null"),
      Value::Boolean(b) => write!(f, "{}", b),
      Value::Integer(i) => write!(f, "{}", i),
      Value::Float(fl) => write!(f, "{}", fl),
      Value::String(s) => write!(f, "{}", s),
      Value::Array(v) => write!(
        f,
        "[{}]",
        v.iter()
          .map(|x| x.to_string())
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Value::Object(o) => value_object_display(o, f),
      Value::Node(n) => write!(f, "{}", n),
      Value::Edge(e) => write!(f, "{}", e),
    }
  }
}

macro_rules! impl_to_value {
  ($type:tt, $vn:tt) => {
    impl Into<Value> for $type
    {
      fn into(self) -> Value
      {
        Value::$vn(self.clone())
      }
    }
  };
}

impl_to_value!(bool, Boolean);
impl_to_value!(i64, Integer);
impl_to_value!(f64, Float);
impl_to_value!(String, String);
impl_to_value!(Node, Node);
impl_to_value!(Edge, Edge);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Key
{
  uuid: u128,
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(tag = "type", rename = "node")]
pub struct Node
{
  pub key: Key,
  pub labels: Vec<String>,
  pub properties: ValueObject,
}

impl std::fmt::Display for Node
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "({} ", self.labels.join(":"))?;
    value_object_display(self.properties.borrow(), f)?;
    write!(f, ")")
  }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(tag = "type", rename = "edge")]
pub struct Edge
{
  pub key: Key,
  pub source: Node,
  pub destination: Node,
  pub label: String,
  pub properties: ValueObject,
}

impl std::fmt::Display for Edge
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{}-[:{} ", self.source, self.label)?;
    value_object_display(self.properties.borrow(), f)?;
    write!(f, "]->{})", self.destination)
  }
}

#[macro_export]
macro_rules! properties {
  // map-like
  ($($k:expr => $v:expr),* $(,)?) => {
    {
    core::convert::From::from([$(($k.to_string(), $v.into()),)*])
    }
  };
}

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
