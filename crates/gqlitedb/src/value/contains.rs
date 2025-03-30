use crate::prelude::*;

/// Compute contains, according to OpenCypher specification, specifically handling the comparison with null.
pub(crate) fn contains(
  container: &Vec<graph::Value>,
  value: &graph::Value,
) -> value::ComparisonResult
{
  use value::ComparisonResult;
  if value.is_null()
  {
    if container.is_empty()
    {
      ComparisonResult::False
    }
    else
    {
      ComparisonResult::ComparedNull
    }
  }
  else
  {
    let mut has_compared_to_null = false;
    for v_c in container.iter()
    {
      use value::ComparisonResult;
      match value::compare(v_c, &value)
      {
        ComparisonResult::True => return ComparisonResult::True,
        ComparisonResult::False =>
        {}
        ComparisonResult::ComparedNull => has_compared_to_null = true,
      }
    }
    if has_compared_to_null
    {
      ComparisonResult::ComparedNull
    }
    else
    {
      ComparisonResult::False
    }
  }
}
