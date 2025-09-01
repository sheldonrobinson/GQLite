mod compare;
mod contains;

pub(crate) use compare::{compare, Ordering};
pub(crate) use contains::{contains, ContainResult};

pub use graphcore::{array, value_map, Value, ValueMap, ValueTryIntoRef};

pub(crate) trait ValueExt
{
  fn access<'a>(&self, path: impl Iterator<Item = &'a String>) -> Value;
  fn compare(&self, rhs: &Value) -> crate::value::Ordering;

  /// Compute the order between self and rhs, for OrderBy, according to the OpenCypher specification.
  /// This order is total.
  fn orderability(&self, rhs: &Value) -> std::cmp::Ordering;
}

fn orderability_map(lhs: &ValueMap, rhs: &ValueMap) -> std::cmp::Ordering
{
  let o = lhs.len().cmp(&rhs.len());
  match o
  {
    std::cmp::Ordering::Equal => lhs
      .iter()
      .map(|(key, value)| value.orderability(rhs.get(key).unwrap_or(&Value::Null)))
      .find(|p| *p != std::cmp::Ordering::Equal)
      .unwrap_or(std::cmp::Ordering::Equal),
    o => o,
  }
}
fn orderability_float(lhs: &f64, rhs: &f64) -> std::cmp::Ordering
{
  if lhs.is_nan()
  {
    if rhs.is_nan()
    {
      std::cmp::Ordering::Equal
    }
    else
    {
      std::cmp::Ordering::Greater
    }
  }
  else if rhs.is_nan()
  {
    std::cmp::Ordering::Less
  }
  else
  {
    lhs.total_cmp(rhs)
  }
}
impl ValueExt for Value
{
  fn access<'a>(&self, mut path: impl Iterator<Item = &'a String>) -> Value
  {
    match path.next()
    {
      Some(name) => match self
      {
        Value::Node(node) => match node.properties().get(name)
        {
          Some(val) => val.access(path),
          None => Value::Null,
        },
        Value::Edge(edge) => match edge.properties().get(name)
        {
          Some(val) => val.access(path),
          None => Value::Null,
        },
        Value::Map(obj) => match obj.get(name)
        {
          Some(val) => val.access(path),
          None => Value::Null,
        },
        _ => Value::Null,
      },
      None => self.to_owned(),
    }
  }
  fn compare(&self, rhs: &Value) -> crate::value::Ordering
  {
    crate::value::compare(self, rhs)
  }

  /// Compute the order between self and rhs, for OrderBy, according to the OpenCypher specification.
  /// This order is total.
  fn orderability(&self, rhs: &Value) -> std::cmp::Ordering
  {
    match self
    {
      Value::Null => match rhs
      {
        Value::Null => std::cmp::Ordering::Equal,
        _ => std::cmp::Ordering::Greater,
      },
      Value::Key(lhs) => match rhs
      {
        Value::Null => std::cmp::Ordering::Less,
        Value::Key(rhs) => lhs.uuid().cmp(&rhs.uuid()),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Integer(lhs) => match rhs
      {
        Value::Null | Value::Key(..) => std::cmp::Ordering::Less,
        Value::Integer(rhs) => lhs.cmp(rhs),
        Value::Float(rhs) => orderability_float(&(*lhs as f64), rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Float(lhs) => match rhs
      {
        Value::Null | Value::Key(..) => std::cmp::Ordering::Less,
        Value::Integer(rhs) => orderability_float(lhs, &(*rhs as f64)),
        Value::Float(rhs) => orderability_float(lhs, rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Boolean(lhs) => match rhs
      {
        Value::Null | Value::Key(..) | Value::Integer(..) | Value::Float(..) =>
        {
          std::cmp::Ordering::Less
        }
        Value::Boolean(rhs) => lhs.cmp(rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::String(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..) => std::cmp::Ordering::Less,
        Value::String(rhs) => lhs.cmp(rhs),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Path(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..) => std::cmp::Ordering::Less,
        Value::Path(rhs) =>
        {
          match orderability_map(lhs.source().properties(), rhs.source().properties())
          {
            std::cmp::Ordering::Equal =>
            {
              match orderability_map(lhs.properties(), rhs.properties())
              {
                std::cmp::Ordering::Equal => orderability_map(
                  lhs.destination().properties(),
                  rhs.destination().properties(),
                ),
                o => o,
              }
            }
            o => o,
          }
        }
        _ => std::cmp::Ordering::Greater,
      },
      Value::Array(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..) => std::cmp::Ordering::Less,
        Value::Array(rhs) => lhs
          .iter()
          .zip(rhs.iter())
          .map(|(lhs, rhs)| Self::orderability(lhs, rhs))
          .find(|p| *p != std::cmp::Ordering::Equal)
          .unwrap_or(lhs.len().cmp(&rhs.len())),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Edge(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..) => std::cmp::Ordering::Less,
        Value::Edge(rhs) => orderability_map(lhs.properties(), rhs.properties()),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Node(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..)
        | Value::Edge(..) => std::cmp::Ordering::Less,
        Value::Node(rhs) => orderability_map(lhs.properties(), rhs.properties()),
        _ => std::cmp::Ordering::Greater,
      },
      Value::Map(lhs) => match rhs
      {
        Value::Null
        | Value::Key(..)
        | Value::Integer(..)
        | Value::Float(..)
        | Value::Boolean(..)
        | Value::String(..)
        | Value::Path(..)
        | Value::Array(..)
        | Value::Edge(..)
        | Value::Node(..) => std::cmp::Ordering::Less,
        Value::Map(rhs) => orderability_map(lhs, rhs),
      },
    }
  }
}
