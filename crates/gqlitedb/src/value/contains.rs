use crate::prelude::*;

pub(crate) enum ContainResult
{
  True,
  False,
  ComparedNull,
}

/// Compute contains, according to OpenCypher specification, specifically handling the comparison with null.
pub(crate) fn contains(container: &[value::Value], value: &value::Value) -> ContainResult
{
  if value.is_null()
  {
    if container.is_empty()
    {
      ContainResult::False
    }
    else
    {
      ContainResult::ComparedNull
    }
  }
  else
  {
    let mut has_compared_to_null = false;
    for v_c in container.iter()
    {
      use value::ContainResult;
      match value::compare(v_c, value)
      {
        value::Ordering::Equal => return ContainResult::True,
        value::Ordering::ComparedNull => has_compared_to_null = true,
        _ =>
        {}
      }
    }
    if has_compared_to_null
    {
      ContainResult::ComparedNull
    }
    else
    {
      ContainResult::False
    }
  }
}
