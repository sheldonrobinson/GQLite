use crate::{error::RunTimeError, graph};

use super::{ExpressionType, FResult, FunctionTypeTrait};

#[derive(Debug, Default)]
pub(super) struct ToString {}

impl ToString
{
  fn call_impl(value: &graph::Value) -> FResult<String>
  {
    match value
    {
      graph::Value::Boolean(b) => Ok(if *b { "true" } else { "false" }.into()),
      graph::Value::Integer(i) => Ok(i.to_string()),
      graph::Value::Float(f) => Ok(f.to_string()),
      graph::Value::String(s) => Ok(s.to_owned()),
      _ => Err(RunTimeError::InvalidArgument {
        function_name: "toString",
        index: 0,
        expected_type: "boolean or integer or double",
        value: format!("{:?}", value),
      }),
    }
  }
}

super::declare_function!(toString, ToString, call_impl(crate::graph::Value) -> String, validate_args(ExpressionType::Boolean | ExpressionType::Integer | ExpressionType::Float | ExpressionType::String | ExpressionType::Null));
