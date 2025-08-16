use crate::prelude::*;

use super::FResult;

#[derive(Debug, Default)]
pub(super) struct Labels {}

impl Labels
{
  fn call_impl(node: &graph::Node) -> FResult<Vec<String>>
  {
    Ok(node.labels().to_owned())
  }
}

super::declare_function!(labels, Labels, call_impl(crate::graph::Node) -> Vec<String>);
