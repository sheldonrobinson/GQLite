use std::{cell::RefCell, path::PathBuf};

use ccutils::pool::{self, Pool};

use crate::{
  prelude::*,
  store::{sqlbase::SqlValue, TransactionBoxable},
};

ccutils::assert_impl_all!(Store: Sync, Send);

use askama::Template;
use rusqlite::named_params;

//  _____     _____                    ____        _
// |_   _|__ |  ___| __ ___  _ __ ___ / ___|  __ _| |
//   | |/ _ \| |_ | '__/ _ \| '_ ` _ \\___ \ / _` | |
//   | | (_) |  _|| | | (_) | | | | | |___) | (_| | |
//   |_|\___/|_|  |_|  \___/|_| |_| |_|____/ \__, |_|
//                                              |_|

impl<'a> rusqlite::ToSql for SqlValue<'a>
{
  fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>>
  {
    match self
    {
      SqlValue::String(string) => string.to_sql(),
      SqlValue::StringRef(string) => string.to_sql(),
      SqlValue::Key(key) => Ok(rusqlite::types::ToSqlOutput::Owned(
        rusqlite::types::Value::Blob(key.uuid().to_be_bytes().into()),
      )),
      SqlValue::Blob(data) => data.to_sql(),
      SqlValue::Text(text) => std::str::from_utf8(text).unwrap().to_sql(),
      SqlValue::Float(fl) => fl.to_sql(),
      SqlValue::Integer(it) => it.to_sql(),
      SqlValue::Null => rusqlite::types::Null.to_sql(),
    }
  }
}

impl<'a> sqlbase::Row for rusqlite::Row<'a>
{
  fn get_value(&self, index: usize) -> Result<SqlValue<'_>>
  {
    match self.get_ref(index)?
    {
      rusqlite::types::ValueRef::Blob(blob) => Ok(SqlValue::Blob(blob)),
      rusqlite::types::ValueRef::Integer(int) => Ok(SqlValue::Integer(int)),
      rusqlite::types::ValueRef::Null => Ok(SqlValue::Null),
      rusqlite::types::ValueRef::Real(f) => Ok(SqlValue::Float(f)),
      rusqlite::types::ValueRef::Text(txt) => Ok(SqlValue::Text(txt)),
    }
  }
}

//  _____                               _   _
// |_   _| __ __ _ _ __  ___  __ _  ___| |_(_) ___  _ __
//   | || '__/ _` | '_ \/ __|/ _` |/ __| __| |/ _ \| '_ \
//   | || | | (_| | | | \__ \ (_| | (__| |_| | (_) | | | |
//   |_||_|  \__,_|_| |_|___/\__,_|\___|\__|_|\___/|_| |_|

struct TransactionBase
{
  connection: pool::Handle<rusqlite::Connection>,
  active: RefCell<bool>,
}

impl Drop for TransactionBase
{
  fn drop(&mut self)
  {
    if *self.active.borrow()
    {
      if let Err(e) = self.connection.execute("ROLLBACK", ())
      {
        println!(
          "Rollback failed with error {:?}, future use of the connection are likely to fail.",
          e
        );
      }
    }
  }
}

pub(crate) struct ReadTransaction
{
  transaction_base: TransactionBase,
}

impl super::ReadTransaction for ReadTransaction
{
  fn discard(mut self) -> Result<()>
  {
    self.transaction_base.connection.execute("ROLLBACK", ())?;
    *self.transaction_base.active.get_mut() = false;
    Ok(())
  }
}

pub(crate) struct WriteTransaction
{
  transaction_base: TransactionBase,
}

impl super::ReadTransaction for WriteTransaction
{
  fn discard(mut self) -> Result<()>
  {
    self.transaction_base.connection.execute("ROLLBACK", ())?;
    *self.transaction_base.active.get_mut() = false;
    Ok(())
  }
}
impl super::WriteTransaction for WriteTransaction
{
  fn commit(mut self) -> Result<()>
  {
    self.transaction_base.connection.execute("COMMIT", ())?;
    *self.transaction_base.active.get_mut() = false;
    Ok(())
  }
}

trait GetConnection
{
  fn get_connection(&self) -> &rusqlite::Connection;
}

impl GetConnection for super::TransactionBox<ReadTransaction, WriteTransaction>
{
  fn get_connection(&self) -> &rusqlite::Connection
  {
    use std::ops::Deref;
    match self
    {
      super::TransactionBox::Read(read) => read.transaction_base.connection.deref(),
      super::TransactionBox::Write(write) => write.transaction_base.connection.deref(),
    }
  }
}

//  _____                    _       _
// |_   _|__ _ __ ___  _ __ | | __ _| |_ ___  ___
//   | |/ _ \ '_ ` _ \| '_ \| |/ _` | __/ _ \/ __|
//   | |  __/ | | | | | |_) | | (_| | ||  __/\__ \
//   |_|\___|_| |_| |_| .__/|_|\__,_|\__\___||___/
//                    |_|

