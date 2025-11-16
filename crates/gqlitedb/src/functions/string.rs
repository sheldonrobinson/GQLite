use crate::prelude::*;

use super::{ExpressionType, FResult, FunctionTypeTrait};

#[derive(Debug, Default)]
pub(super) struct ToString {}

impl ToString
{
  fn call_impl(value: &value::Value) -> FResult<String>
  {
    match value
    {
      value::Value::Boolean(b) => Ok(if *b { "true" } else { "false" }.into()),
      value::Value::Integer(i) => Ok(i.to_string()),
      value::Value::Float(f) => Ok(f.to_string()),
      value::Value::String(s) => Ok(s.to_owned()),
      _ => Err(RunTimeError::InvalidArgument {
        function_name: "toString",
        index: 0,
        expected_type: "boolean or integer or double",
        value: format!("{:?}", value),
      }),
    }
  }
}

super::declare_function!(tostring, ToString, call_impl(crate::value::Value) -> String, validate_args(ExpressionType::Boolean | ExpressionType::Integer | ExpressionType::Float | ExpressionType::String | ExpressionType::Null));
