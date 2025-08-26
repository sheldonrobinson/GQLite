#[cfg(feature = "_pgql")]
pub(crate) mod pgql;
#[cfg(feature = "redb")]
pub(crate) mod redb;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

#[cfg(feature = "_pgql")]
pub(crate) use pgql::Store;

use crate::prelude::*;

//  ____  _        _   _     _   _
// / ___|| |_ __ _| |_(_)___| |_(_) ___ ___
// \___ \| __/ _` | __| / __| __| |/ __/ __|
//  ___) | || (_| | |_| \__ \ |_| | (__\__ \
// |____/ \__\__,_|\__|_|___/\__|_|\___|___/

pub(crate) struct Statistics
{
  pub nodes_count: usize,
  pub edges_count: usize,
  pub labels_nodes_count: usize,
  pub properties_count: usize,
}

pub(crate) trait ReadTransaction
{
  fn discard(self) -> Result<()>;
}

pub(crate) trait WriteTransaction: ReadTransaction
{
  // Commit.
  fn commit(self) -> Result<()>;
}

/// Box that holds a Read or a Write transaction for a store.
pub(crate) enum TransactionBox<TRead, TWrite>
where
  TRead: ReadTransaction,
  TWrite: WriteTransaction,
{
  Read(TRead),
  Write(TWrite),
}

impl<TRead, TWrite> TransactionBox<TRead, TWrite>
where
  TRead: ReadTransaction,
  TWrite: WriteTransaction,
{
  pub(crate) fn from_read(read: TRead) -> Self
  {
    Self::Read(read)
  }
  pub(crate) fn from_write(write: TWrite) -> Self
  {
    Self::Write(write)
  }
  pub(crate) fn try_into_write(&mut self) -> Result<&mut TWrite>
  {
    match self
    {
      Self::Read(_) => Err(InternalError::NotWriteTransaction.into()),
      Self::Write(write) => Ok(write),
    }
  }
}

/// Trait that represent a box that can contain a read or write transaction for a store.
pub(crate) trait TransactionBoxable
{
  type ReadTransaction: ReadTransaction;
  type WriteTransaction: WriteTransaction;

  fn close(self) -> Result<()>;
}

impl<TRead, TWrite> TransactionBoxable for TransactionBox<TRead, TWrite>
where
  TRead: ReadTransaction,
  TWrite: WriteTransaction,
{
  type ReadTransaction = TRead;
  type WriteTransaction = TWrite;

  fn close(self) -> Result<()>
  {
    match self
    {
      Self::Read(read) => read.discard(),
      Self::Write(write) => write.commit(),
    }
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

pub(crate) trait Store
{
  type TransactionBox: TransactionBoxable;
  fn begin_read(&self) -> Result<Self::TransactionBox>;
  fn begin_write(&self) -> Result<Self::TransactionBox>;
  /// List the graphs
  fn graphs_list(&self, transaction: &mut Self::TransactionBox) -> Result<Vec<String>>;
  /// Create a new graph
  fn create_graph(
    &self,
    transaction: &mut Self::TransactionBox,
    name: impl AsRef<str>,
    ignore_if_exists: bool,
  ) -> Result<()>;
  /// Delete a graph
  fn drop_graph(
    &self,
    transaction: &mut Self::TransactionBox,
    name: impl AsRef<str>,
    if_exists: bool,
  ) -> Result<()>;
  /// Create nodes and add them to a graph
  fn create_nodes<'a, T: IntoIterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    nodes_iter: T,
  ) -> Result<()>;
  /// Create nodes and add them to a graph
  fn update_node(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    node: &graph::Node,
  ) -> Result<()>;
  /// Delete nodes according to a given query
  fn delete_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: SelectNodeQuery,
    detach: bool,
  ) -> Result<()>;
  /// Select nodes according to a given query
  fn select_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>;
  /// Add edge
  fn create_edges<'a, T: IntoIterator<Item = &'a crate::graph::SinglePath>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    edges_iter: T,
  ) -> Result<()>;
  fn update_edge(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    edge: &graph::Edge,
  ) -> Result<()>;
  /// Delete nodes according to a given query
  fn delete_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<()>;
  /// Select edges
  fn select_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<Vec<EdgeResult>>;
  /// Compute store statistics
  fn compute_statistics(&self, transaction: &mut Self::TransactionBox) -> Result<Statistics>;
}

//  _____    _            ____                 _ _
// | ____|__| | __ _  ___|  _ \ ___  ___ _   _| | |_
// |  _| / _` |/ _` |/ _ \ |_) / _ \/ __| | | | | __|
// | |__| (_| | (_| |  __/  _ <  __/\__ \ |_| | | |_
// |_____\__,_|\__, |\___|_| \_\___||___/\__,_|_|\__|
//             |___/

pub(crate) struct EdgeResult
{
  pub(crate) path: graph::SinglePath,
  pub(crate) reversed: bool,
}

//  ____       _           _   _   _           _       ___
// / ___|  ___| | ___  ___| |_| \ | | ___   __| | ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  \| |/ _ \ / _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |\  | (_) | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_| \_|\___/ \__,_|\___|\__\_\\__,_|\___|_|   \__, |
//                                                                          |___/

#[derive(Debug, Clone)]
pub(crate) struct SelectNodeQuery
{
  keys: Option<Vec<graph::Key>>,
  labels: Option<Vec<String>>,
  properties: Option<value::ValueMap>,
  select_all: bool,
}