mod templates
{
  use askama::Template;

  // Graph related templates
  #[derive(Template)]
  #[template(path = "sql/sqlite/upgrade_graph_from_1_01.sql", escape = "none")]
  pub(super) struct UpgradeGraphFrom1_01<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/graph_create.sql", escape = "none")]
  pub(super) struct GraphCreate<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/graph_delete.sql", escape = "none")]
  pub(super) struct GraphDelete<'a>
  {
    pub graph_name: &'a str,
  }
  // Node related templates
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_create.sql", escape = "none")]
  pub(super) struct NodeCreate<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_delete.sql", escape = "none")]
  pub(super) struct NodeDelete<'a>
  {
    pub graph_name: &'a str,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_update.sql", escape = "none")]
  pub(super) struct NodeUpdate<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_select.sql", escape = "none")]
  pub(super) struct NodeSelect<'a>
  {
    pub graph_name: &'a str,
    pub keys_var: &'a Option<usize>,
    pub labels_var: &'a Option<usize>,
    pub properties_var: &'a Option<usize>,
  }
  // Edge queries
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_count_for_node.sql", escape = "none")]
  pub(super) struct EdgeCountForNode<'a>
  {
    pub graph_name: &'a str,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_create.sql", escape = "none")]
  pub(super) struct EdgeCreate<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_delete_by_nodes.sql", escape = "none")]
  pub(super) struct EdgeDeleteByNodes<'a>
  {
    pub graph_name: &'a str,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_delete.sql", escape = "none")]
  pub(super) struct EdgeDelete<'a>
  {
    pub graph_name: &'a str,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_update.sql", escape = "none")]
  pub(super) struct EdgeUpdate<'a>
  {
    pub graph_name: &'a str,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_select.sql", escape = "none")]
  pub(super) struct EdgeSelect<'a>
  {
    pub graph_name: &'a str,
    pub is_undirected: bool,
    pub table_suffix: &'a str,
    pub edge_keys_var: Option<usize>,
    pub edge_labels_var: Option<usize>,
    pub edge_properties_var: Option<usize>,
    pub left_keys_var: Option<usize>,
    pub left_labels_var: Option<usize>,
    pub left_properties_var: Option<usize>,
    pub right_keys_var: Option<usize>,
    pub right_labels_var: Option<usize>,
    pub right_properties_var: Option<usize>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/call_stats.sql", escape = "none")]
  pub(super) struct CallStats<'a>
  {
    pub graph_name: &'a str,
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

type TransactionBox = store::TransactionBox<ReadTransaction, WriteTransaction>;

pub(crate) struct Store
{
  connection: Pool<rusqlite::Connection, ErrorType>,
}
ccutils::assert_impl_all!(Store: Sync, Send);

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    let path: PathBuf = path.as_ref().into();
    let connection = Pool::new(
      move || Ok(rusqlite::Connection::open(&path)?),
      pool::Options::default().minimum_pool_size(1).pool_size(3),
    )?;
    let s = Self { connection };
    s.initialise()?;
    Ok(s)
  }
  pub(crate) fn in_memory() -> Result<Store>
  {
    let id = uuid::Uuid::new_v4().as_u128();
    let connection = Pool::new(
      move || {
        Ok(rusqlite::Connection::open_with_flags(
          format!("file:{}?mode=memory&cache=shared", id),
          rusqlite::OpenFlags::default(),
        )?)
      },
      pool::Options::default().minimum_pool_size(1).pool_size(3),
    )?;
    let s = Self { connection };
    s.initialise()?;
    Ok(s)
  }
  fn upgrade_database(&self, transaction: &mut TransactionBox, from: utils::Version) -> Result<()>
  {
    use crate::store::Store;
    if let (1, 0) = (from.major, from.minor)
    {
      // Create a metadata table and add the default graph.
      transaction.get_connection().execute(
        include_str!("../../templates/sql/sqlite/metadata_create_table.sql"),
        (),
      )?;
      self.set_metadata_value_json(transaction, "graphs", &vec!["default".to_string()])?;
    }
    match (from.major, from.minor)
    {
      (1, 0) | (1, 1) =>
      {
        // uuid function is needed for upgrade
        transaction.get_connection().create_scalar_function(
          "uuid",
          0,
          rusqlite::functions::FunctionFlags::SQLITE_UTF8,
          |_| {
            let uuid = uuid::Uuid::new_v4();
            let bytes = uuid.as_bytes(); // [u8; 16]
            Ok(rusqlite::types::Value::Blob(bytes.to_vec()))
          },
        )?;

        for graph in self.graphs_list(transaction)?
        {
          transaction.get_connection().execute_batch(
            templates::UpgradeGraphFrom1_01 { graph_name: &graph }
              .render()?
              .as_str(),
          )?;
        }
        transaction.get_connection().execute_batch(include_str!(
          "../../templates/sql/sqlite/upgrade_from_1_01.sql"
        ))?;
        Ok(())
      }
      _ => Err(
        StoreError::IncompatibleVersion {
          expected: consts::GQLITE_VERSION,
          actual: from,
        }
        .into(),
      ),
    }
  }
  /// Check if table exists
  pub(crate) fn check_if_table_exists(
    &self,
    transaction: &mut TransactionBox,
    table_name: impl Into<String>,
  ) -> Result<bool>
  {
    Ok(transaction.get_connection().query_row(
      include_str!("../../templates/sql/sqlite/table_exists.sql"),
      named_params! {":table_name": table_name.into()},
      |row| Ok(row.get::<_, i32>(0)? == 1),
    )?)
  }
}

impl super::sqlbase::SqlMetaDataQueries for Store
{
  fn metadata_get_query() -> Result<String>
  {
    Ok(include_str!("../../templates/sql/sqlite/metadata_get.sql").to_string())
  }
  fn metadata_set_query() -> Result<String>
  {
    Ok(include_str!("../../templates/sql/sqlite/metadata_set.sql").to_string())
  }
}

impl super::sqlbase::SqlStore for Store
{
  type TransactionBox = TransactionBox;
  type Row<'a> = rusqlite::Row<'a>;
  fn initialise(&self) -> Result<()>
  {
    use store::Store;
    let mut tx = self.begin_sql_write()?;
    if self.check_if_table_exists(&mut tx, "gqlite_metadata")?
    {
      // gqlite version 1.1 incorrectly use ' instead of " in the version number
      let version_raw = self
        .get_metadata_value::<String>(&mut tx, "version")?
        .replace("'", "\"");
      let version: utils::Version = serde_json::from_str(&version_raw)?;
      if version.major != consts::GQLITE_VERSION.major
        || version.minor != consts::GQLITE_VERSION.minor
      {
        self.upgrade_database(&mut tx, version)?;
      }
    }
    else if !self.check_if_table_exists(&mut tx, "gqlite_metadata")?
      && self.check_if_table_exists(&mut tx, "gqlite_default_nodes")?
    {
      // 1.0 didn't have the metadata table
      self.upgrade_database(
        &mut tx,
        utils::Version {
          major: 1,
          minor: 0,
          patch: 0,
        },
      )?;
    }
    else
    {
      tx.get_connection().execute(
        include_str!("../../templates/sql/sqlite/metadata_create_table.sql"),
        (),
      )?;
      self.set_metadata_value_json(&mut tx, "graphs", &Vec::<String>::new())?;
      self.create_graph(&mut tx, "default", true)?;
    }
    self.set_metadata_value_json(&mut tx, "version", &consts::GQLITE_VERSION)?;
    tx.close()?;
    Ok(())
  }
  fn begin_sql_read(&self) -> Result<Self::TransactionBox>
  {
    let connection = self.connection.get()?;
    connection.execute("BEGIN", ())?;
    Ok(Self::TransactionBox::from_read(ReadTransaction {
      transaction_base: TransactionBase {
        connection,
        active: RefCell::new(true),
      },
    }))
  }
  fn begin_sql_write(&self) -> Result<Self::TransactionBox>
  {
    let connection = self.connection.get()?;
    connection.execute("BEGIN", ())?;
    Ok(Self::TransactionBox::from_write(WriteTransaction {
      transaction_base: TransactionBase {
        connection,
        active: RefCell::new(true),
      },
    }))
  }

