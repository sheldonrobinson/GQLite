use super::AggregatorState;

use crate::{value::Value, Result};

#[derive(Debug)]
struct CollectState
{
  value: Vec<Value>,
}

impl CollectState
{
  fn new() -> Result<Self>
  {
    Ok(Self {
      value: Default::default(),
    })
  }
}
impl AggregatorState for CollectState
{
  fn next(&mut self, value: Value) -> crate::Result<()>
  {
    self.value.push(value);
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::value::Value>
  {
    Ok(self.value.into())
  }
}

super::declare_aggregator!(collect, Collect, CollectState, () -> i64);
