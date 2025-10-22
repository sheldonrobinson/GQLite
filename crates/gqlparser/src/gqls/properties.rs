//! Module with properties definitions

use std::collections::HashMap;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
      LiteralBaseType::Float => 2 << 1,
      LiteralBaseType::String => 3 << 1,
      LiteralBaseType::TimeStamp => 4 << 1,
    })
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
  Map(HashMap<String, Property>),
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
