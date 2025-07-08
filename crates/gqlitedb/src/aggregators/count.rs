use super::AggregatorState;

use crate::Result;

#[derive(Debug)]
struct CountState
{
  count: i64,
}

impl CountState
{
  fn new() -> Result<Self>
  {
    Ok(Self { count: 0 })
  }
}

impl AggregatorState for CountState
{
  fn next(&mut self, _: crate::value::Value) -> crate::Result<()>
  {
    self.count += 1;
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::value::Value>
  {
    Ok(self.count.into())
  }
}

super::declare_aggregator!(count, Count, CountState, () -> i64);
