use super::{ExpressionType, FResult, FunctionTypeTrait};
use crate::{error::RunTimeError, graph};

#[derive(Debug, Default)]
pub(super) struct Length {}

impl super::FunctionTrait for Length
{
  fn call(&self, arguments: Vec<graph::Value>) -> crate::Result<graph::Value>
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
      graph::Value::Array(arr) => Ok((arr.len() as i64).into()),
      graph::Value::Object(obj) => Ok((obj.len() as i64).into()),
      graph::Value::Path(..) => Ok(1.into()),
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

#[derive(Debug, Default)]
pub(super) struct Range {}

impl Range
{
  fn call_impl(min: &i64, max: &i64) -> FResult<Vec<i64>>
  {
    Ok((*min..=*max).step_by(1).collect())
  }
}

super::declare_function!(range, Range, call_impl(i64, i64) -> Vec<i64>);
