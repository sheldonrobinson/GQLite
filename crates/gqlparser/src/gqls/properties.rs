//! Module with properties definitions

use indexmap::IndexMap;

use crate::Error;

/// Literal types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralBaseType
{
  /// Boolean value
  Boolean,
  /// i64 value
  Integer,
  /// f64 value
  Float,
  /// String value
  String,
  /// Timestamp value
  TimeStamp,
}

/// This structure allow to map several literal types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiteralAlternativeType(u8);

impl LiteralAlternativeType
{
  /// Check if the alternative contains the given type
  pub fn has_type(&self, ltype: LiteralBaseType) -> bool
  {
    let c: LiteralAlternativeType = ltype.into();
    (self.0 & c.0) != 0
  }
}

impl From<LiteralBaseType> for LiteralAlternativeType
{
  fn from(value: LiteralBaseType) -> Self
  {
    LiteralAlternativeType(match value
    {
      LiteralBaseType::Boolean => 1,
      LiteralBaseType::Integer => 1 << 1,
      LiteralBaseType::Float => 1 << 2,
      LiteralBaseType::String => 1 << 3,
      LiteralBaseType::TimeStamp => 1 << 4,
    })
  }
}

impl TryFrom<LiteralAlternativeType> for LiteralBaseType
{
  type Error = Error;
  fn try_from(value: LiteralAlternativeType) -> Result<Self, Self::Error>
  {
    if value.0.count_ones() == 1
    {
      if value.has_type(LiteralBaseType::Boolean)
      {
        Ok(LiteralBaseType::Boolean)
      }
      else if value.has_type(LiteralBaseType::Float)
      {
        Ok(LiteralBaseType::Float)
      }
      else if value.has_type(LiteralBaseType::Integer)
      {
        Ok(LiteralBaseType::Integer)
      }
      else if value.has_type(LiteralBaseType::String)
      {
        Ok(LiteralBaseType::String)
      }
      else if value.has_type(LiteralBaseType::TimeStamp)
      {
        Ok(LiteralBaseType::TimeStamp)
      }
      else
      {
        Err(Error::InvalidType)
      }
    }
    else
    {
      Err(Error::MultiTypeCannotBeConvertedToSingle)
    }
  }
}

/// Type constraint on a property
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Property
{
  /// Can have any value
  Any,
  /// Is optional, other are assumed required, unless contained in optional
  Optional(Box<Property>),
  /// Literal value (string, float...)
  Literal(LiteralAlternativeType),
  /// A map of property
  Map(IndexMap<String, Property>),
  /// An array of property
  Array(Box<Property>),
}

impl From<LiteralBaseType> for Property
{
  fn from(value: LiteralBaseType) -> Self
  {
    Property::Literal(value.into())
  }
}
