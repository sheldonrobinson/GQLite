use std::ops::Index;

use ciborium::value;

use crate::{error::RunTimeError, graph};

use super::FResult;

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

super::declare_function!(has_label, HasLabel, (crate::graph::Edge, String));
