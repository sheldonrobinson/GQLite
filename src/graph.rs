use serde::{Serialize, Deserialize};

/// Represent a value in a properties for a Node or an Edge.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub enum Value
{
  #[default]
  Invalid,
  Integer(i64),
  Float(f64),
  String(String),
  Array(Vec<Value>),
  Object(ValueObject),
  Node(Node),
  Edge(Edge),
}

pub type ValueObject = std::collections::HashMap<String, Value>;

impl Value
{
  /// Return an object from the value, or an empty object
  pub fn to_object_safe(&self) -> ValueObject
  {
    match self {
      Value::Object(o) => o.clone(),
      _ => ValueObject::new()
    }
  }
}

pub trait ToValue
{
  fn to_value(&self) -> Value;
}

macro_rules! impl_to_value
{
  ($type:tt, $vn:tt) => (
    impl ToValue for $type
    {
      fn to_value(&self) -> Value
      {
        Value::$vn(self.clone())
      }
    }
  )
}

impl_to_value!{i64, Integer}
impl_to_value!{f64, Float}
impl_to_value!{String, String}
impl_to_value!{Node, Node}
impl_to_value!{Edge, Edge}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Key
{
  uuid: u128,
}

impl Default for Key
{
  fn default() -> Self {
    Key {
      uuid: uuid::Uuid::new_v4().as_u128(),
    }
  }
}

impl From<&Key> for u128
{
  fn from(value: &Key) -> Self {
    value.uuid
  }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Node
{
  pub key: Key,
  pub labels: Vec<String>,
  pub properties: ValueObject,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Edge
{
  pub source: Key,
  pub destination: Key,
  pub label: String,
  pub properties: ValueObject,
}

#[macro_export]
macro_rules! properties {
  // map-like
  ($($k:expr => $v:expr),* $(,)?) => {{
    core::convert::From::from([$(($k.to_string(), $v.to_value()),)*])
  }};
}

#[macro_export]
macro_rules! labels {
  // match a list of expressions separated by comma:
  ($($str:expr),*) => ({
    // create a Vec with this list of expressions,
    // calling String::from on each:
    vec![$(String::from($str),)*] as Vec<String>
  });
}
