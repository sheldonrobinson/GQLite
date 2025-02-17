use std::{
  borrow::Borrow,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

use serde::{Deserialize, Serialize};

use crate::error::{InternalError, RunTimeError};

#[derive(Debug, Clone, Copy)]
pub(crate) enum EdgeDirectivity
{
  Undirected,
  Directed,
}

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
  Path(Path),
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

pub(crate) trait ValueObjectExtension
{
  fn remove_null(self) -> Self;
  fn remove_value<'a>(
    &mut self,
    field: Option<&'a String>,
    path: impl Iterator<Item = &'a String>,
  ) -> crate::Result<()>;
  fn add_values<'a>(
    &mut self,
    field: Option<&'a String>,
    path: impl Iterator<Item = &'a String>,
    value: ValueObject,
  ) -> crate::Result<()>;
  fn set_value<'a>(
    &mut self,
    field: Option<&'a String>,
    path: impl Iterator<Item = &'a String>,
    value: Value,
  ) -> crate::Result<()>;
}

impl ValueObjectExtension for ValueObject
{
  fn remove_null(self) -> Self
  {
    self
      .into_iter()
      .filter(|(_, v)| !v.is_null())
      .map(|(k, v)| (k, v.remove_null()))
      .collect()
  }
  fn remove_value<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
  ) -> crate::Result<()>
  {
    if let Some(field) = field
    {
      if let Some(next_field) = path.next()
      {
        let v = self.get_mut(field);
        match v
        {
          Some(Value::Object(o)) =>
          {
            o.remove_value(Some(next_field), path)?;
          }
          None =>
          {}
          _ => Err(crate::error::Error::Unimplemented(
            "remove_value should get a better error",
          ))?, // TODO
        }
      }
      else
      {
        self.remove(field);
      }
    }
    else
    {
      Err(crate::error::Error::Unimplemented(
        "remove_value should get a better error",
      ))? // TODO
    }
    Ok(())
  }
  fn add_values<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
    value: ValueObject,
  ) -> crate::Result<()>
  {
    if let Some(field) = field
    {
      let v = self.get_mut(field);

      if let Some(next_field) = path.next()
      {
        match v
        {
          Some(Value::Object(o)) =>
          {
            o.add_values(Some(next_field), path, value)?;
          }
          None =>
          {
            let mut o = ValueObject::new();
            o.set_value(Some(next_field), path, value.remove_null().into())?;
            self.insert(field.to_owned(), o.into());
          }
          _ => Err(crate::error::Error::Unimplemented(
            "add_values should get a better error",
          ))?, // TODO
        }
      }
      else
      {
        match v
        {
          Some(v) =>
          {
            match v
            {
              Value::Object(object) =>
              {
                for (k, v) in value.into_iter()
                {
                  if v.is_null()
                  {
                    object.remove(&k);
                  }
                  else
                  {
                    object.insert(k, v);
                  }
                }
              }
              _ => Err(crate::error::Error::Unimplemented(
                "add_values should get a better error",
              ))?, // TODO
            }
          }
          None =>
          {
            self.insert(field.to_owned(), value.remove_null().into());
          }
        }
      }
    }
    else
    {
      for (k, v) in value.into_iter()
      {
        if v.is_null()
        {
          self.remove(&k);
        }
        else
        {
          self.insert(k, v);
        }
      }
    }
    Ok(())
  }
  fn set_value<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
    value: Value,
  ) -> crate::Result<()>
  {
    if let Some(field) = field
    {
      let v = self.get_mut(field);

      if let Some(next_field) = path.next()
      {
        match v
        {
          Some(Value::Object(o)) =>
          {
            o.set_value(Some(next_field), path, value)?;
          }
          None =>
          {
            if !value.is_null()
            {
              let mut o = ValueObject::new();
              o.set_value(Some(next_field), path, value)?;
              self.insert(field.to_owned(), o.into());
            }
          }
          _ => Err(crate::error::Error::Unimplemented(
            "update_value should get a better error",
          ))?, // TODO
        }
      }
      else
      {
        if value.is_null()
        {
          self.remove(field);
        }
        else
        {
          match v
          {
            Some(v) =>
            {
              *v = value;
            }
            None =>
            {
              self.insert(field.to_owned(), value);
            }
          }
        }
      }

      Ok(())
    }
    else
    {
      match value
      {
        Value::Object(o) =>
        {
          *self = o;
          Ok(())
        }
        _ => Err(crate::error::Error::Unimplemented(
          "set_value should get a better error",
        )), // TODO
      }
    }
  }
}

