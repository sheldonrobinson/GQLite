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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "node")]
pub struct Node
{
  pub key: Key,
  pub labels: Vec<String>,
  pub properties: value::ValueMap,
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "edge")]
pub struct Edge
{
  pub key: Key,
  #[serde(skip_serializing)]
  pub source: Node,
  #[serde(skip_serializing)]
  pub destination: Node,
  pub labels: Vec<String>,
  pub properties: value::ValueMap,
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Hash)]
#[serde(tag = "type", rename = "path")]
pub struct Path
{
  pub key: Key,
  pub source: Node,
  pub destination: Node,
  pub labels: Vec<String>,
  pub properties: value::ValueMap,
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