  fn execute_batch(
    &self,
    transaction: &mut Self::TransactionBox,
    sql: impl AsRef<str>,
  ) -> Result<()>
  {
    Ok(transaction.get_connection().execute_batch(sql.as_ref())?)
  }
  fn execute<'a>(
    &self,
    transaction: &mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl sqlbase::IntoBindings<'a>,
  ) -> Result<()>
  {
    transaction.get_connection().execute(
      sql.as_ref(),
      rusqlite::params_from_iter(bindings.into_bindings_iter()),
    )?;
    Ok(())
  }
  fn query_rows<'a, 'tx>(
    &self,
    transaction: &'tx mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl sqlbase::IntoBindings<'a>,
    mut f: impl for<'b> FnMut(&Self::Row<'b>) -> Result<()>,
  ) -> Result<()>
  {
    let mut statement = transaction.get_connection().prepare(sql.as_ref())?;
    let mut rows = statement.query(rusqlite::params_from_iter(bindings.into_bindings_iter()))?;
    loop
    {
      let row = rows.next()?;
      match row
      {
        Some(row) => f(row)?,
        None => break,
      }
    }
    Ok(())
  }
  fn query_row<'a, 'tx, T>(
    &self,
    transaction: &'tx mut Self::TransactionBox,
    sql: impl AsRef<str>,
    bindings: impl sqlbase::IntoBindings<'a>,
    f: impl for<'b> FnOnce(&Self::Row<'b>) -> Result<T>,
  ) -> Result<T>
  {
    transaction.get_connection().query_row(
      sql.as_ref(),
      rusqlite::params_from_iter(bindings.into_bindings_iter()),
      |row| Ok(f(row)),
    )?
  }
}

impl super::sqlbase::SqlQueries for Store
{
  fn graph_create_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::GraphCreate {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
  fn graph_delete(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::GraphDelete {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
  fn node_create_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::NodeCreate {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
  fn edge_delete_by_nodes_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>
  {
    Ok(
      templates::EdgeDeleteByNodes {
        graph_name: graph_name.as_ref(),
        keys: keys.as_ref(),
      }
      .render()?,
    )
  }

  fn edge_count_for_nodes_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>
  {
    Ok(
      templates::EdgeCountForNode {
        graph_name: graph_name.as_ref(),
        keys: keys.as_ref(),
      }
      .render()?,
    )
  }

  fn node_delete_query(graph_name: impl AsRef<str>, keys: impl AsRef<Vec<String>>)
    -> Result<String>
  {
    Ok(
      templates::NodeDelete {
        graph_name: graph_name.as_ref(),
        keys: keys.as_ref(),
      }
      .render()?,
    )
  }

  fn node_update_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::NodeUpdate {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }

  fn node_select_query(
    graph_name: impl AsRef<str>,
    keys_var: Option<usize>,
    labels_var: Option<usize>,
    properties_var: Option<usize>,
  ) -> Result<String>
  {
    Ok(
      templates::NodeSelect {
        graph_name: graph_name.as_ref(),
        keys_var: &keys_var,
        labels_var: &labels_var,
        properties_var: &properties_var,
      }
      .render()?,
    )
  }

  fn edge_create_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::EdgeCreate {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
  fn edge_delete_query(graph_name: impl AsRef<str>, keys: impl AsRef<Vec<String>>)
    -> Result<String>
  {
    Ok(
      templates::EdgeDelete {
        graph_name: graph_name.as_ref(),
        keys: keys.as_ref(),
      }
      .render()?,
    )
  }
  fn edge_update_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::EdgeUpdate {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
  fn edge_select_query(
    graph_name: impl AsRef<str>,
    is_undirected: bool,
    table_suffix: impl AsRef<str>,
    edge_keys_var: Option<usize>,
    edge_labels_var: Option<usize>,
    edge_properties_var: Option<usize>,
    left_keys_var: Option<usize>,
    left_labels_var: Option<usize>,
    left_properties_var: Option<usize>,
    right_keys_var: Option<usize>,
    right_labels_var: Option<usize>,
    right_properties_var: Option<usize>,
  ) -> Result<String>
  {
    Ok(
      templates::EdgeSelect {
        graph_name: graph_name.as_ref(),
        is_undirected,
        table_suffix: table_suffix.as_ref(),
        edge_keys_var,
        edge_labels_var,
        edge_properties_var,
        left_keys_var,
        left_labels_var,
        left_properties_var,
        right_keys_var,
        right_labels_var,
        right_properties_var,
      }
      .render()?,
    )
  }
  fn compute_statistics_query(graph_name: impl AsRef<str>) -> Result<String>
  {
    Ok(
      templates::CallStats {
        graph_name: graph_name.as_ref(),
      }
      .render()?,
    )
  }
}

#[cfg(test)]
mod tests
{
  use crate::{prelude::*, store::TransactionBoxable};
  #[test]
  fn test_sqlite_metadata()
  {
    let temp_file = crate::tests::create_tmp_file();
    let store = super::Store::open(temp_file.path()).unwrap();
    let mut tx = store.begin_sql_read().unwrap();
    let version: utils::Version = store.get_metadata_value_json(&mut tx, "version").unwrap();
    assert_eq!(version.major, consts::GQLITE_VERSION.major);
    assert_eq!(version.minor, consts::GQLITE_VERSION.minor);
    assert_eq!(version.patch, consts::GQLITE_VERSION.patch);
    tx.close().unwrap();
    drop(store);

    // Try to reopen
    let store = super::Store::open(temp_file.path()).unwrap();
    let mut tx = store.begin_sql_read().unwrap();
    let version: utils::Version = store.get_metadata_value_json(&mut tx, "version").unwrap();
    assert_eq!(version.major, consts::GQLITE_VERSION.major);
    assert_eq!(version.minor, consts::GQLITE_VERSION.minor);
    assert_eq!(version.patch, consts::GQLITE_VERSION.patch);
    tx.close().unwrap();
    drop(store);
  }
}
