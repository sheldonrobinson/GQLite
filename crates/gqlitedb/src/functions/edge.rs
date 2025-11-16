use crate::prelude::*;

use super::FResult;

#[derive(Debug, Default)]
pub(super) struct Type {}

impl Type
{
  fn call_impl(edge: &graph::Edge) -> FResult<String>
  {
    edge
      .labels()
      .first()
      .ok_or(RunTimeError::MissingEdgeLabel)
      .map(|v| v.to_owned())
  }
}

super::declare_function!(type, Type, call_impl(crate::graph::Edge) -> String);
