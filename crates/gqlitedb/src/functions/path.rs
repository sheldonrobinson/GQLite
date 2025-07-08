use super::ExpressionType;
use crate::prelude::*;

#[derive(Debug, Default)]
pub(super) struct Length {}

impl super::FunctionTrait for Length
{
  fn call(&self, arguments: Vec<value::Value>) -> crate::Result<value::Value>
  {
    let container = arguments
      .first()
      .ok_or_else(|| RunTimeError::InvalidNumberOfArguments {
        function_name: "length",
        got: arguments.len(),
        expected: 1,
      })?;

    match container
    {
      value::Value::Array(arr) => Ok((arr.len() as i64).into()),
      value::Value::Map(obj) => Ok((obj.len() as i64).into()),
      value::Value::Path(..) => Ok(1.into()),
      _ =>
      {
        return Err(
          RunTimeError::InvalidArgument {
            function_name: "length",
            index: 0,
            expected_type: "array or map",
            value: format!("{:?}", container),
          }
          .into(),
        )
      }
    }
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

super::declare_function!(length, Length, custom_trait);
