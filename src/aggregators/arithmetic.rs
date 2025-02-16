use super::AggregatorState;

use crate::{error::RunTimeError, graph::Value, Result};

#[derive(Debug)]
struct SumState
{
  value: Value,
}

impl SumState
{
  fn new() -> Result<Self>
  {
    Ok(Self { value: 0.into() })
  }
}

impl AggregatorState for SumState
{
  fn next(&mut self, value: Value) -> crate::Result<()>
  {
    match self.value
    {
      Value::Boolean(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::String(..)
      | Value::Object(..)
      | Value::Path(..) =>
      {
        Err::<(), crate::error::Error>(RunTimeError::InvalidBinaryOperands.into())?
      }
      Value::Invalid =>
      {}
      Value::Float(lhs) => match value
      {
        Value::Float(rhs) => self.value = (lhs + rhs).into(),
        Value::Integer(rhs) => self.value = (lhs + rhs as f64).into(),
        _ => Err::<(), crate::error::Error>(RunTimeError::InvalidBinaryOperands.into())?,
      },
      Value::Integer(lhs) => match value
      {
        Value::Float(rhs) => self.value = (lhs as f64 + rhs).into(),
        Value::Integer(rhs) => self.value = (lhs + rhs).into(),
        _ => Err::<(), crate::error::Error>(RunTimeError::InvalidBinaryOperands.into())?,
      },
    }
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::graph::Value>
  {
    Ok(self.value)
  }
}

super::declare_aggregator!(sum, Sum, SumState, () -> i64);
