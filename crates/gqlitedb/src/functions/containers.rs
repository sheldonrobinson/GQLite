use super::{ExpressionType, FResult, FunctionTypeTrait};
use crate::{error::RunTimeError, graph};

#[derive(Debug, Default)]
pub(super) struct Keys {}

impl Keys
{
  fn call_impl(container: &graph::Value) -> crate::Result<Vec<crate::graph::Value>>
  {
    match container
    {
      graph::Value::Object(obj) => Ok(obj.keys().map(|x| x.to_owned().into()).collect()),
      graph::Value::Node(n) => Ok(n.properties.keys().map(|x| x.to_owned().into()).collect()),
      graph::Value::Edge(e) => Ok(e.properties.keys().map(|x| x.to_owned().into()).collect()),
      _ =>
      {
        return Err(
          RunTimeError::InvalidArgument {
            function_name: "keys",
            index: 0,
            expected_type: "map, node or relationship",
            value: format!("{:?}", container),
          }
          .into(),
        )
      }
    }
  }
}

super::declare_function!(keys, Keys, call_impl(crate::graph::Value) -> Vec<crate::graph::Value>, validate_args(ExpressionType::Map | ExpressionType::Node | ExpressionType::Edge | ExpressionType::Null));

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
  fn call(&self, arguments: Vec<graph::Value>) -> crate::Result<graph::Value>
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
      graph::Value::Invalid => Ok(graph::Value::Invalid),
      graph::Value::Array(arr) => Ok((arr.len() as i64).into()),
      graph::Value::Object(obj) => Ok((obj.len() as i64).into()),
      graph::Value::Path(..) => Ok(1.into()),
      _ =>
      {
        return Err(
          RunTimeError::InvalidArgument {
            function_name: "size",
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

super::declare_function!(size, Size, custom_trait);
