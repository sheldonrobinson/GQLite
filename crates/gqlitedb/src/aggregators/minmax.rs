use std::fmt::Debug;

use super::AggregatorState;

use crate::{value::Value, Result};

#[derive(Debug)]
struct MinState
{
  value: Value,
}

impl MinState
{
  fn new() -> Result<Self>
  {
    Ok(Self { value: Value::Null })
  }
}

impl AggregatorState for MinState
{
  fn next(&mut self, value: Value) -> crate::Result<()>
  {
    if self.value.is_null()
    {
      self.value = value;
    }
    else if !value.is_null()
    {
      match value.orderability(&self.value)
      {
        std::cmp::Ordering::Less =>
        {
          self.value = value;
        }
        _ =>
        {}
      }
    }
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::value::Value>
  {
    Ok(self.value)
  }
}

super::declare_aggregator!(min, Min, MinState, () -> i64);

#[derive(Debug)]
struct MaxState
{
  value: Value,
}

impl MaxState
{
  fn new() -> Result<Self>
  {
    Ok(Self { value: Value::Null })
  }
}

impl AggregatorState for MaxState
{
  fn next(&mut self, value: Value) -> crate::Result<()>
  {
    if self.value.is_null()
    {
      self.value = value;
    }
    else if !value.is_null()
    {
      match value.orderability(&self.value)
      {
        std::cmp::Ordering::Greater =>
        {
          self.value = value;
        }
        _ =>
        {}
      }
    }
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::value::Value>
  {
    Ok(self.value)
  }
}

super::declare_aggregator!(max, Max, MaxState, () -> i64);
