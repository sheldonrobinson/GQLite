use super::ExpressionType;
use crate::{functions::FResult, prelude::*};

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
      _ => Err(
        RunTimeError::InvalidArgument {
          function_name: "length",
          index: 0,
          expected_type: "array or map",
          value: format!("{:?}", container),
        }
        .into(),
      ),
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
pub(super) struct Nodes {}

impl Nodes
{
  fn call_impl(path: &graph::Path) -> FResult<Vec<graph::Node>>
  {
    Ok(vec![path.source().clone(), path.destination().clone()])
  }
}

super::declare_function!(nodes, Nodes, call_impl(crate::graph::Path) -> Vec<graph::Node>);

#[derive(Debug, Default)]
pub(super) struct Edges {}

impl Edges
{
  fn call_impl(path: &graph::Path) -> FResult<Vec<graph::Edge>>
  {
    Ok(vec![graph::Edge::new(
      path.key(),
      path.labels().clone(),
      path.properties().clone(),
    )])
  }
}

super::declare_function!(edges, Edges, call_impl(crate::graph::Path) -> Vec<graph::Edge>);
