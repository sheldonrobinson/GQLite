use super::{ExpressionType, FResult, FunctionTypeTrait};
use crate::prelude::*;

#[derive(Debug, Default)]
pub(super) struct Head {}

impl Head
{
  #[allow(clippy::ptr_arg)]
  fn call_impl(array: &Vec<value::Value>) -> FResult<value::Value>
  {
    Ok(
      array
        .first()
        .map(|x| x.to_owned())
        .unwrap_or(value::Value::Null),
    )
  }
}

super::declare_function!(
  head,
  Head,
  call_impl(Vec<crate::value::Value>) -> crate::value::Value,
  accept_null
);

#[derive(Debug, Default)]
pub(super) struct Keys {}

impl Keys
{
  fn call_impl(container: &value::Value) -> Result<Vec<value::Value>>
  {
    match container
    {
      value::Value::Map(obj) => Ok(obj.keys().map(|x| x.to_owned().into()).collect()),
      value::Value::Node(n) => Ok(n.properties().keys().map(|x| x.to_owned().into()).collect()),
      value::Value::Edge(e) => Ok(e.properties().keys().map(|x| x.to_owned().into()).collect()),
      _ => Err(
        RunTimeError::InvalidArgument {
          function_name: "keys",
          index: 0,
          expected_type: "map, node or relationship",
          value: format!("{:?}", container),
        }
        .into(),
      ),
    }
  }
}

super::declare_function!(keys, Keys, call_impl(crate::value::Value) -> Vec<crate::value::Value>, validate_args(ExpressionType::Map | ExpressionType::Node | ExpressionType::Edge | ExpressionType::Null));

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

#[derive(Debug, Default)]
pub(super) struct Size {}

impl super::FunctionTrait for Size
{
  fn call(&self, arguments: Vec<value::Value>) -> Result<value::Value>
  {
    let container = arguments
      .first()
      .ok_or_else(|| RunTimeError::InvalidNumberOfArguments {
        function_name: "size",
        got: arguments.len(),
        expected: 1,
      })?;

    match container
    {
      value::Value::Null => Ok(value::Value::Null),
      value::Value::Array(arr) => Ok((arr.len() as i64).into()),
      value::Value::Map(obj) => Ok((obj.len() as i64).into()),
      value::Value::Path(..) => Ok(1.into()),
      _ => Err(
        RunTimeError::InvalidArgument {
          function_name: "size",
          index: 0,
          expected_type: "array or map",
          value: format!("{:?}", container),
        }
        .into(),
      ),
    }
  }
  fn validate_arguments(&self, _: Vec<ExpressionType>) -> Result<ExpressionType>
  {
    Ok(ExpressionType::Variant)
  }
  fn is_deterministic(&self) -> bool
  {
    true
  }
}

super::declare_function!(size, Size, custom_trait);
