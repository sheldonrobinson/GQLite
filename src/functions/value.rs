use crate::{error::RunTimeError, graph};

use super::{ExpressionType, FResult, FunctionTypeTrait};

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

#[derive(Debug, Default)]
pub(super) struct HasLabels {}

impl super::FunctionTrait for HasLabels
{
  fn call(&self, arguments: Vec<graph::Value>) -> crate::Result<graph::Value>
  {
    if arguments.len() < 2
    {
      Err(
        RunTimeError::InvalidNumberOfArguments {
          function_name: "has_labels",
          got: arguments.len(),
          expected: 2,
        }
        .into(),
      )
    }
    else
    {
      let mut it = arguments.into_iter();
      let labels = match it.next().unwrap()
      {
        graph::Value::Edge(e) => e.labels,
        graph::Value::Node(n) => n.labels,
        _ => Err(RunTimeError::InvalidArgument {
          function_name: "has_labels",
          index: 0,
          expected_type: "node or edge",
          value: format!("{:?}", it),
        })?,
      };
      for (index, label) in it.enumerate()
      {
        match label
        {
          graph::Value::String(l) =>
          {
            if !labels.contains(&l)
            {
              return Ok(false.into());
            }
          }
          _ =>
          {
            return Err(
              RunTimeError::InvalidArgument {
                function_name: "has_labels",
                index,
                expected_type: "string",
                value: format!("{:?}", label),
              }
              .into(),
            )
          }
        }
      }
      Ok(true.into())
    }
  }
  fn validate_arguments(&self, _: Vec<ExpressionType>) -> crate::Result<ExpressionType>
  {
    Ok(ExpressionType::Boolean)
  }
  fn is_deterministic(&self) -> bool
  {
    true
  }
}

super::declare_function!(has_labels, HasLabels, custom_trait);
