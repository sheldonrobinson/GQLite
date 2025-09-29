use crate::{prelude::*, store::TransactionBoxable};

use super::{FromSqlValue, IntoBindings, SqlValue};

pub(crate) trait Row: Sized
{
  fn get<T: FromSqlValue>(&self, index: usize) -> Result<T>
  {
    T::from_sql_value(self.get_value(index)?)
  }
  fn get_value(&self, index: usize) -> Result<SqlValue<'_>>;
}

/// Base trait for SQL Store.
pub(crate) trait SqlStore
{
  type TransactionBox: TransactionBoxable;
  type Row<'a>: Row;
  /// Initialise an SQL store
  fn initialise(&self) -> Result<()>;

  /// Begin a read transaction.
  fn begin_sql_read(&self) -> Result<Self::TransactionBox>;
  /// Begin a write transaction.
  fn begin_sql_write(&self) -> Result<Self::TransactionBox>;
  /// Execute a batch of SQL queries for the given transaction.
  fn execute_batch(
    &self,
    transaction: &mut Self::TransactionBox,
    sql: impl AsRef<str>,
  ) -> Result<()>;
  /// Execute a single SQL query for the given transaction.
  fn execute<'a>(
    &self,
    transaction: &mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl IntoBindings<'a>,
  ) -> Result<()>;
  /// Execute a SQL Query that return a single row.
  fn query_row<'a, 'tx, T>(
    &self,
    transaction: &'tx mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl IntoBindings<'a>,
    f: impl for<'b> FnOnce(&Self::Row<'b>) -> Result<T>,
  ) -> Result<T>;
  /// Execute a SQL Query that return multiple rows.
  fn query_rows<'a, 'tx>(
    &self,
    transaction: &'tx mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl IntoBindings<'a>,
    f: impl for<'b> FnMut(&Self::Row<'b>) -> Result<()>,
  ) -> Result<()>;
}
