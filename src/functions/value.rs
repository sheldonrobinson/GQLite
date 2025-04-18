use crate::{error::RunTimeError, graph};

use super::{FResult, FunctionTypeTrait};

#[derive(Debug, Default)]
pub(super) struct Coalesce {}

impl super::FunctionTrait for Coalesce
{
  fn call(&self, arguments: Vec<graph::Value>) -> crate::Result<graph::Value>
  {
    for arg in arguments
    {
      match arg
      {
        graph::Value::Invalid =>
        {}
        other => return Ok(other),
      }
    }
    Ok(graph::Value::Invalid)
  }
  fn validate_arguments(
    &self,
    _: Vec<crate::interpreter::validator::VariableType>,
  ) -> crate::Result<crate::interpreter::validator::VariableType>
  {
    Ok(crate::interpreter::validator::VariableType::Variant)
  }
}

super::declare_function!(coalesce, Coalesce, custom_trait);

#[derive(Debug, Default)]
pub(super) struct HasLabel {}

impl HasLabel
{
  fn call_impl(value: &graph::Value, label: &String) -> FResult<bool>
  {
    match value
    {
      graph::Value::Edge(e) => Ok(e.labels.contains(label)),
      graph::Value::Node(n) => Ok(n.labels.contains(label)),
      _ => Err(RunTimeError::InvalidArgument {
        function_name: "has_label",
        index: 0,
        expected_type: "node or edege",
        value: format!("{:?}", value),
      }),
    }
  }
}

super::declare_function!(has_label, HasLabel, call_impl(crate::graph::Edge, String) -> bool);
