use serde::{Deserialize, Serialize};

use super::{FromSqlValue, SqlValue};
use crate::prelude::*;

pub(crate) trait SqlMetaDataQueries
{
  fn metadata_get_query() -> Result<String>;
  fn metadata_set_query() -> Result<String>;
}

pub(crate) trait SqlMetaDataStore: super::SqlStore
{
  /// Get the metavalue
  fn get_optional_metadata_value<T: FromSqlValue>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
  ) -> Result<Option<T>>;
  fn get_metadata_value<T: FromSqlValue>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
  ) -> Result<T>
  {
    let key = key.into();
    self
      .get_optional_metadata_value(transaction, &key)?
      .ok_or_else(|| InternalError::MissingMetadata { key }.into())
  }
  #[allow(dead_code)]
  fn get_metadata_value_or_else<T: FromSqlValue>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
    f: impl FnOnce() -> T,
  ) -> Result<T>
  {
    let v = self.get_optional_metadata_value(transaction, key)?;
    Ok(v.unwrap_or_else(f))
  }
  /// Get the metadata value and deserialize it from JSON.
  fn get_metadata_value_json<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
  ) -> Result<T>
  {
    Ok(serde_json::from_str(
      &self.get_metadata_value::<String>(transaction, key)?,
    )?)
  }
  #[allow(dead_code)]
  fn get_metadata_value_json_or_else<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
    f: impl FnOnce() -> T,
  ) -> Result<T>
  {
    let v = self.get_optional_metadata_value::<String>(transaction, key)?;
    v.map_or_else(|| Ok(f()), |x| Ok(serde_json::from_str(&x)?))
  }
  fn set_metadata_value<'a>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
    value: impl Into<SqlValue<'a>>,
  ) -> Result<()>;
  /// Serialize a value to JSON and store the result in the database.
  fn set_metadata_value_json(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
    value: &impl Serialize,
  ) -> Result<()>
  {
    self.set_metadata_value(transaction, key, &serde_json::to_string(value)?)
  }
}

impl<TStore> SqlMetaDataStore for TStore
where
  TStore: super::SqlStore + SqlMetaDataQueries,
{
  fn get_optional_metadata_value<T: FromSqlValue>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
  ) -> Result<Option<T>>
  {
    let mut res = None;
    self.query_rows(
      transaction,
      Self::metadata_get_query()?,
      (&key.into(),),
      |x| {
        res = Some(x.get(0)?);
        Ok(())
      },
    )?;
    Ok(res)
  }
  fn set_metadata_value<'a>(
    &self,
    transaction: &mut Self::TransactionBox,
    key: impl Into<String>,
    value: impl Into<SqlValue<'a>>,
  ) -> Result<()>
  {
    self.execute(
      transaction,
      Self::metadata_set_query()?,
      (&key.into(), value.into()),
    )
  }
}
