use std::{cell::RefCell, collections::HashSet, path::PathBuf};

use askama::Template;
use ccutils::pool::{self, Pool};
use rusqlite::{named_params, types::FromSql, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};

use crate::{
  prelude::*,
  store::{EdgeResult, TransactionBoxable},
};

//  _____     _____                    ____        _
// |_   _|__ |  ___| __ ___  _ __ ___ / ___|  __ _| |
//   | |/ _ \| |_ | '__/ _ \| '_ ` _ \\___ \ / _` | |
//   | | (_) |  _|| | | (_) | | | | | |___) | (_| | |
//   |_|\___/|_|  |_|  \___/|_| |_| |_|____/ \__, |_|
//                                              |_|

impl rusqlite::ToSql for graph::Key
{
  fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>>
  {
    Ok(rusqlite::types::ToSqlOutput::Owned(
      rusqlite::types::Value::Blob(self.uuid.to_be_bytes().into()),
    ))
  }
}

impl rusqlite::types::FromSql for graph::Key
{
  fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self>
  {
    Ok(Self {
      uuid: u128::from_be_bytes(<[u8; 16]>::column_result(value)?),
    })
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
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/graph_create.sql", escape = "none")]
  pub(super) struct GraphCreate<'a>
  {
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/graph_delete.sql", escape = "none")]
  pub(super) struct GraphDelete<'a>
  {
    pub graph_name: &'a String,
  }
  // Node related templates
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_create.sql", escape = "none")]
  pub(super) struct NodeCreate<'a>
  {
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_delete.sql", escape = "none")]
  pub(super) struct NodeDelete<'a>
  {
    pub graph_name: &'a String,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_update.sql", escape = "none")]
  pub(super) struct NodeUpdate<'a>
  {
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_select.sql", escape = "none")]
  pub(super) struct NodeSelect<'a>
  {
    pub graph_name: &'a String,
    pub has_keys: bool,
    pub has_labels: bool,
    pub has_properties: bool,
  }
  // Edge queries
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_count_for_node.sql", escape = "none")]
  pub(super) struct EdgeCountForNode<'a>
  {
    pub graph_name: &'a String,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_create.sql", escape = "none")]
  pub(super) struct EdgeCreate<'a>
  {
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_delete_by_nodes.sql", escape = "none")]
  pub(super) struct EdgeDeleteByNodes<'a>
  {
    pub graph_name: &'a String,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_delete.sql", escape = "none")]
  pub(super) struct EdgeDelete<'a>
  {
    pub graph_name: &'a String,
    pub keys: &'a Vec<String>,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_update.sql", escape = "none")]
  pub(super) struct EdgeUpdate<'a>
  {
    pub graph_name: &'a String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_select.sql", escape = "none")]
  pub(super) struct EdgeSelect<'a>
  {
    pub graph_name: &'a String,
    pub is_undirected: bool,
    pub table_suffix: &'a str,
    pub has_edge_keys: bool,
    pub has_edge_labels: bool,
    pub has_edge_properties: bool,
    pub has_n_left_keys: bool,
    pub has_n_left_labels: bool,
    pub has_n_left_properties: bool,
    pub has_n_right_keys: bool,
    pub has_n_right_labels: bool,
    pub has_n_right_properties: bool,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/call_stats.sql", escape = "none")]
  pub(super) struct CallStats<'a>
  {
    pub graph_name: &'a String,
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

type TransactionBox = store::TransactionBox<ReadTransaction, WriteTransaction>;

fn hex(key: &graph::Key) -> String
{
  format!("{:032X}", key.uuid)
}

pub(crate) struct Store
{
  connection: Pool<rusqlite::Connection, Error>,
}

ccutils::assert_impl_all!(Store: Sync, Send);

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    use store::Store;
    let path: PathBuf = path.as_ref().into();
    let connection = Pool::new(
      move || Ok(rusqlite::Connection::open(&path)?),
      pool::Options::default().minimum_pool_size(1).pool_size(3),
    )?;
    let s = Self { connection };

    let mut tx = s.begin_write()?;
    if s.check_if_table_exists(&mut tx, "gqlite_metadata")?
    {
      // gqlite version 1.1 incorrectly use ' instead of " in the version number
      let version_raw = s
        .get_metadata_value::<String>(&mut tx, "version")?
        .replace("'", "\"");
      let version: utils::Version = serde_json::from_str(&version_raw)?;
      if version.major != consts::GQLITE_VERSION.major
        || version.minor != consts::GQLITE_VERSION.minor
      {
        s.upgrade_database(&mut tx, version)?;
      }
    }
    else if !s.check_if_table_exists(&mut tx, "gqlite_metadata")?
      && s.check_if_table_exists(&mut tx, "gqlite_default_nodes")?
    {
      // 1.0 didn't have the metadata table
      s.upgrade_database(
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
      s.set_metadata_value_json(&mut tx, "graphs", &Vec::<String>::new())?;
      s.create_graph(&mut tx, &"default".to_string(), true)?;
    }
    s.set_metadata_value_json(&mut tx, "version", &consts::GQLITE_VERSION)?;
    tx.close()?;
    Ok(s)
  }
  fn upgrade_database(&self, transaction: &mut TransactionBox, from: utils::Version) -> Result<()>
  {
    use crate::store::Store;
    match (from.major, from.minor)
    {
      (1, 0) =>
      {
        // Create a metadata table and add the default graph.
        transaction.get_connection().execute(
          include_str!("../../templates/sql/sqlite/metadata_create_table.sql"),
          (),
        )?;
        self.set_metadata_value_json(transaction, "graphs", &vec!["default".to_string()])?;
      }
      _ =>
      {}
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
  pub(crate) fn get_metadata_value<T: FromSql>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
  ) -> Result<T>
  {
    Ok(transaction.get_connection().query_row(
      include_str!("../../templates/sql/sqlite/metadata_get.sql"),
      named_params! {":name": key.into()},
      |row| row.get(0),
    )?)
  }
  pub(crate) fn get_metadata_value_or_else<T: FromSql>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    f: impl FnOnce() -> T,
  ) -> Result<T>
  {
    Ok(
      transaction
        .get_connection()
        .query_row(
          include_str!("../../templates/sql/sqlite/metadata_get.sql"),
          named_params! { ":name": key.into()},
          |row| row.get(0),
        )
        .optional()
        .map(|v| v.unwrap_or_else(f))?,
    )
  }
  pub(crate) fn set_metadata_value(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    value: &impl ToSql,
  ) -> Result<()>
  {
    transaction.get_connection().execute(
      include_str!("../../templates/sql/sqlite/metadata_set.sql"),
      named_params! { ":name": key.into(), ":value": value },
    )?;
    Ok(())
  }

  pub(crate) fn get_metadata_value_json<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
  ) -> Result<T>
  {
    Ok(serde_json::from_str(
      &self.get_metadata_value::<String>(transaction, key)?,
    )?)
  }
  pub(crate) fn get_metadata_value_json_or_else<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    f: impl FnOnce() -> T,
  ) -> Result<T>
  {
    Ok(
      transaction
        .get_connection()
        .query_row(
          include_str!("../../templates/sql/sqlite/metadata_get.sql"),
          named_params! {":name": key.into()},
          |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|v| {
          v.map(|x| serde_json::from_str(&x))
            .unwrap_or_else(|| Ok(f()))
        })??,
    )
  }
  pub(crate) fn set_metadata_value_json(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    value: &impl Serialize,
  ) -> Result<()>
  {
    self.set_metadata_value(transaction, key, &serde_json::to_string(value)?)
  }
}

impl store::Store for Store
{
  type TransactionBox = TransactionBox;
  fn begin_read(&self) -> Result<Self::TransactionBox>
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
  fn begin_write(&self) -> Result<Self::TransactionBox>
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
  fn graphs_list(&self, transaction: &mut Self::TransactionBox) -> Result<Vec<String>>
  {
    self.get_metadata_value_json(transaction, "graphs")
  }
  fn create_graph(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    ignore_if_exists: bool,
  ) -> Result<()>
  {
    let mut graphs_list = self.graphs_list(transaction)?;
    if graphs_list.contains(graph_name)
    {
      if ignore_if_exists
      {
        return Ok(());
      }
      else
      {
        return Err(
          StoreError::DuplicatedGraph {
            graph_name: graph_name.to_owned(),
          }
          .into(),
        );
      }
    }
    transaction.get_connection().execute_batch(
      templates::GraphCreate {
        graph_name: &graph_name,
      }
      .render()?
      .as_str(),
    )?;
    graphs_list.push(graph_name.to_owned());
    self.set_metadata_value_json(transaction, "graphs", &graphs_list)?;
    Ok(())
  }
  fn delete_graph(&self, transaction: &mut Self::TransactionBox, graph_name: &String)
    -> Result<()>
  {
    let mut graphs_list = self.graphs_list(transaction)?;
    if graphs_list.contains(graph_name)
    {
      transaction.get_connection().execute_batch(
        templates::GraphDelete {
          graph_name: &graph_name,
        }
        .render()?
        .as_str(),
      )?;
      graphs_list.retain(|x| x != graph_name);
      self.set_metadata_value_json(transaction, "graphs", &graphs_list)?;

      Ok(())
    }
    else
    {
      Err(
        StoreError::UnknownGraph {
          graph_name: graph_name.to_owned(),
        }
        .into(),
      )
    }
  }
  fn create_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    nodes_iter: T,
  ) -> Result<()>
  {
    for x in nodes_iter
    {
      transaction.get_connection().execute(
        templates::NodeCreate {
          graph_name: graph_name.into(),
        }
        .render()?
        .as_str(),
        (
          x.key,
          serde_json::to_string(&x.labels)?,
          serde_json::to_string(&x.properties)?,
        ),
      )?;
    }
    Ok(())
  }
  fn delete_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: store::SelectNodeQuery,
    detach: bool,
  ) -> Result<()>
  {
    let nodes = self.select_nodes(transaction, graph_name, query)?;
    let nodes_keys: Vec<String> = nodes.into_iter().map(|x| hex(&x.key)).collect();
    if detach
    {
      transaction.get_connection().execute(
        templates::EdgeDeleteByNodes {
          graph_name: &graph_name.into(),
          keys: &nodes_keys,
        }
        .render()?
        .as_str(),
        (),
      )?;
    }
    else
    {
      let count = transaction.get_connection().query_row(
        templates::EdgeCountForNode {
          graph_name: &graph_name,
          keys: &nodes_keys,
        }
        .render()?
        .as_str(),
        (),
        |row| row.get::<_, usize>(0),
      )?;
      if count > 0
      {
        return Err(error::RunTimeError::DeleteConnectedNode.into());
      }
    }
    transaction.get_connection().execute(
      templates::NodeDelete {
        graph_name: &graph_name,
        keys: &nodes_keys,
      }
      .render()?
      .as_str(),
      (),
    )?;
    Ok(())
  }
  fn update_node(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    node: &crate::graph::Node,
  ) -> Result<()>
  {
    transaction.get_connection().execute(
      templates::NodeUpdate {
        graph_name: &graph_name,
      }
      .render()?
      .as_str(),
      named_params! {
        ":key": node.key,
        ":labels": serde_json::to_string(&node.labels)?,
        ":properties": serde_json::to_string(&node.properties)?
      },
    )?;
    Ok(())
  }
  fn select_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: store::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
    let mut prepared_query = transaction.get_connection().prepare(
      templates::NodeSelect {
        graph_name: graph_name.into(),
        has_keys: query.keys.is_some(),
        has_labels: query.labels.is_some(),
        has_properties: query.properties.is_some(),
      }
      .render()?
      .as_str(),
    )?;
    let mut bindings = Vec::<(&'static str, String)>::new();
    if let Some(keys) = query.keys
    {
      let hex_keys = keys.iter().map(|key| hex(key)).collect::<Vec<_>>();
      bindings.push((":keys", serde_json::to_string(&hex_keys)?));
    }
    if let Some(labels) = query.labels
    {
      bindings.push((":labels", serde_json::to_string(&labels)?));
    }
    if let Some(properties) = query.properties
    {
      bindings.push((":properties", serde_json::to_string(&properties)?));
    }
    let mut it = prepared_query.query(
      bindings
        .iter()
        .map(|(k, v)| (*k, v as &dyn rusqlite::ToSql))
        .collect::<Vec<_>>()
        .as_slice(),
    )?;
    let mut nodes: Vec<graph::Node> = Default::default();
    while let Some(row) = it.next()?
    {
      let key = row.get(0)?;
      let labels = serde_json::from_str(&row.get::<_, String>(1)?)?;
      let properties = serde_json::from_str(&row.get::<_, String>(2)?)?;
      nodes.push(graph::Node {
        key,
        labels,
        properties,
      });
    }
    Ok(nodes)
  }
  fn create_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    edges_iter: T,
  ) -> Result<()>
  {
    for x in edges_iter
    {
      transaction.get_connection().execute(
        templates::EdgeCreate {
          graph_name: graph_name.into(),
        }
        .render()?
        .as_str(),
        (
          x.key,
          serde_json::to_string(&x.labels)?,
          serde_json::to_string(&x.properties)?,
          x.source.key,
          x.destination.key,
        ),
      )?;
    }
    Ok(())
  }
  fn delete_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: store::SelectEdgeQuery,
    directivity: crate::graph::EdgeDirectivity,
  ) -> Result<()>
  {
    let edges = self.select_edges(transaction, graph_name, query, directivity)?;
    let edges_keys: Vec<String> = edges.into_iter().map(|x| hex(&x.edge.key)).collect();
    transaction.get_connection().execute(
      templates::EdgeDelete {
        graph_name: &graph_name,
        keys: &edges_keys,
      }
      .render()?
      .as_str(),
      (),
    )?;
    Ok(())
  }
  fn update_edge(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    edge: &crate::graph::Edge,
  ) -> Result<()>
  {
    transaction.get_connection().execute(
      templates::EdgeUpdate {
        graph_name: &graph_name,
      }
      .render()?
      .as_str(),
      named_params! {
        ":key": edge.key,
        ":labels": serde_json::to_string(&edge.labels)?,
        ":properties": serde_json::to_string(&edge.properties)?
      },
    )?;
    Ok(())
  }
  fn select_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: store::SelectEdgeQuery,
    directivity: crate::graph::EdgeDirectivity,
  ) -> Result<Vec<store::EdgeResult>>
  {
    if query.source.is_select_none() || query.destination.is_select_none()
    {
      return Ok(Default::default());
    }
    let (is_undirected, table_suffix) = match directivity
    {
      graph::EdgeDirectivity::Directed => (false, ""),
      graph::EdgeDirectivity::Undirected => (true, "_undirected"),
    };
    let mut prepared_query = transaction.get_connection().prepare(
      templates::EdgeSelect {
        graph_name: graph_name,
        is_undirected,
        table_suffix: table_suffix,
        has_edge_keys: query.keys.is_some(),
        has_edge_labels: query.labels.is_some(),
        has_edge_properties: query.properties.is_some(),
        has_n_left_keys: query.source.keys.is_some(),
        has_n_left_labels: query.source.labels.is_some(),
        has_n_left_properties: query.source.properties.is_some(),
        has_n_right_keys: query.destination.keys.is_some(),
        has_n_right_labels: query.destination.labels.is_some(),
        has_n_right_properties: query.destination.properties.is_some(),
      }
      .render()?
      .as_str(),
    )?;

    let mut bindings = Vec::<(&'static str, String)>::new();

    // Edge queries
    if let Some(keys) = query.keys
    {
      let hex_keys = keys.iter().map(|key| hex(&key)).collect::<Vec<_>>();
      bindings.push((":edge_keys", serde_json::to_string(&hex_keys)?));
    }
    if let Some(labels) = query.labels
    {
      bindings.push((":edge_labels", serde_json::to_string(&labels)?));
    }
    if let Some(properties) = query.properties
    {
      bindings.push((":edge_properties", serde_json::to_string(&properties)?));
    }

    // Left queries
    if let Some(keys) = query.source.keys
    {
      let hex_keys = keys.iter().map(|key| hex(&key)).collect::<Vec<_>>();
      bindings.push((":n_left_keys", serde_json::to_string(&hex_keys)?));
    }
    if let Some(labels) = query.source.labels
    {
      bindings.push((":n_left_labels", serde_json::to_string(&labels)?));
    }
    if let Some(properties) = query.source.properties
    {
      bindings.push((":n_left_properties", serde_json::to_string(&properties)?));
    }

    // Right queries
    if let Some(keys) = query.destination.keys
    {
      let hex_keys = keys.iter().map(|key| hex(&key)).collect::<Vec<_>>();
      bindings.push((":n_right_keys", serde_json::to_string(&hex_keys)?));
    }
    if let Some(labels) = query.destination.labels
    {
      bindings.push((":n_right_labels", serde_json::to_string(&labels)?));
    }
    if let Some(properties) = query.destination.properties
    {
      bindings.push((":n_right_properties", serde_json::to_string(&properties)?));
    }

    // Execute query
    let mut it = prepared_query.query(
      bindings
        .iter()
        .map(|(k, v)| (*k, v as &dyn rusqlite::ToSql))
        .collect::<Vec<_>>()
        .as_slice(),
    )?;

    let mut edges: Vec<EdgeResult> = Default::default();
    let mut edges_keys: HashSet<u128> = Default::default();
    while let Some(row) = it.next()?
    {
      let edge_key: graph::Key = row.get(0)?;
      let n_left_key = row.get(4)?;
      let n_right_key = row.get(7)?;

      // This ensure that if (a)-[]->(a) the edge is returned only once. But matching [a]-[]-[b] return the edge twice.
      if n_left_key == n_right_key && edges_keys.contains(&edge_key.uuid)
      {
        continue;
      }
      edges_keys.insert(edge_key.uuid);
      let edge_labels = serde_json::from_str(&row.get::<_, String>(1)?)?;
      let edge_properties = serde_json::from_str(&row.get::<_, String>(2)?)?;

      let n_left_labels = serde_json::from_str(&row.get::<_, String>(5)?)?;
      let n_left_properties = serde_json::from_str(&row.get::<_, String>(6)?)?;

      let n_right_labels = serde_json::from_str(&row.get::<_, String>(8)?)?;
      let n_right_properties = serde_json::from_str(&row.get::<_, String>(9)?)?;

      let source = graph::Node {
        key: n_left_key,
        labels: n_left_labels,
        properties: n_left_properties,
      };
      let destination = graph::Node {
        key: n_right_key,
        labels: n_right_labels,
        properties: n_right_properties,
      };
      let reversed = row.get::<_, u32>(3)? == 1;
      let (source, destination) = if reversed
      {
        (destination, source)
      }
      else
      {
        (source, destination)
      };

      edges.push(EdgeResult {
        edge: graph::Edge {
          key: edge_key,
          labels: edge_labels,
          properties: edge_properties,
          source,
          destination,
        },
        reversed,
      });
    }
    Ok(edges)
  }
  fn compute_statistics(&self, transaction: &mut Self::TransactionBox)
    -> Result<store::Statistics>
  {
    let (nodes_count, edges_count, labels_nodes_count, properties_count) =
      transaction.get_connection().query_row(
        templates::CallStats {
          graph_name: &"default".to_string(),
        }
        .render()?
        .as_str(),
        (),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
      )?;
    Ok(store::Statistics {
      nodes_count,
      edges_count,
      labels_nodes_count,
      properties_count,
    })
  }
}

#[cfg(test)]
mod tests
{
  use crate::{
    prelude::*,
    store::{Store, TransactionBoxable},
  };
  #[test]
  fn test_hex()
  {
    assert_eq!(
      super::hex(&graph::Key {
        uuid: 18580062510968287067562660977870108180
      }),
      "0DFA63CEE7484B0DBFC407697F77F614"
    );
    assert_eq!(
      super::hex(&graph::Key { uuid: 0 }),
      "00000000000000000000000000000000"
    );
  }
  #[test]
  fn test_sqlite_metadata()
  {
    let temp_file = crate::tests::create_tmp_file();
    let store = super::Store::new(temp_file.path()).unwrap();
    let mut tx = store.begin_read().unwrap();
    let version: utils::Version = store.get_metadata_value_json(&mut tx, "version").unwrap();
    assert_eq!(version.major, consts::GQLITE_VERSION.major);
    assert_eq!(version.minor, consts::GQLITE_VERSION.minor);
    assert_eq!(version.patch, consts::GQLITE_VERSION.patch);
    tx.close().unwrap();
    drop(store);

    // Try to reopen
    let store = super::Store::new(temp_file.path()).unwrap();
    let mut tx = store.begin_read().unwrap();
    let version: utils::Version = store.get_metadata_value_json(&mut tx, "version").unwrap();
    assert_eq!(version.major, consts::GQLITE_VERSION.major);
    assert_eq!(version.minor, consts::GQLITE_VERSION.minor);
    assert_eq!(version.patch, consts::GQLITE_VERSION.patch);
    tx.close().unwrap();
    drop(store);
  }
}
