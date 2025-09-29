use crate::prelude::*;

#[derive(Debug)]
pub(crate) enum SqlValue<'a>
{
  StringRef(&'a String),
  String(String),
  Key(&'a graph::Key),
  Blob(&'a [u8]),
  Text(&'a [u8]),
  Float(f64),
  Integer(i64),
  Null,
}

impl<'a> From<&'a String> for SqlValue<'a>
{
  fn from(val: &'a String) -> Self
  {
    SqlValue::StringRef(val)
  }
}

impl<'a> From<String> for SqlValue<'a>
{
  fn from(val: String) -> Self
  {
    SqlValue::String(val)
  }
}

impl<'a> From<&'a graph::Key> for SqlValue<'a>
{
  fn from(val: &'a graph::Key) -> Self
  {
    SqlValue::Key(val)
  }
}

pub(crate) trait IntoBindings<'a>
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>;
}

impl<'a> IntoBindings<'a> for ()
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    vec![].into_iter()
  }
}

impl<'a, T0> IntoBindings<'a> for (T0,)
where
  T0: Into<SqlValue<'a>>,
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    vec![self.0.into()].into_iter()
  }
}

impl<'a, T0, T1> IntoBindings<'a> for (T0, T1)
where
  T0: Into<SqlValue<'a>>,
  T1: Into<SqlValue<'a>>,
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    vec![self.0.into(), self.1.into()].into_iter()
  }
}

impl<'a, T0, T1, T2> IntoBindings<'a> for (T0, T1, T2)
where
  T0: Into<SqlValue<'a>>,
  T1: Into<SqlValue<'a>>,
  T2: Into<SqlValue<'a>>,
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    vec![self.0.into(), self.1.into(), self.2.into()].into_iter()
  }
}

impl<'a, T0, T1, T2, T3, T4> IntoBindings<'a> for (T0, T1, T2, T3, T4)
where
  T0: Into<SqlValue<'a>>,
  T1: Into<SqlValue<'a>>,
  T2: Into<SqlValue<'a>>,
  T3: Into<SqlValue<'a>>,
  T4: Into<SqlValue<'a>>,
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    vec![
      self.0.into(),
      self.1.into(),
      self.2.into(),
      self.3.into(),
      self.4.into(),
    ]
    .into_iter()
  }
}

impl<'a> IntoBindings<'a> for Vec<SqlValue<'a>>
{
  fn into_bindings_iter(self) -> impl Iterator<Item = SqlValue<'a>>
  {
    self.into_iter()
  }
}

pub(crate) trait FromSqlValue: Sized
{
  fn from_sql_value<'a>(value: SqlValue<'a>) -> Result<Self>;
}

impl FromSqlValue for String
{
  fn from_sql_value<'a>(value: SqlValue<'a>) -> Result<Self>
  {
    match value
    {
      SqlValue::Text(text) => Ok(std::str::from_utf8(text)?.to_string()),
      SqlValue::String(string) => Ok(string),
      SqlValue::StringRef(string) => Ok(string.clone()),
      _ =>
      {
        println!("{:?} to string", value);
        Err(InternalError::InvalidQueryResultCast.into())
      }
    }
  }
}

impl FromSqlValue for usize
{
  fn from_sql_value<'a>(value: SqlValue<'a>) -> Result<Self>
  {
    match value
    {
      SqlValue::Integer(i) => Ok(i as usize),
      _ =>
      {
        println!("{:?} to usize", value);
        Err(InternalError::InvalidQueryResultCast.into())
      }
    }
  }
}

impl FromSqlValue for u32
{
  fn from_sql_value<'a>(value: SqlValue<'a>) -> Result<Self>
  {
    match value
    {
      SqlValue::Integer(i) => Ok(i as u32),
      _ =>
      {
        println!("{:?} to u32", value);
        Err(InternalError::InvalidQueryResultCast.into())
      }
    }
  }
}

impl FromSqlValue for graph::Key
{
  fn from_sql_value<'a>(value: SqlValue<'a>) -> Result<Self>
  {
    match value
    {
      SqlValue::Integer(i) => Ok(graphcore::Key::new(i as u128)),
      SqlValue::Key(k) => Ok(*k),
      SqlValue::Blob(b) => <[u8; 16]>::try_from(b)
        .map(u128::from_be_bytes)
        .map(graphcore::Key::new)
        .map_err(|_| InternalError::InvalidQueryResultCast.into()),
      _ =>
      {
        println!("{:?} to key", value);
        Err(InternalError::InvalidQueryResultCast.into())
      }
    }
  }
}
