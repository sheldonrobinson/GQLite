use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use std::{
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

mod value_map;

pub(crate) use crate::prelude::*;

pub use value_map::ValueMap;

/// Represent a value in a properties for a Node or an Edge.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value
{
  /// Null value.
  #[default]
  Null,
  /// A UUID Key in the graph.
  Key(graph::Key),
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
  Path(graph::SinglePath),
}

impl Value
{
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
  /// Return true if the value is null, false otherwise.
  pub fn is_null(&self) -> bool
  {
    matches!(self, Value::Null)
  }
  /// Remove all elements of a map that are null. Walk through the map values recursively.
  pub fn remove_null(self) -> Self
  {
    match self
    {
      Value::Map(object) => object.remove_null().into(),
      o => o,
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
      Value::Key(k) => k.hash(state),
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
  type Output = Result<Value>;
  fn add(self, rhs: Self) -> Self::Output
  {
    match self
    {
      Value::Boolean(..)
      | Value::Key(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Map(..)
      | Value::Path(..) => Err(Error::InvalidBinaryOperands),
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
        Self::Null => Ok(Self::Null),
        _ => Err(Error::InvalidBinaryOperands),
      },
      Self::Integer(lhs) => match rhs
      {
        Self::Float(rhs) => Ok((lhs as f64 + rhs).into()),
        Self::Integer(rhs) => Ok((lhs + rhs).into()),
        Self::Null => Ok(Self::Null),
        _ => Err(Error::InvalidBinaryOperands),
      },
      Self::String(lhs) => match rhs
      {
        Self::String(rhs) => Ok((lhs + &rhs).into()),
        Self::Null => Ok(Self::Null),
        _ => Err(Error::InvalidBinaryOperands),
      },
    }
  }
}

macro_rules! impl_mdsr {
  ($x:tt, $op:tt) => {
    impl $x for Value
    {
      type Output = Result<Value>;
      fn $op(self, rhs: Self) -> Self::Output
      {
        match self
        {
          Value::Boolean(..)
          | Value::Key(..)
          | Value::String(..)
          | Value::Node(..)
          | Value::Edge(..)
          | Value::Array(..)
          | Value::Map(..)
          | Value::Path(..) => Err(Error::InvalidBinaryOperands.into()),
          Value::Null => Ok(Value::Null),
          Self::Float(lhs) => match rhs
          {
            Self::Float(rhs) => Ok(lhs.$op(rhs).into()),
            Self::Integer(rhs) => Ok(lhs.$op(rhs as f64).into()),
            Self::Null => Ok(Self::Null),
            _ => Err(Error::InvalidBinaryOperands.into()),
          },
          Self::Integer(lhs) => match rhs
          {
            Self::Float(rhs) => Ok((lhs as f64).$op(rhs).into()),
            Self::Integer(rhs) => Ok(lhs.$op(rhs).into()),
            Self::Null => Ok(Self::Null),
            _ => Err(Error::InvalidBinaryOperands.into()),
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

impl Value
{
  /// Compute this value to the power of rhs. Return an error if called a non-number values.
  pub fn pow(self, rhs: Value) -> Result<Value>
  {
    match self
    {
      Value::Boolean(..)
      | Value::Key(..)
      | Value::String(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::Map(..)
      | Value::Path(..) => Err(Error::InvalidBinaryOperands),
      Value::Null => Ok(Value::Null),
      Self::Float(lhs) => match rhs
      {
        Self::Float(rhs) => Ok(lhs.powf(rhs).into()),
        Self::Integer(rhs) => Ok(lhs.powf(rhs as f64).into()),
        Self::Null => Ok(Self::Null),
        _ => Err(Error::InvalidBinaryOperands),
      },
      Self::Integer(lhs) => match rhs
      {
        Self::Float(rhs) => Ok((lhs as f64).powf(rhs).into()),
        Self::Integer(rhs) => match rhs.try_into()
        {
          Ok(rhs) => Ok(lhs.pow(rhs).into()),
          Err(_) => Ok((lhs as f64).powf(rhs as f64).into()),
        },
        Self::Null => Ok(Self::Null),
        _ => Err(Error::InvalidBinaryOperands),
      },
    }
  }
}

impl Neg for Value
{
  type Output = Result<Value>;
  fn neg(self) -> Self::Output
  {
    match self
    {
      Self::Float(fl) => Ok((-fl).into()),
      Self::Integer(i) => Ok((-i).into()),
      Value::Null => Ok(Value::Null),
      Value::Boolean(..)
      | Value::Key(..)
      | Value::String(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::Map(..)
      | Value::Path(..) => Err(Error::InvalidNegationOperands),
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
      Value::Key(k) => write!(f, "<{}>", k.uuid()),
      Value::Boolean(b) => write!(f, "{}", b),
      Value::Integer(i) => write!(f, "{}", i),
      Value::Float(fl) => write!(f, "{}", fl),
      Value::String(s) => write!(f, "{}", s),
      Value::Array(v) => write!(f, "[{}]", v.iter().map(|x| x.to_string()).join(", ")),
      Value::Map(o) => write!(f, "{}", o),
      Value::Node(n) => write!(f, "{}", n),
      Value::Edge(e) => write!(f, "{}", e),
      Value::Path(p) => write!(f, "{}", p),
    }
  }
}

/// Trait to return a reference to the underlying type
pub trait ValueTryIntoRef<T>
{
  /// Return a reference to T
  fn try_into_ref(&self) -> Result<&T, Error>;
}

impl ValueTryIntoRef<Value> for Value
{
  fn try_into_ref(&self) -> Result<&Value, Error>
  {
    Ok(self)
  }
}

macro_rules! impl_to_value {
  ($type:ty, $vn:tt) => {
    impl From<$type> for Value
    {
      fn from(v: $type) -> Value
      {
        Value::$vn(v)
      }
    }

    impl From<Vec<$type>> for Value
    {
      fn from(v: Vec<$type>) -> Value
      {
        Value::Array(v.into_iter().map(|v| v.into()).collect())
      }
    }
    impl TryInto<$type> for Value
    {
      type Error = Error;
      fn try_into(self) -> Result<$type, Self::Error>
      {
        match self
        {
          Value::$vn(v) => Ok(v),
          _ => Err(
            Error::InvalidValueCast {
              value: Box::new(self),
              typename: stringify!($type),
            }
            .into(),
          ),
        }
      }
    }

    impl ValueTryIntoRef<$type> for Value
    {
      fn try_into_ref(&self) -> Result<&$type, Error>
      {
        match self
        {
          Value::$vn(v) => Ok(v),
          _ => Err(
            Error::InvalidValueCast {
              value: Box::new(self.clone()),
              typename: stringify!($type),
            }
            .into(),
          ),
        }
      }
    }
  };
}

impl_to_value!(graph::Key, Key);
impl_to_value!(bool, Boolean);
impl_to_value!(i64, Integer);
impl_to_value!(f64, Float);
impl_to_value!(String, String);
impl_to_value!(graph::Node, Node);
impl_to_value!(graph::Edge, Edge);
impl_to_value!(graph::SinglePath, Path);
impl_to_value!(Vec<Value>, Array);
impl_to_value!(ValueMap, Map);

impl From<&str> for Value
{
  fn from(val: &str) -> Self
  {
    Value::String(val.into())
  }
}

/// Convenient macro for creating Array.
///
/// Example:
///
/// ```rust
/// # use graphcore::{Value, array};
/// let value_arr: Value = array!("hello", 12);
/// ```
#[macro_export]
macro_rules! array {
  () => (
      $crate::Value::Array(Default::default())
  );
  ($($x:expr),+ $(,)?) => (
    $crate::Value::Array(
      vec![$($x.into()),+]
    )
  );
}

/// Convenient macro for creating ValueMap.
///
/// Example:
///
/// ```rust
/// # use graphcore::{ValueMap, value_map};
/// let value_map: ValueMap = value_map!("hello" => 12);
/// ```
#[macro_export]
macro_rules! value_map {
  // map-like
  ($($k:expr => $v:expr),* $(,)?) => {
    {
      let value_map: $crate::ValueMap = core::convert::From::from([$(($k.to_string(), $v.into()),)*]);
      value_map
    }
  };
}
