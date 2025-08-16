use crate::prelude::*;

use super::{ExpressionType, FResult, FunctionTypeTrait};

#[derive(Debug, Default)]
pub(super) struct Coalesce {}

impl super::FunctionTrait for Coalesce
{
  fn call(&self, arguments: Vec<value::Value>) -> crate::Result<value::Value>
  {
    for arg in arguments
    {
      match arg
      {
        value::Value::Null =>
        {}
        other => return Ok(other),
      }
    }
    Ok(value::Value::Null)
  }
  fn validate_arguments(&self, _: Vec<ExpressionType>) -> crate::Result<ExpressionType>
  {
    Ok(ExpressionType::Variant)
  }
  fn is_deterministic(&self) -> bool
  {
    true
  }
}

super::declare_function!(coalesce, Coalesce, custom_trait);

#[derive(Debug, Default)]
pub(super) struct ToInteger {}

impl ToInteger
{
  fn call_impl(value: &value::Value) -> FResult<i64>
  {
    match value
    {
      value::Value::Integer(i) => Ok(*i),
      value::Value::Float(f) => Ok(*f as i64),
      value::Value::String(s) => Ok(s.parse().map_err(|_| RunTimeError::InvalidArgument {
        function_name: "toInteger",
        index: 0,
        expected_type: "A string convertible to integer",
        value: format!("{:?}", value),
      })?),
      _ => Err(RunTimeError::InvalidArgument {
        function_name: "toInteger",
        index: 0,
        expected_type: "integer, float, or string",
        value: format!("{:?}", value),
      }),
    }
  }
}

super::declare_function!(tointeger, ToInteger, call_impl(crate::value::Value) -> i64);

#[derive(Debug, Default)]
pub(super) struct Properties {}

impl Properties
{
  fn call_impl(value: &value::Value) -> FResult<value::ValueMap>
  {
    match value
    {
      value::Value::Node(n) => Ok(n.properties().to_owned()),
      value::Value::Edge(e) => Ok(e.properties().to_owned()),
      value::Value::Map(m) => Ok(m.to_owned()),
      _ => Err(RunTimeError::InvalidArgument {
        function_name: "properties",
        index: 0,
        expected_type: "node or relationship",
        value: format!("{:?}", value),
      }),
    }
  }
}

super::declare_function!(properties, Properties, call_impl(crate::value::Value) -> value::ValueMap, validate_args(ExpressionType::Map | ExpressionType::Node | ExpressionType::Edge | ExpressionType::Null));