impl SelectNodeQuery
{
  fn is_select_all(&self) -> bool
  {
    self.select_all
  }
  fn is_select_none(&self) -> bool
  {
    self.keys.is_none() && self.labels.is_none() && self.properties.is_none() && !self.select_all
  }
  pub(crate) fn is_select_only_keys(&self) -> bool
  {
    self.labels.is_none() && self.properties.is_none() && !self.select_all
  }
  pub(crate) fn select_all() -> Self
  {
    Self {
      keys: None,
      labels: None,
      properties: None,
      select_all: true,
    }
  }
  pub(crate) fn select_none() -> Self
  {
    Self {
      keys: None,
      labels: None,
      properties: None,
      select_all: false,
    }
  }
  #[allow(unused)]
  pub(crate) fn select_keys(keys: impl Into<Vec<graph::Key>>) -> Self
  {
    Self {
      keys: Some(keys.into()),
      labels: None,
      properties: None,
      select_all: false,
    }
  }
  #[allow(dead_code)]
  pub(crate) fn select_labels(labels: impl Into<Vec<String>>) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels.into()),
      properties: None,
      select_all: false,
    }
  }
  pub(crate) fn select_labels_properties(
    labels: impl Into<Vec<String>>,
    properties: value::ValueMap,
  ) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels.into()),
      properties: Some(properties),
      select_all: false,
    }
  }
  pub(crate) fn is_match(&self, node: &graph::Node) -> bool
  {
    if self.select_all
    {
      return true;
    }
    if let Some(keys) = &self.keys
    {
      if !keys.iter().any(|x| node.key() == *x)
      {
        return false;
      }
    }
    if let Some(labels) = &self.labels
    {
      if !labels.iter().all(|x| node.labels().contains(x))
      {
        return false;
      }
    }
    if let Some(properties) = &self.properties
    {
      if !properties
        .iter()
        .all(|(k, v)| node.properties().get(k) == Some(v))
      {
        return false;
      }
    }
    true
  }
}

//  ____       _           _   _____    _             ___
// / ___|  ___| | ___  ___| |_| ____|__| | __ _  ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  _| / _` |/ _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |__| (_| | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_____\__,_|\__, |\___|\__\_\\__,_|\___|_|   \__, |
//                                        |___/                            |___/

#[derive(Debug, Clone)]
pub(crate) struct SelectEdgeQuery
{
  keys: Option<Vec<graph::Key>>,
  labels: Option<Vec<String>>,
  properties: Option<value::ValueMap>,
  source: SelectNodeQuery,
  destination: SelectNodeQuery,
}

impl SelectEdgeQuery
{
  #[allow(dead_code)]
  pub(crate) fn is_select_only_keys(&self) -> bool
  {
    self.keys.is_some()
      && self.labels.is_none()
      && self.properties.is_none()
      && self.source.select_all
      && self.destination.select_all
  }

  pub(crate) fn select_all() -> Self
  {
    Self {
      keys: None,
      labels: None,
      properties: None,
      source: SelectNodeQuery::select_all(),
      destination: SelectNodeQuery::select_all(),
    }
  }
  pub(crate) fn select_none() -> Self
  {
    Self {
      keys: None,
      labels: None,
      properties: None,
      source: SelectNodeQuery::select_none(),
      destination: SelectNodeQuery::select_none(),
    }
  }
  #[allow(unused)]
  pub(crate) fn select_keys(keys: impl Into<Vec<graph::Key>>) -> Self
  {
    Self {
      keys: Some(keys.into()),
      labels: None,
      properties: None,
      source: SelectNodeQuery::select_all(),
      destination: SelectNodeQuery::select_all(),
    }
  }
  pub(crate) fn select_source_keys(source_query: SelectNodeQuery) -> Self
  {
    Self {
      keys: None,
      labels: None,
      properties: None,
      source: source_query,
      destination: SelectNodeQuery::select_all(),
    }
  }
  pub(crate) fn select_source_destination_keys(
    source_query: SelectNodeQuery,
    keys: impl Into<Vec<graph::Key>>,
    destination_query: SelectNodeQuery,
  ) -> Self
  {
    Self {
      keys: Some(keys.into()),
      labels: None,
      properties: None,
      source: source_query,
      destination: destination_query,
    }
  }
  pub(crate) fn select_source_destination_labels_properties(
    source_query: SelectNodeQuery,
    labels: impl Into<Vec<String>>,
    properties: value::ValueMap,
    destination_query: SelectNodeQuery,
  ) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels.into()),
      properties: Some(properties),
      source: source_query,
      destination: destination_query,
    }
  }
  pub(crate) fn is_match(&self, edge: &graph::Path) -> bool
  {
    if let Some(keys) = &self.keys
    {
      if !keys.iter().any(|x| edge.key() == *x)
      {
        return false;
      }
    }
    if let Some(labels) = &self.labels
    {
      if !labels.iter().all(|x| edge.labels().contains(x))
      {
        return false;
      }
    }
    if let Some(properties) = &self.properties
    {
      if !properties
        .iter()
        .all(|(k, v)| edge.properties().get(k) == Some(v))
      {
        return false;
      }
    }
    self.source.is_match(edge.source()) && self.destination.is_match(edge.destination())
  }
}
