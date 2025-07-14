use serde::{Deserialize, Serialize};
use std::{
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

use crate::prelude::*;

mod compare;
mod contains;
mod value_map;

pub(crate) use compare::{compare, Ordering};
pub(crate) use contains::{contains, ContainResult};
pub use value_map::ValueMap;

/// Represent a value in a properties for a Node or an Edge.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value
{
  /// Null value.
  #[default]
  Null,
  /// Boolean value.
  Boolean(bool),
  /// Signed integer value.
  Integer(i64),
  #[serde(
    serialize_with = "serialize_with::serialize_float",
    deserialize_with = "serialize_with::deserialize_float"
  )]
  /// Floating point value.
  Float(f64),
  /// String value.
  String(String),
  /// Array of values.
  Array(Vec<Value>),
  /// Unordered map of values.
  Map(ValueMap),
  /// A node in the graph.
  Node(graph::Node),
  /// An edge in the graph.
  Edge(graph::Edge),
  /// A path in the graph.
  Path(graph::Path),
}

impl Value
{
  pub(crate) fn is_null(&self) -> bool
  {
    match self
    {
      Value::Null => true,
      _ => false,
    }
  }
  pub(crate) fn remove_null(self) -> Self
  {
    match self
    {
      Value::Map(object) => object.remove_null().into(),
      o => o,
    }
  }
  /// Transform this value into a map. This function is guaranteed to succeed,
  /// in case the value does not contains a map, it will create a default empty
  /// map.
  pub fn into_map(self) -> ValueMap
  {
    match self
    {
      Value::Map(o) => o.clone(),
      _ => ValueMap::new(),
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
          None => Value::Null,
        },
        Value::Edge(edge) => match edge.properties.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Null,
        },
        Value::Map(obj) => match obj.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Null,
        },
        _ => Value::Null,
      },
      None => self.to_owned(),
    }
  }
  pub(crate) fn compare(&self, rhs: &Value) -> crate::value::Ordering
  {
    crate::value::compare(self, rhs)
  }

  fn orderability_map(lhs: &ValueMap, rhs: &ValueMap) -> std::cmp::Ordering
  {
    let o = lhs.len().cmp(&rhs.len());
    match o
    {
      std::cmp::Ordering::Equal => lhs
        .iter()
        .map(|(key, value)| value.orderability(rhs.get(key).unwrap_or(&Value::Null)))
        .find(|p| *p != std::cmp::Ordering::Equal)
        .unwrap_or(std::cmp::Ordering::Equal),
      o => o,
    }
  }
  fn orderability_float(lhs: &f64, rhs: &f64) -> std::cmp::Ordering
  {
    if lhs.is_nan()
    {
      if rhs.is_nan()
      {
        std::cmp::Ordering::Equal
      }
      else
      {
        std::cmp::Ordering::Greater
      }
    }
    else if rhs.is_nan()
    {
      std::cmp::Ordering::Less
    }
    else
    {
      lhs.total_cmp(rhs)
    }
  }
  /// Compute the order between self and rhs, for OrderBy, according to the OpenCypher specification.
  /// This order is total.
  pub(crate) fn orderability(&self, rhs: &Value) -> std::cmp::Ordering
  {
    match self
    {
      Value::Null => match rhs
      {
        Value::Null => std::cmp::Ordering::Equal,
        _ => std::cmp::Ordering::Greater,
      },
      Value::Integer(lhs) => match rhs
      {
        Value::Null => std::cmp::Ordering::Less,
        Value::Integer(rhs) => lhs.cmp(rhs),
        Value::Float(rhs) => Self::orderability_float(&(*lhs as f64), rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Float(lhs) => match rhs
      {
        Value::Null => std::cmp::Ordering::Less,
        Value::Integer(rhs) => Self::orderability_float(lhs, &(*rhs as f64)),
        Value::Float(rhs) => Self::orderability_float(lhs, rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Boolean(lhs) => match rhs
      {
        Value::Null | Value::Integer(..) | Value::Float(..) => std::cmp::Ordering::Less,
        Value::Boolean(rhs) => lhs.cmp(rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::String(lhs) => match rhs
      {
        Value::Null | Value::Integer(..) | Value::Float(..) | Value::Boolean(..) =>
        {
          std::cmp::Ordering::Less
        }
        Value::String(rhs) => lhs.cmp(rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Path(lhs) => match rhs
      {
        Value::Null
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..) => std::cmp::Ordering::Less,
        Value::Path(rhs) =>
        {
          match Self::orderability_map(&lhs.source.properties, &rhs.source.properties)
          {
            std::cmp::Ordering::Equal =>
            {
              match Self::orderability_map(&lhs.properties, &rhs.properties)
              {
                std::cmp::Ordering::Equal =>
                {
                  Self::orderability_map(&lhs.destination.properties, &rhs.destination.properties)
                }
                o => o,
              }
            }
            o => o,
          }
        }
        _ => std::cmp::Ordering::Greater,
      },
      Value::Array(lhs) => match rhs
      {
        Value::Null
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..) => std::cmp::Ordering::Less,
        Value::Array(rhs) => lhs
          .iter()
          .zip(rhs.iter())
          .map(|(lhs, rhs)| Self::orderability(lhs, rhs))
          .find(|p| *p != std::cmp::Ordering::Equal)
          .unwrap_or(lhs.len().cmp(&rhs.len())),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Edge(lhs) => match rhs
      {
        Value::Null
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..) => std::cmp::Ordering::Less,
        Value::Edge(rhs) => Self::orderability_map(&lhs.properties, &rhs.properties),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Node(lhs) => match rhs
      {
        Value::Null
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..)
        | Value::Edge(..) => std::cmp::Ordering::Less,
        Value::Node(rhs) => Self::orderability_map(&lhs.properties, &rhs.properties),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Map(lhs) => match rhs
      {
        Value::Null
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..)
        | Value::Edge(..)
        | Value::Node(..) => std::cmp::Ordering::Less,
        Value::Map(rhs) => Self::orderability_map(lhs, rhs),
      },
    }
  }
}

impl Hash for Value
{
  fn hash<H: std::hash::Hasher>(&self, state: &mut H)
  {
    match self
    {
      Value::Null =>
      {}
      Value::Boolean(b) => b.hash(state),
      Value::Integer(i) => i.hash(state),
      Value::Float(f) =>
      {
        let bits = if f.is_nan()
        {
          0x7ff8000000000000
        }
        else
        {
          f.to_bits()
        };
        bits.hash(state);
      }
      Value::String(s) => s.hash(state),
      Value::Array(a) => a.hash(state),
      Value::Map(m) => m.hash(state),
      Value::Node(n) => n.hash(state),
      Value::Edge(e) => e.hash(state),
      Value::Path(p) => p.hash(state),
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
      Value::Boolean(..) | Value::Node(..) | Value::Edge(..) | Value::Map(..) | Value::Path(..) =>
      {
        Err(RunTimeError::InvalidBinaryOperands.into())
      }
      Value::Null => Ok(Value::Null),
      Self::Array(lhs) => match rhs
      {
        Self::Array(rhs) =>
        {
          let mut lhs = lhs.clone();
          lhs.append(&mut rhs.clone());
          Ok(lhs.into())
        }
        _ =>
        {
          let mut lhs = lhs.clone();
          lhs.push(rhs.clone());
          Ok(lhs.into())
        }
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
          Value::Boolean(..)
          | Value::String(..)
          | Value::Node(..)
          | Value::Edge(..)
          | Value::Array(..)
          | Value::Map(..)
          | Value::Path(..) => Err(RunTimeError::InvalidBinaryOperands.into()),
          Value::Null => Ok(Value::Null),
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
      Value::Null => Ok(Value::Null),
      Value::Boolean(..)
      | Value::String(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::Map(..)
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
      Value::Null => write!(f, "null"),
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
      Value::Map(o) => write!(f, "{}", o),
      Value::Node(n) => write!(f, "{}", n),
      Value::Edge(e) => write!(f, "{}", e),
      Value::Path(p) => write!(f, "{}", p),
    }
  }
}

/// Trait to return a reference to the underlying type
pub(crate) trait ValueTryIntoRef<T>
{
  /// Return a reference to T
  fn try_into_ref<'a>(&'a self) -> Result<&'a T, Error>;
}

impl ValueTryIntoRef<Value> for Value
{
  fn try_into_ref<'a>(&'a self) -> Result<&'a Value, Error>
  {
    Ok(self)
  }
}

macro_rules! impl_to_value {
  ($type:ty, $vn:tt) => {
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
      type Error = ErrorType;
      fn try_into(self) -> Result<$type, Self::Error>
      {
        match self
        {
          Value::$vn(v) => Ok(v),
          _ => Err(
            InternalError::InvalidValueCast {
              value: self,
              typename: stringify!($type),
            }
            .into(),
          ),
        }
      }
    }

    impl ValueTryIntoRef<$type> for Value
    {
      fn try_into_ref<'a>(&'a self) -> Result<&'a $type, Error>
      {
        match self
        {
          Value::$vn(v) => Ok(v),
          _ => Err(
            InternalError::InvalidValueCast {
              value: self.clone(),
              typename: stringify!($type),
            }
            .into(),
          ),
        }
      }
    }
  };
}

impl_to_value!(bool, Boolean);
impl_to_value!(i64, Integer);
impl_to_value!(f64, Float);
impl_to_value!(String, String);
impl_to_value!(graph::Node, Node);
impl_to_value!(graph::Edge, Edge);
impl_to_value!(Vec<Value>, Array);
impl_to_value!(ValueMap, Map);

impl Into<Value> for &str
{
  fn into(self) -> Value
  {
    Value::String(self.into())
  }
}

#[cfg(test)]
macro_rules! array {
  () => (
      $crate::value::Value::Array(Default::default())
  );
  ($($x:expr),+ $(,)?) => (
    $crate::value::Value::Array(
      vec![$($x.into()),+]
    )
  );
}

#[cfg(test)]
pub(crate) use array;

/// Convenient macro for creating ValueMap.
///
/// Example:
///
/// ```rust
/// # use gqlitedb::{ValueMap, map};
/// let value_map: ValueMap = map!("hello" => 12);
/// ```
#[macro_export]
macro_rules! map {
  // map-like
  ($($k:expr => $v:expr),* $(,)?) => {
    {
    core::convert::From::from([$(($k.to_string(), $v.into()),)*])
    }
  };
}

#[cfg(test)]
pub(crate) use map;
