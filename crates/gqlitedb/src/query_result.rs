use graphcore::value_map;
use itertools::Itertools as _;

use crate::error::InternalError;

/// Hold the result of executing a query.
#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum QueryResult
{
  /// No results are returned by the query
  Empty,
  /// A table is returned by the query
  Table(graphcore::Table),
  /// Many tables are returned by the query. Happpen if multiple queries are executed at the same time.
  Array(Vec<QueryResult>),
  /// A value is returned
  Value(graphcore::Value),
}

impl QueryResult
{
  /// Convert this query result into a value.
  pub fn into_value(self) -> graphcore::Value
  {
    match self {
      QueryResult::Empty => graphcore::Value::Null,
      QueryResult::Value(v) => v,
      QueryResult::Table(t) => t.into(),
      QueryResult::Array(qr) => value_map!("type" => "results", "results" => qr.into_iter().map(|x| x.into_value()).collect::<Vec<graphcore::Value>>()).into()
    }
  }
  /// Try convert into table
  pub fn try_into_table(self) -> Result<graphcore::Table, InternalError>
  {
    self.try_into()
  }
}

impl From<graphcore::Value> for QueryResult
{
  fn from(value: graphcore::Value) -> Self
  {
    QueryResult::Value(value)
  }
}

impl From<graphcore::Table> for QueryResult
{
  fn from(table: graphcore::Table) -> Self
  {
    QueryResult::Table(table)
  }
}

impl TryInto<graphcore::Table> for QueryResult
{
  type Error = InternalError;
  fn try_into(self) -> Result<graphcore::Table, Self::Error>
  {
    match self
    {
      QueryResult::Table(tbl) => Ok(tbl),
      _ => Err(InternalError::InvalidQueryResultCast),
    }
  }
}

impl std::fmt::Display for QueryResult
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      QueryResult::Empty => write!(f, "empty"),
      QueryResult::Value(v) => write!(f, "{}", v),
      QueryResult::Array(arr) => write!(f, "[{}]", arr.iter().map(|x| x.to_string()).join(", ")),
      QueryResult::Table(t) => write!(
        f,
        "[{};{}]",
        t.headers().join(", "),
        t.row_iter()
          .map(|x| x.iter().map(|x| x.to_string()).join(", "))
          .join(";")
      ),
    }
  }
}
