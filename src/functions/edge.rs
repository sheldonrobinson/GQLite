use std::rc::Rc;

use crate::{
  error::{self, RunTimeError},
  graph,
};

use super::{Function, FunctionTrait};

type FResult<T> = std::result::Result<T, error::RunTimeError>;

#[derive(Debug, Default)]
pub(super) struct Type {}

impl Type
{
  pub(super) fn new() -> (String, Function)
  {
    ("type".to_string(), Rc::new(Box::new(Self {})))
  }
  fn call_impl(edge: &graph::Edge) -> FResult<String>
  {
    edge
      .labels
      .first()
      .ok_or_else(|| RunTimeError::MissingEdgeLabel)
      .map(|v| v.to_owned())
  }
}

impl FunctionTrait for Type
{
  fn call(&self, arguments: Vec<crate::graph::Value>) -> crate::Result<crate::graph::Value>
  {
    if arguments.len() == 1
    {
      Ok(
        Self::call_impl(
          &arguments[0]
            .to_edge()
            .ok_or_else(|| RunTimeError::ExpectedEdge {
              name: "type",
              index: 0,
            })?,
        )
        .map(|r| -> graph::Value { r.into() })?,
      )
    }
    else
    {
      Err(RunTimeError::InvalidNumberOfArguments { name: "type" }.into())
    }
  }
}
