use rand::Rng;

use crate::prelude::*;

use super::FResult;

#[derive(Debug, Default)]
pub(super) struct Rand {}

impl Rand
{
  fn call_impl() -> FResult<f64>
  {
    Ok(rand::rng().random())
  }
}

super::declare_function!(rand, Rand, call_impl() -> Vec<f64>);

#[derive(Debug, Default)]
pub(super) struct Ceil {}

impl Ceil
{
  fn call_impl(value: &f64) -> FResult<f64>
  {
    Ok(value.ceil())
  }
}

super::declare_function!(ceil, Ceil, call_impl(f64) -> f64);

#[derive(Debug, Default)]
pub(super) struct Floor {}

impl Floor
{
  fn call_impl(value: &f64) -> FResult<f64>
  {
    Ok(value.floor())
  }
}

super::declare_function!(floor, Floor, call_impl(f64) -> f64);
