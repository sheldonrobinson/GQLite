use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::prelude::*;

mod compare;
mod contains;

pub(crate) use compare::{compare, Ordering};
pub(crate) use contains::{contains, ContainResult};

/// Represent a value in a properties for a Node or an Edge.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value
{
  #[default]
  Null,
  Boolean(bool),
  Integer(i64),
  #[serde(
    serialize_with = "serialize_with::serialize_float",
    deserialize_with = "serialize_with::deserialize_float"
  )]
  Float(f64),
  String(String),
  Array(Vec<Value>),
  Map(ValueMap),
  Node(graph::Node),
  Edge(graph::Edge),
  Path(graph::Path),
}

pub type ValueMap = std::collections::HashMap<String, Value>;

pub(crate) fn value_object_display(
  obj: &ValueMap,
  f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result
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

pub(crate) trait ValueMapExtension
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
    value: ValueMap,
  ) -> crate::Result<()>;
  fn set_value<'a>(
    &mut self,
    field: Option<&'a String>,
    path: impl Iterator<Item = &'a String>,
    value: Value,
  ) -> crate::Result<()>;
}

impl ValueMapExtension for ValueMap
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
          Some(Value::Map(o)) =>
          {
            o.remove_value(Some(next_field), path)?;
          }
          None =>
          {}
          _ => Err(InternalError::Unimplemented(
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
      Err(InternalError::Unimplemented(
        "remove_value should get a better error",
      ))? // TODO
    }
    Ok(())
  }
  fn add_values<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
    value: ValueMap,
  ) -> crate::Result<()>
  {
    if let Some(field) = field
    {
      let v = self.get_mut(field);

      if let Some(next_field) = path.next()
      {
        match v
        {
          Some(Value::Map(o)) =>
          {
            o.add_values(Some(next_field), path, value)?;
          }
          None =>
          {
            let mut o = ValueMap::new();
            o.set_value(Some(next_field), path, value.remove_null().into())?;
            self.insert(field.to_owned(), o.into());
          }
          _ => Err(InternalError::Unimplemented(
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
              Value::Map(object) =>
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
              _ => Err(InternalError::Unimplemented(
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
          Some(Value::Map(o)) =>
          {
            o.set_value(Some(next_field), path, value)?;
          }
          None =>
          {
            if !value.is_null()
            {
              let mut o = ValueMap::new();
              o.set_value(Some(next_field), path, value)?;
              self.insert(field.to_owned(), o.into());
            }
          }
          _ => Err(InternalError::Unimplemented(
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
        Value::Map(o) =>
        {
          *self = o;
          Ok(())
        }
        _ => Err(InternalError::Unimplemented("set_value should get a better error").into()), // TODO
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
      Value::Map(o) => value_object_display(o, f),
      Value::Node(n) => write!(f, "{}", n),
      Value::Edge(e) => write!(f, "{}", e),
      Value::Path(p) => write!(f, "{}", p),
    }
  }
}

pub(crate) trait ValueTryIntoRef<T>
{
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

#[cfg(test)]
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
