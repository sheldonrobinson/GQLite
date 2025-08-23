use std::fmt::Debug;

use super::AggregatorState;

use crate::{error::RunTimeError, value::Value, Result};

trait Op
{
  fn op_i64(a: i64, b: i64) -> i64;
  fn op_f64(a: f64, b: f64) -> f64;
}

#[derive(Debug)]
struct OpState<T>
where
  T: Op + Debug,
{
  value: Value,
  _marker: std::marker::PhantomData<T>,
}

impl<T> OpState<T>
where
  T: Op + Debug,
{
  fn new() -> Result<Self>
  {
    Ok(Self {
      value: Value::Null,
      _marker: Default::default(),
    })
  }
}

impl<T> AggregatorState for OpState<T>
where
  T: Op + Debug,
{
  fn next(&mut self, value: Value) -> crate::Result<()>
  {
    match self.value
    {
      Value::Boolean(..)
      | Value::Key(..)
      | Value::Node(..)
      | Value::Edge(..)
      | Value::Array(..)
      | Value::String(..)
      | Value::Map(..)
      | Value::Path(..) => Err(RunTimeError::InvalidBinaryOperands)?,
      Value::Null =>
      {
        self.value = value;
      }
      Value::Float(state) => match value
      {
        Value::Null =>
        {}
        Value::Float(new_value) => self.value = T::op_f64(state, new_value).into(),
        Value::Integer(new_value) => self.value = T::op_f64(state, new_value as f64).into(),
        _ => Err(RunTimeError::InvalidBinaryOperands)?,
      },
      Value::Integer(state) => match value
      {
        Value::Null =>
        {}
        Value::Float(new_value) => self.value = T::op_f64(state as f64, new_value).into(),
        Value::Integer(new_value) => self.value = T::op_i64(state, new_value).into(),
        _ => Err(RunTimeError::InvalidBinaryOperands)?,
      },
    }
    Ok(())
  }
  fn finalise(self: Box<Self>) -> crate::Result<crate::value::Value>
  {
    Ok(self.value)
  }
}

#[derive(Debug)]
struct SumOp;

impl Op for SumOp
{
  fn op_f64(a: f64, b: f64) -> f64
  {
    a + b
  }
  fn op_i64(a: i64, b: i64) -> i64
  {
    a + b
  }
}

type SumState = OpState<SumOp>;

super::declare_aggregator!(sum, Sum, SumState, () -> i64);
