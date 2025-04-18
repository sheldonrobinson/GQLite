use crate::prelude::*;

pub(crate) enum ComparisonResult
{
  True,
  False,
  ComparedNull,
}

impl Into<graph::Value> for ComparisonResult
{
  fn into(self) -> graph::Value
  {
    match self
    {
      ComparisonResult::True => true.into(),
      ComparisonResult::False => false.into(),
      ComparisonResult::ComparedNull => graph::Value::Invalid,
    }
  }
}

impl From<bool> for ComparisonResult
{
  fn from(value: bool) -> Self
  {
    match value
    {
      true => Self::True,
      false => Self::False,
    }
  }
}

impl std::ops::Not for ComparisonResult
{
  type Output = ComparisonResult;
  fn not(self) -> Self::Output
  {
    match self
    {
      ComparisonResult::True => ComparisonResult::False,
      ComparisonResult::False => ComparisonResult::True,
      ComparisonResult::ComparedNull => ComparisonResult::ComparedNull,
    }
  }
}

pub(crate) fn compare(left: &graph::Value, right: &graph::Value) -> ComparisonResult
{
  use graph::Value;
  match left
  {
    Value::Invalid => ComparisonResult::ComparedNull,
    Value::Boolean(bl) => match right
    {
      Value::Boolean(br) => (bl == br).into(),
      Value::Invalid => ComparisonResult::ComparedNull,
      _ => ComparisonResult::False,
    },
    Value::Integer(il) => match right
    {
      Value::Integer(ir) => (il == ir).into(),
      Value::Invalid => ComparisonResult::ComparedNull,
      _ => ComparisonResult::False,
    },
    Value::Float(fl) => match right
    {
      Value::Float(fr) => (fl == fr).into(),
      Value::Invalid => ComparisonResult::ComparedNull,
      _ => ComparisonResult::False,
    },
    Value::String(sl) => match right
    {
      Value::String(sr) => (sl == sr).into(),
      Value::Invalid => ComparisonResult::ComparedNull,
      _ => ComparisonResult::False,
    },
    Value::Array(al) => match right
    {
      Value::Array(ar) =>
      {
        if al.len() != ar.len()
        {
          ComparisonResult::False
        }
        else
        {
          let (comp, comp_to_null) =
            al.iter()
              .zip(ar.iter())
              .fold((true, false), |acc, (l, r)| match compare(l, r)
              {
                ComparisonResult::ComparedNull => (acc.0, true),
                ComparisonResult::True => acc,
                ComparisonResult::False => (false, acc.1),
              });
          if comp
          {
            if comp_to_null
            {
              // this is opencypher madness, if you compare [nil, 2] with [1, 2] this should return null because of the nil/1 comparison,
              // however [nil, 2] with [1, "2"] should return false because 2 != "2" 🤦
              ComparisonResult::ComparedNull
            }
            else
            {
              ComparisonResult::True
            }
          }
          else
          {
            ComparisonResult::False
          }
        }
      }
      Value::Invalid => ComparisonResult::ComparedNull,
      _ => ComparisonResult::False,
    },
    // Value::Object(vol),
    // Value::Node(Node),
    // Value::Edge(Edge),
    // Value::Path(Path),
    _ => todo!("value::compare"),
  }
}
