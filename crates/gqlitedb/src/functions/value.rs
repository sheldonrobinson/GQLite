use crate::prelude::*;

use super::{ExpressionType, FResult};

#[derive(Debug, Default)]
pub(super) struct HasLabel {}

impl HasLabel
{
  fn call_impl(value: &value::Value, label: &String) -> FResult<bool>
  {
    match value
    {
      value::Value::Edge(e) => Ok(e.labels().contains(label)),
      value::Value::Node(n) => Ok(n.labels().contains(label)),
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
  fn call(&self, arguments: Vec<value::Value>) -> crate::Result<value::Value>
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
      let mut it = arguments.iter();
      let labels = match it.next().unwrap()
      {
        value::Value::Edge(e) => e.labels(),
        value::Value::Node(n) => n.labels(),
        value::Value::Null => return Ok(value::Value::Null),
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
          value::Value::String(l) =>
          {
            if !labels.contains(l)
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