impl Value
{
  pub(crate) fn is_null(&self) -> bool
  {
    match self
    {
      Value::Invalid => true,
      _ => false,
    }
  }
  pub(crate) fn remove_null(self) -> Self
  {
    match self
    {
      Value::Object(object) => object.remove_null().into(),
      o => o,
    }
  }
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
  pub fn to_edge(&self) -> Option<Edge>
  {
    match self
    {
      Value::Edge(e) => Some(e.clone()),
      _ => None,
    }
  }
  pub fn to_boolean(&self) -> Option<bool>
  {
    match self
    {
      Value::Boolean(b) => Some(*b),
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
  pub(crate) fn partial_compare<E: crate::error::GenericErrors>(
    &self,
    rhs: &Self,
  ) -> crate::Result<std::cmp::Ordering>
  {
    match self
    {
      Value::Invalid
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::Object(..)
      | Value::Path(..) => Err(E::not_comparable().into()),
      Value::Boolean(lhs) => match rhs
      {
        Value::Boolean(rhs) => lhs.partial_cmp(rhs).ok_or(E::not_comparable().into()),
        _ => Err(E::not_comparable().into()),
      },
      Value::Integer(lhs) => match rhs
      {
        Value::Integer(rhs) => lhs.partial_cmp(rhs).ok_or(E::not_comparable().into()),
        Value::Float(rhs) => (*lhs as f64)
          .partial_cmp(rhs)
          .ok_or(E::not_comparable().into()),
        _ => Err(E::not_comparable().into()),
      },
      Value::Float(lhs) => match rhs
      {
        Value::Integer(rhs) => lhs
          .partial_cmp(&(*rhs as f64))
          .ok_or(E::not_comparable().into()),
        Value::Float(rhs) => lhs.partial_cmp(rhs).ok_or(E::not_comparable().into()),
        _ => Err(E::not_comparable().into()),
      },
      Value::String(lhs) => match rhs
      {
        Value::String(rhs) => lhs.partial_cmp(rhs).ok_or(E::not_comparable().into()),
        _ => Err(E::not_comparable().into()),
      },
    }
  }
}

impl Add for Value
{
  type Output = crate::Result<Value>;
  fn add(self, rhs: Self) -> Self::Output
  {
    match self
    {
      Value::Invalid
      | Value::Boolean(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Object(..)
      | Value::Path(..) => Err(RunTimeError::InvalidBinaryOperands.into()),
      Self::Array(lhs) => match rhs
      {
        Self::Array(rhs) =>
        {
          let mut lhs = lhs.clone();
          lhs.append(&mut rhs.clone());
          Ok(lhs.into())
        }
        _ => Err(RunTimeError::InvalidBinaryOperands.into()),
      },
      Self::Float(lhs) => match rhs
      {
        Self::Float(rhs) => Ok((lhs + rhs).into()),
        Self::Integer(rhs) => Ok((lhs + rhs as f64).into()),
        _ => Err(RunTimeError::InvalidBinaryOperands.into()),
      },
      Self::Integer(lhs) => match rhs
      {
        Self::Float(rhs) => Ok((lhs as f64 + rhs).into()),
        Self::Integer(rhs) => Ok((lhs + rhs).into()),
        _ => Err(RunTimeError::InvalidBinaryOperands.into()),
      },
      Self::String(lhs) => match rhs
      {
        Self::String(rhs) => Ok((lhs + &rhs).into()),
        _ => Err(RunTimeError::InvalidBinaryOperands.into()),
      },
    }
  }
}

macro_rules! impl_mdsr {
  ($x:tt, $op:tt) => {
    impl $x for Value
    {
      type Output = crate::Result<Value>;
      fn $op(self, rhs: Self) -> Self::Output
      {
        match self
        {
          Value::Invalid
          | Value::Boolean(..)
          | Value::String(..)
          | Value::Node(..)
          | Value::Edge(..)
          | Value::Array(..)
          | Value::Object(..)
          | Value::Path(..) => Err(RunTimeError::InvalidBinaryOperands.into()),
          Self::Float(lhs) => match rhs
          {
            Self::Float(rhs) => Ok(lhs.$op(rhs).into()),
            Self::Integer(rhs) => Ok(lhs.$op(rhs as f64).into()),
            _ => Err(RunTimeError::InvalidBinaryOperands.into()),
          },
          Self::Integer(lhs) => match rhs
          {
            Self::Float(rhs) => Ok((lhs as f64).$op(rhs).into()),
            Self::Integer(rhs) => Ok(lhs.$op(rhs).into()),
            _ => Err(RunTimeError::InvalidBinaryOperands.into()),
          },
        }
      }
    }
  };
}

impl_mdsr!(Mul, mul);
impl_mdsr!(Sub, sub);
impl_mdsr!(Div, div);
impl_mdsr!(Rem, rem);

impl Neg for Value
{
  type Output = crate::Result<Value>;
  fn neg(self) -> Self::Output
  {
    match self
    {
      Self::Float(fl) => Ok((-fl).into()),
      Self::Integer(i) => Ok((-i).into()),
      Value::Invalid
      | Value::Boolean(..)
      | Value::String(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::Object(..)
      | Value::Path(..) => Err(RunTimeError::InvalidNegationOperands.into()),
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
      Value::Path(p) => write!(f, "{}", p),
    }
  }
}

pub(crate) trait ValueTryIntoRef<T>
{
  fn try_into_ref<'a>(&'a self) -> Result<&'a T, crate::error::Error>;
}

impl ValueTryIntoRef<Value> for Value
{
  fn try_into_ref<'a>(&'a self) -> Result<&'a Value, crate::error::Error>
  {
    Ok(self)
  }
}

macro_rules! impl_to_value {
  ($type:ty, $vn:tt $(, null_into: $null_into:tt)?) => {
    impl Into<Value> for $type
    {
      fn into(self) -> Value
      {
        Value::$vn(self.clone())
      }
    }

    impl Into<Value> for Vec<$type>
    {
      fn into(self) -> Value
      {
        Value::Array(self.into_iter().map(|v| v.into()).collect())
      }
    }
    impl TryInto<$type> for Value
    {
      type Error = crate::error::Error;
      fn try_into(self) -> Result<$type, Self::Error>
      {
        match self
        {
          $(Value::Invalid => Ok($null_into),)?
          Value::$vn(v) => Ok(v),
          _ => Err(InternalError::InvalidValueCast.into()),
        }
      }
    }

    impl ValueTryIntoRef<$type> for Value
    {
      fn try_into_ref<'a>(&'a self) -> Result<&'a $type, crate::error::Error>
      {
        match self
        {
          Value::$vn(v) => Ok(v),
          _ => Err(crate::error::InternalError::InvalidValueCast.into()),
        }
      }
    }
  };
}

impl_to_value!(bool, Boolean, null_into: false);
impl_to_value!(i64, Integer);
impl_to_value!(f64, Float);
impl_to_value!(String, String);
impl_to_value!(Node, Node);
impl_to_value!(Edge, Edge);
impl_to_value!(Vec<Value>, Array);
impl_to_value!(ValueObject, Object);

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
    if self.labels.is_empty()
    {
      write!(f, "(")?;
    }
    else
    {
      write!(f, "(:{} ", self.labels.join(":"))?;
    }
    value_object_display(self.properties.borrow(), f)?;
    write!(f, ")")
  }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(tag = "type", rename = "edge")]
pub struct Edge
{
  pub key: Key,
  #[serde(skip_serializing)]
  pub source: Node,
  #[serde(skip_serializing)]
  pub destination: Node,
  pub labels: Vec<String>,
  pub properties: ValueObject,
}

impl std::fmt::Display for Edge
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "[:{} ", self.labels.join(":"))?;
    value_object_display(self.properties.borrow(), f)?;
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(tag = "type", rename = "path")]
pub struct Path
{
  pub key: Key,
  pub source: Node,
  pub destination: Node,
  pub labels: Vec<String>,
  pub properties: ValueObject,
}

impl std::fmt::Display for Path
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{}-[:{} ", self.source, self.labels.join(":"))?;
    value_object_display(self.properties.borrow(), f)?;
    write!(f, "])->{}", self.destination)
  }
}

#[cfg(test)]
macro_rules! properties {
  // map-like
  ($($k:expr => $v:expr),* $(,)?) => {
    {
    core::convert::From::from([$(($k.to_string(), $v.into()),)*])
    }
  };
}

#[cfg(test)]
pub(crate) use properties;

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
