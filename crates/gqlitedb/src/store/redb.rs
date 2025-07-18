use ccutils::sync::ArcRwLock;
use redb::{ReadableTable, ReadableTableMetadata};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use crate::{prelude::*, store::TransactionBoxable};

//  ____               _     _             _   _____    _
// |  _ \ ___ _ __ ___(_)___| |_ ___ _ __ | |_| ____|__| | __ _  ___
// | |_) / _ \ '__/ __| / __| __/ _ \ '_ \| __|  _| / _` |/ _` |/ _ \
// |  __/  __/ |  \__ \ \__ \ ||  __/ | | | |_| |__| (_| | (_| |  __/
// |_|   \___|_|  |___/_|___/\__\___|_| |_|\__|_____\__,_|\__, |\___|
//                                                        |___/

/// This structure is used to represent the internal storage of an edge.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct PersistentEdge
{
  pub key: graph::Key,
  pub source: graph::Key,
  pub destination: graph::Key,
  pub labels: Vec<String>,
  pub properties: value::ValueMap,
}

impl redb::Value for PersistentEdge
{
  type AsBytes<'a> = Vec<u8>;
  type SelfType<'a> = PersistentEdge;
  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    let mut data = Vec::<u8>::new();
    ciborium::into_writer(value, &mut data).unwrap(); // This unwrap should not happen, unless there is a bug
    data
  }
  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    ciborium::from_reader(data).unwrap() // This unwrap should not happen, unless there is a bug
  }
  fn fixed_width() -> Option<usize>
  {
    None
  }
  fn type_name() -> redb::TypeName
  {
    redb::TypeName::new("PersistentEdge")
  }
}

impl redb::Value for graph::Node
{
  type AsBytes<'a> = Vec<u8>;
  type SelfType<'a> = graph::Node;
  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    let mut data = Vec::<u8>::new();
    ciborium::into_writer(value, &mut data).unwrap(); // This unwrap should not happen, unless there is a bug
    data
  }
  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    ciborium::from_reader(data).unwrap() // This unwrap should not happen, unless there is a bug
  }
  fn fixed_width() -> Option<usize>
  {
    None
  }
  fn type_name() -> redb::TypeName
  {
    redb::TypeName::new("graph::Node")
  }
}

impl redb::Value for graph::Key
{
  type AsBytes<'a> = <u128 as redb::Value>::AsBytes<'a>;
  type SelfType<'a> = graph::Key;
  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    u128::as_bytes(&value.uuid)
  }
  fn fixed_width() -> Option<usize>
  {
    u128::fixed_width()
  }
  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    graph::Key {
      uuid: u128::from_bytes(data),
    }
  }
  fn type_name() -> redb::TypeName
  {
    redb::TypeName::new("gqlite::graph::Key")
  }
}

impl redb::Key for graph::Key
{
  fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering
  {
    u128::compare(data1, data2)
  }
}

struct EdgeIdResult
{
  edge_data: PersistentEdge,
  reversed: Option<bool>,
  source_id: Option<graph::Key>,
  destination_id: Option<graph::Key>,
}

impl EdgeIdResult
{
  fn new(
    edge_data: PersistentEdge,
    reversed: Option<bool>,
    source_id: Option<graph::Key>,
    destination_id: Option<graph::Key>,
  ) -> EdgeIdResult
  {
    assert!(reversed.is_some() || source_id.is_some() || destination_id.is_some());
    EdgeIdResult {
      edge_data,
      reversed,
      source_id,
      destination_id,
    }
  }
  fn is_reversed(&self) -> bool
  {
    match self.reversed
    {
      Some(v) => v,
      None => match self.source_id
      {
        Some(v) => self.edge_data.destination == v,
        None => match self.destination_id
        {
          Some(v) => self.edge_data.source == v,
          None => panic!("is_reversed"),
        },
      },
    }
  }
}

//  _____     _     _
// |_   _|_ _| |__ | | ___
//   | |/ _` | '_ \| |/ _ \
//   | | (_| | |_) | |  __/
//   |_|\__,_|_.__/|_|\___|

trait NodeTableExtension
{
  fn get_node(&self, key: graph::Key) -> Result<graph::Node>;
}

impl<'txn> NodeTableExtension for redb::Table<'txn, graph::Key, &[u8]>
{
  fn get_node(&self, key: graph::Key) -> Result<graph::Node>
  {
    let v = self.get_required(key, || InternalError::UnknownNode)?;
    let c = ciborium::from_reader::<graph::Node, &[u8]>(&mut v.value())?;
    Ok::<graph::Node, crate::prelude::ErrorType>(c)
  }
}

trait TableExtension<K, V>
where
  K: redb::Key + 'static,
  V: redb::Value + 'static,
{
  fn get_required<'a, TError: Into<Error>>(
    &self,
    key: impl std::borrow::Borrow<K::SelfType<'a>>,
    f: impl FnOnce() -> TError,
  ) -> Result<redb::AccessGuard<V>>;
}

impl<'txn, K, V, T> TableExtension<K, V> for T
where
  T: ReadableTable<K, V>,
  K: redb::Key + 'static,
  V: redb::Value + 'static,
{
  fn get_required<'a, TError: Into<ErrorType>>(
    &self,
    key: impl std::borrow::Borrow<K::SelfType<'a>>,
    f: impl FnOnce() -> TError,
  ) -> Result<redb::AccessGuard<V>>
  {
    self.get(key)?.ok_or_else(|| f().into())
  }
}

//   ____                 _     ___        __
//  / ___|_ __ __ _ _ __ | |__ |_ _|_ __  / _| ___
// | |  _| '__/ _` | '_ \| '_ \ | || '_ \| |_ / _ \
// | |_| | | | (_| | |_) | | | || || | | |  _| (_) |
//  \____|_|  \__,_| .__/|_| |_|___|_| |_|_|  \___/
//                 |_|

#[derive(Debug)]
struct GraphInfo
{
  name: String,
  nodes_table: String,
  edges_table: String,
  edges_source_index: String,
  edges_destination_index: String,
}

impl GraphInfo
{
  fn new(name: impl Into<String>) -> GraphInfo
  {
    let name = name.into();
    let nodes_table = format!("__{}__nodes_table", name);
    let edges_table = format!("__{}__edges_table", name);
    let edges_source_index = format!("__{}__edges_source_index", name);
    let edges_destination_index = format!("__{}__edges_destination_index", name);

    GraphInfo {
      name,
      nodes_table,
      edges_table,
      edges_source_index,
      edges_destination_index,
    }
  }
  fn nodes_table_definition<'a>(&'a self) -> redb::TableDefinition<'a, graph::Key, graph::Node>
  {
    redb::TableDefinition::new(&self.nodes_table)
  }
  fn edges_table_definition<'a>(&'a self) -> redb::TableDefinition<'a, graph::Key, PersistentEdge>
  {
    redb::TableDefinition::new(&self.edges_table)
  }
  fn edges_source_index_definition<'a>(
    &'a self,
  ) -> redb::TableDefinition<'a, graph::Key, Vec<graph::Key>>
  {
    redb::TableDefinition::new(&self.edges_source_index)
  }
  fn edges_destination_index_definition<'a>(
    &'a self,
  ) -> redb::TableDefinition<'a, graph::Key, Vec<graph::Key>>
  {
    redb::TableDefinition::new(&self.edges_destination_index)
  }
}

//  _____                          _   _
// |_   _| __ __ _ _ __  ___  __ _| |_(_) ___  _ __
//   | || '__/ _` | '_ \/ __|/ _` | __| |/ _ \| '_ \
//   | || | | (_| | | | \__ \ (_| | |_| | (_) | | | |
//   |_||_|  \__,_|_| |_|___/\__,_|\__|_|\___/|_| |_|

impl super::ReadTransaction for redb::ReadTransaction
{
  fn discard(self) -> Result<()>
  {
    Ok(self.close()?)
  }
}

impl super::ReadTransaction for redb::WriteTransaction
{
  fn discard(self) -> Result<()>
  {
    Ok(self.abort()?)
  }
}

impl super::WriteTransaction for redb::WriteTransaction
{
  fn commit(self) -> Result<()>
  {
    Ok(self.commit()?)
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

/// Storage, aka, interface to the underlying redb store.
pub(crate) struct Store
{
  redb_store: redb::Database,
  graphs: ArcRwLock<HashMap<String, Arc<GraphInfo>>>,
}

ccutils::assert_impl_all!(Store: Sync, Send);

type TransactionBox = store::TransactionBox<redb::ReadTransaction, redb::WriteTransaction>;

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    let s = Self {
      redb_store: redb::Database::create(path.as_ref())?,
      graphs: Default::default(),
    };
    use crate::store::Store;
    let mut tx = s.begin_write()?;
    s.create_graph(&mut tx, &"default".to_string(), true)?;
    s.set_metadata_value(&mut tx, "version", &consts::GQLITE_VERSION)?;
    tx.close()?;
    Ok(s)
  }
  fn get_metadata_from_table<TTable, TValue>(
    &self,
    table: TTable,
    key: impl Into<String>,
  ) -> Result<TValue>
  where
    TTable: ReadableTable<String, Vec<u8>>,
    TValue: for<'a> Deserialize<'a>,
  {
    let key = key.into();
    let value = table
      .get(&key)?
      .ok_or_else(|| InternalError::MissingMetadata { key: key })?;
    Ok(ciborium::from_reader(value.value().as_slice())?)
  }
  fn get_metadata_value<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
  ) -> Result<T>
  {
    let table_def = redb::TableDefinition::new("gqlite_metadata");
    match transaction
    {
      TransactionBox::Read(read) =>
      {
        self.get_metadata_from_table(read.open_table(table_def)?, key.into())
      }
      TransactionBox::Write(write) =>
      {
        self.get_metadata_from_table(write.open_table(table_def)?, key.into())
      }
    }
  }
  fn get_metadata_value_or_else_from_table<TTable, TValue>(
    &self,
    table: TTable,
    key: impl Into<String>,
    f: impl FnOnce() -> TValue,
  ) -> Result<TValue>
  where
    TTable: ReadableTable<String, Vec<u8>>,
    TValue: for<'a> Deserialize<'a>,
  {
    let key = key.into();
    let value = table
      .get(&key)?
      .map(|r| Ok::<_, ErrorType>(ciborium::from_reader(r.value().as_slice())?))
      .unwrap_or_else(|| Ok(f()))?;
    Ok(value)
  }
  fn get_metadata_value_or_else<T: for<'a> Deserialize<'a>>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    f: impl FnOnce() -> T,
  ) -> Result<T>
  {
    let table_def = redb::TableDefinition::new("gqlite_metadata");
    match transaction
    {
      TransactionBox::Read(read) =>
      {
        self.get_metadata_value_or_else_from_table(read.open_table(table_def)?, key.into(), f)
      }
      TransactionBox::Write(write) =>
      {
        self.get_metadata_value_or_else_from_table(write.open_table(table_def)?, key.into(), f)
      }
    }
  }
  fn set_metadata_value<T: Serialize>(
    &self,
    transaction: &mut TransactionBox,
    key: impl Into<String>,
    value: &T,
  ) -> Result<()>
  {
    let tx = transaction.try_into_write()?;
    let mut metadata_table = tx.open_table(redb::TableDefinition::<'_, String, Vec<u8>>::new(
      "gqlite_metadata",
    ))?;
    let key = key.into();
    let mut data = Vec::<u8>::new();
    ciborium::into_writer(value, &mut data)?;
    metadata_table.insert(&key, data)?;
    Ok(())
  }
  fn get_graph_info(&self, graph_name: &String) -> Result<Arc<GraphInfo>>
  {
    let graphs = self.graphs.read()?;
    let graph_info = graphs.get(graph_name);
    match graph_info
    {
      Some(graph_info) => Ok(graph_info.clone()),
      None =>
      {
        drop(graphs);
        let graph_info = Arc::new(GraphInfo::new(graph_name));
        self
          .graphs
          .write()?
          .insert(graph_name.to_owned(), graph_info.clone());
        Ok(graph_info)
      }
    }
  }
  fn select_nodes_from_table<'txn, T>(
    &self,
    nodes_table: &'txn T,
    query: super::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  where
    T: ReadableTable<graph::Key, graph::Node>,
  {
    let r = match query.keys
    {
      Some(keys) => Box::new(keys.into_iter().map(|key| {
        Ok(
          nodes_table
            .get_required(key, || InternalError::UnknownNode)?
            .value(),
        )
      })) as Box<dyn Iterator<Item = Result<graph::Node>>>,
      None => Box::new({
        nodes_table.range::<graph::Key>(..)?.into_iter().map(|r| {
          let (_, v) = r?;
          Ok(v.value())
        })
      }) as Box<dyn Iterator<Item = Result<graph::Node>>>,
    };
    let r = match query.labels
    {
      Some(labels) => Box::new(r.filter(move |n| match n
      {
        Ok(n) =>
        {
          for l in labels.iter()
          {
            if !n.labels.contains(l)
            {
              return false;
            }
          }
          true
        }
        Err(_) => true,
      })) as Box<dyn Iterator<Item = Result<crate::graph::Node>>>,
      None => Box::new(r) as Box<dyn Iterator<Item = Result<crate::graph::Node>>>,
    };
    let r = match query.properties
    {
      Some(properties) => Box::new(r.filter(move |n| match n
      {
        Ok(n) =>
        {
          for (k, v) in properties.iter()
          {
            match n.properties.get(k)
            {
              Some(val) =>
              {
                if val != v
                {
                  return false;
                }
              }
              None =>
              {
                return false;
              }
            }
          }
          true
        }
        Err(_) => true,
      })) as Box<dyn Iterator<Item = Result<crate::graph::Node>>>,
      None => Box::new(r) as Box<dyn Iterator<Item = Result<crate::graph::Node>>>,
    };
    r.collect()
  }

  fn select_edges_from_tables<TEdges, TNodes, TEdgesIndex>(
    &self,
    query: super::SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
    edges_table: TEdges,
    nodes_table: TNodes,
    edges_source_index: Rc<RefCell<TEdgesIndex>>,
    edges_destination_index: Rc<RefCell<TEdgesIndex>>,
  ) -> Result<Vec<super::EdgeResult>>
  where
    TEdges: ReadableTable<graph::Key, PersistentEdge>,
    TNodes: ReadableTable<graph::Key, graph::Node>,
    TEdgesIndex: ReadableTable<graph::Key, Vec<graph::Key>>,
  {
    let edges_uuid_indices = match directivity
    {
      graph::EdgeDirectivity::Directed => vec![(edges_source_index, edges_destination_index)],
      graph::EdgeDirectivity::Undirected => vec![
        (edges_source_index.clone(), edges_destination_index.clone()),
        (edges_destination_index, edges_source_index),
      ],
    };

    // Get the UUID of the edges
    let mut edges_raw = Vec::<EdgeIdResult>::new();

    match &query.keys
    {
      Some(keys) =>
      {
        for key in keys.into_iter()
        {
          edges_raw.push(EdgeIdResult::new(
            edges_table
              .get(key)?
              .ok_or_else(|| InternalError::UnknownNode)?
              .value(),
            Some(false),
            None,
            None,
          ));
        }
      }
      None =>
      {
        if query.source.is_select_all() && query.destination.is_select_all()
        {
          edges_raw = edges_table
            .range::<graph::Key>(..)?
            .into_iter()
            .map(|r| {
              let (_, v) = r?;
              Ok(EdgeIdResult::new(v.value(), Some(false), None, None))
            })
            .collect::<Result<Vec<EdgeIdResult>>>()?
        }
        else
        {
          let mut edge_ids = Vec::<(graph::Key, Option<graph::Key>, Option<graph::Key>)>::new();
          for (edges_source_uuid_index, edges_destination_uuid_index) in edges_uuid_indices
          {
            if !query.destination.is_select_all() && !query.source.is_select_all()
            {
              let dest_it =
                self.select_nodes_from_table(&nodes_table, query.destination.clone())?;
              let dest_it: Vec<graph::Key> = dest_it
                .into_iter()
                .map(|n| {
                  edges_destination_uuid_index
                    .borrow()
                    .get_required(n.key, || InternalError::UnknownNode)
                    .map(|x| x.value())
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect();
              let nodes = self.select_nodes_from_table(&nodes_table, query.source.clone())?;
              for n in nodes.iter()
              {
                let nkey = n.key;
                for k in edges_source_uuid_index
                  .borrow()
                  .get_required(nkey, || InternalError::UnknownNode)?
                  .value()
                  .iter()
                {
                  if dest_it.contains(k)
                  {
                    let uniq_k = (k.to_owned(), Some(nkey), None);
                    if !edge_ids.contains(&uniq_k)
                    {
                      edges_raw.push(EdgeIdResult::new(
                        edges_table
                          .get_required(k, || InternalError::UnknownEdge)?
                          .value(),
                        None,
                        Some(nkey),
                        None,
                      ));
                      edge_ids.push(uniq_k);
                    }
                  }
                }
              }
            }
            else if !query.source.is_select_all()
            {
              let nodes = self.select_nodes_from_table(&nodes_table, query.source.clone())?;

              for n in nodes.into_iter()
              {
                let nkey = n.key;
                for k in edges_source_uuid_index
                  .borrow()
                  .get_required(nkey, || InternalError::UnknownNode)?
                  .value()
                  .iter()
                {
                  let uniq_k = (k.to_owned(), Some(nkey), None);
                  if !edge_ids.contains(&uniq_k)
                  {
                    edges_raw.push(EdgeIdResult::new(
                      edges_table
                        .get_required(k, || InternalError::UnknownEdge)?
                        .value(),
                      None,
                      Some(nkey),
                      None,
                    ));
                    edge_ids.push(uniq_k);
                  }
                }
              }
            }
            else
            {
              let nodes = self.select_nodes_from_table(&nodes_table, query.destination.clone())?;
              for n in nodes.into_iter()
              {
                let nkey = n.key;
                for k in edges_destination_uuid_index
                  .borrow()
                  .get_required(nkey, || InternalError::UnknownNode)?
                  .value()
                  .iter()
                {
                  let uniq_k = (k.to_owned(), None, Some(nkey));
                  if !edge_ids.contains(&uniq_k)
                  {
                    edges_raw.push(EdgeIdResult::new(
                      edges_table
                        .get_required(k, || InternalError::UnknownEdge)?
                        .value(),
                      None,
                      None,
                      Some(nkey),
                    ));
                    edge_ids.push(uniq_k);
                  }
                }
              }
            }
          }
        }
      }
    }

    // Get the edges
    let r = edges_raw.into_iter().map(|v| {
      Ok::<super::EdgeResult, crate::prelude::ErrorType>({
        let reversed = v.is_reversed();
        let edge = v.edge_data;

        let source = nodes_table
          .get_required(edge.source, || InternalError::UnknownNode)?
          .value();
        let destination = nodes_table
          .get_required(edge.destination, || InternalError::UnknownNode)?
          .value();

        let edge = graph::Edge {
          key: edge.key,
          source,
          destination,
          labels: edge.labels,
          properties: edge.properties,
        };
        super::EdgeResult { edge, reversed }
      })
    });
    // Filter using the labels
    let r = match &query.labels
    {
      Some(labels) => Box::new(r.filter(move |e| match e
      {
        Ok(e) =>
        {
          for l in labels.iter()
          {
            if !e.edge.labels.contains(l)
            {
              return false;
            }
          }
          true
        }
        Err(_) => true,
      })) as Box<dyn Iterator<Item = Result<super::EdgeResult>>>,
      None => Box::new(r) as Box<dyn Iterator<Item = Result<super::EdgeResult>>>,
    };
    let r = match &query.properties
    {
      Some(properties) => Box::new(r.filter(move |e| match e
      {
        Ok(e) =>
        {
          for (k, v) in properties.iter()
          {
            match e.edge.properties.get(k)
            {
              Some(val) =>
              {
                if val != v
                {
                  return false;
                }
              }
              None =>
              {
                return false;
              }
            }
          }
          true
        }
        Err(_) => true,
      })) as Box<dyn Iterator<Item = Result<super::EdgeResult>>>,
      None => Box::new(r) as Box<dyn Iterator<Item = Result<super::EdgeResult>>>,
    };
    if query.keys.is_some() && (!query.source.is_select_all() || !query.destination.is_select_all())
    {
      r.filter(|e| {
        if let Ok(e) = &e
        {
          query.is_match(&e.edge)
        }
        else
        {
          return true;
        }
      })
      .collect()
    }
    else
    {
      r.collect()
    }
  }
}

impl store::Store for Store
{
  type TransactionBox = TransactionBox;

  fn begin_write(&self) -> Result<Self::TransactionBox>
  {
    let s = self.redb_store.begin_write()?;
    Ok(Self::TransactionBox::from_write(s))
  }
  fn begin_read(&self) -> Result<Self::TransactionBox>
  {
    let s = self.redb_store.begin_read()?;
    Ok(Self::TransactionBox::from_read(s))
  }
  fn graphs_list(&self, transaction: &mut Self::TransactionBox) -> Result<Vec<String>>
  {
    self.get_metadata_value_or_else(transaction, "graphs".to_string(), || vec![])
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

    {
      let tx = transaction.try_into_write()?;

      let gi = Arc::new(GraphInfo::new(graph_name));
      tx.open_table(gi.nodes_table_definition())?;
      tx.open_table(gi.edges_table_definition())?;
      tx.open_table(gi.edges_source_index_definition())?;
      tx.open_table(gi.edges_destination_index_definition())?;

      self.graphs.write()?.insert(graph_name.to_owned(), gi);
    }
    graphs_list.push(graph_name.clone());
    self.set_metadata_value(transaction, "graphs", &graphs_list)?;

    Ok(())
  }
  fn delete_graph(&self, transaction: &mut Self::TransactionBox, graph_name: &String)
    -> Result<()>
  {
    let mut graphs_list = self.graphs_list(transaction)?;
    if graphs_list.contains(graph_name)
    {
      {
        let tx = transaction.try_into_write()?;
        let graph_info = self.get_graph_info(graph_name)?;
        tx.delete_table(graph_info.nodes_table_definition())?;
        tx.delete_table(graph_info.edges_table_definition())?;
        tx.delete_table(graph_info.edges_source_index_definition())?;
        tx.delete_table(graph_info.edges_destination_index_definition())?;
      }
      graphs_list.retain(|x| x != graph_name);
      self.set_metadata_value(transaction, "graphs", &graphs_list)?;

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
  /// Create nodes and add them to a graph
  fn create_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    nodes_iter: T,
  ) -> Result<()>
  {
    let graph_info = self.get_graph_info(graph_name)?;
    let transaction = transaction.try_into_write()?;
    let mut table = transaction.open_table(graph_info.nodes_table_definition())?;
    let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
    let mut table_destination =
      transaction.open_table(graph_info.edges_destination_index_definition())?;
    for x in nodes_iter
    {
      table.insert(x.key, x)?;
      table_source.insert(x.key, vec![])?;
      table_destination.insert(x.key, vec![])?;
    }
    Ok(())
  }
  /// Create nodes and add them to a graph
  fn update_node(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    node: &graph::Node,
  ) -> Result<()>
  {
    let graph_info = self.get_graph_info(graph_name)?;
    let transaction = transaction.try_into_write()?;
    let mut table = transaction.open_table(graph_info.nodes_table_definition())?;
    table.insert(node.key, node)?;
    Ok(())
  }
  /// Delete nodes according to a given query
  fn delete_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: super::SelectNodeQuery,
    detach: bool,
  ) -> Result<()>
  {
    let graph_info = self.get_graph_info(graph_name)?;

    if query.is_select_all()
    {
      let write_transaction = transaction.try_into_write()?;
      if detach
      {
        write_transaction.delete_table(graph_info.edges_table_definition())?;
        write_transaction.delete_table(graph_info.edges_source_index_definition())?;
        write_transaction.delete_table(graph_info.edges_destination_index_definition())?;
        write_transaction.open_table(graph_info.edges_table_definition())?;
        write_transaction.open_table(graph_info.edges_source_index_definition())?;
        write_transaction.open_table(graph_info.edges_destination_index_definition())?;
      }
      else
      {
        let edge_table = write_transaction.open_table(graph_info.edges_table_definition())?;
        if edge_table.len()? > 0
        {
          return Err(error::RunTimeError::DeleteConnectedNode.into());
        }
      }
      write_transaction.delete_table(graph_info.nodes_table_definition())?;
      write_transaction.open_table(graph_info.nodes_table_definition())?;
    }
    else
    {
      let node_keys = if query.is_select_only_keys()
      {
        query
          .keys
          .ok_or_else(|| error::InternalError::Unreachable {
            context: "persy/store/delete_nodes",
          })?
      }
      else
      {
        self
          .select_nodes(transaction, graph_name, query)?
          .into_iter()
          .map(|x| x.key)
          .collect()
      };

      if detach
      {
        // Delete the edges connected to the nodes
        self.delete_edges(
          transaction,
          graph_name,
          super::SelectEdgeQuery::select_source_keys(super::SelectNodeQuery::select_keys(
            node_keys.clone(),
          )),
          graph::EdgeDirectivity::Undirected,
        )?;
      }
      else
      {
        let write_transaction = transaction.try_into_write()?;
        // Check if the nodes are disconnected
        let table_source =
          write_transaction.open_table(graph_info.edges_source_index_definition())?;
        let table_destination =
          write_transaction.open_table(graph_info.edges_destination_index_definition())?;

        for key in node_keys.iter()
        {
          if !table_source
            .get_required(key, || InternalError::UnknownNode)?
            .value()
            .is_empty()
            || !table_destination
              .get_required(key, || InternalError::UnknownNode)?
              .value()
              .is_empty()
          {
            return Err(error::RunTimeError::DeleteConnectedNode.into());
          }
        }
      }
      let write_transaction = transaction.try_into_write()?;
      // Delete the nodes
      let mut table_nodes = write_transaction.open_table(graph_info.nodes_table_definition())?;
      let mut table_source =
        write_transaction.open_table(graph_info.edges_source_index_definition())?;
      let mut table_destination =
        write_transaction.open_table(graph_info.edges_destination_index_definition())?;
      for key in node_keys.into_iter()
      {
        table_nodes.remove(key)?;
        table_source.remove(key)?;
        table_destination.remove(key)?;
      }
    }
    Ok(())
  }
  /// Select nodes according to a given query
  fn select_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: super::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
    let graph_info = self.get_graph_info(graph_name)?;
    match transaction
    {
      store::TransactionBox::Read(read) =>
      {
        let nodes_table = read.open_table(graph_info.nodes_table_definition())?;
        self.select_nodes_from_table(&nodes_table, query)
      }
      store::TransactionBox::Write(write) =>
      {
        let nodes_table = write.open_table(graph_info.nodes_table_definition())?;
        self.select_nodes_from_table(&nodes_table, query)
      }
    }
  }
  /// Add edge
  fn create_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    edges_iter: T,
  ) -> Result<()>
  {
    let transaction = transaction.try_into_write()?;
    let graph_info = self.get_graph_info(graph_name)?;
    let mut table = transaction.open_table(graph_info.edges_table_definition())?;
    let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
    let mut table_destination =
      transaction.open_table(graph_info.edges_destination_index_definition())?;

    for x in edges_iter
    {
      let mut keys_source = table_source
        .remove(x.source.key)?
        .ok_or(InternalError::UnknownNode)?
        .value();
      keys_source.push(x.key);
      let mut keys_destination = table_destination
        .remove(x.destination.key)?
        .ok_or(InternalError::UnknownNode)?
        .value();
      keys_destination.push(x.key);

      table.insert(
        x.key,
        &PersistentEdge {
          key: x.key,
          source: x.source.key,
          destination: x.destination.key,
          labels: x.labels.clone(),
          properties: x.properties.clone(),
        },
      )?;
      table_source.insert(x.source.key, keys_source)?;
      table_destination.insert(x.destination.key, keys_destination)?;
    }
    Ok(())
  }
  fn update_edge(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    edge: &graph::Edge,
  ) -> Result<()>
  {
    let transaction = transaction.try_into_write()?;
    let graph_info = self.get_graph_info(graph_name)?;
    let mut table = transaction.open_table(graph_info.edges_table_definition())?;
    table.insert(
      edge.key,
      &PersistentEdge {
        key: edge.key,
        source: edge.source.key,
        destination: edge.destination.key,
        labels: edge.labels.to_owned(),
        properties: edge.properties.to_owned(),
      },
    )?;
    Ok(())
  }
  /// Delete nodes according to a given query
  fn delete_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: super::SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<()>
  {
    let graph_info = self.get_graph_info(graph_name)?;
    let edges = self.select_edges(transaction, graph_name, query, directivity)?;

    let transaction = transaction.try_into_write()?;

    let mut table = transaction.open_table(graph_info.edges_table_definition())?;
    let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
    let mut table_destination =
      transaction.open_table(graph_info.edges_destination_index_definition())?;

    for e in edges
    {
      table.remove(e.edge.key)?;
      let (sk, dk) = if e.reversed
      {
        (e.edge.destination.key, e.edge.source.key)
      }
      else
      {
        (e.edge.source.key, e.edge.destination.key)
      };

      let mut v = table_source
        .remove(sk)?
        .ok_or_else(|| InternalError::UnknownNode)?
        .value();
      v.retain(|x| *x != e.edge.key);
      table_source.insert(sk, v)?;

      let mut v = table_destination
        .remove(dk)?
        .ok_or_else(|| InternalError::UnknownNode)?
        .value();
      v.retain(|x| *x != e.edge.key);
      table_destination.insert(dk, v)?;
    }
    Ok(())
  }
  /// Select edges
  fn select_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: &String,
    query: super::SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<Vec<super::EdgeResult>>
  {
    if query.source.is_select_none() || query.destination.is_select_none()
    {
      return Ok(Default::default());
    }
    let graph_info = self.get_graph_info(graph_name)?;

    match transaction
    {
      store::TransactionBox::Read(read) =>
      {
        let edges_table = read.open_table(graph_info.edges_table_definition())?;
        let nodes_table = read.open_table(graph_info.nodes_table_definition())?;

        let edges_source_index = Rc::new(RefCell::new(
          read.open_table(graph_info.edges_source_index_definition())?,
        ));
        let edges_destination_index = Rc::new(RefCell::new(
          read.open_table(graph_info.edges_destination_index_definition())?,
        ));

        self.select_edges_from_tables(
          query,
          directivity,
          edges_table,
          nodes_table,
          edges_source_index,
          edges_destination_index,
        )
      }
      store::TransactionBox::Write(write) =>
      {
        let edges_table = write.open_table(graph_info.edges_table_definition())?;
        let nodes_table = write.open_table(graph_info.nodes_table_definition())?;

        let edges_source_index = Rc::new(RefCell::new(
          write.open_table(graph_info.edges_source_index_definition())?,
        ));
        let edges_destination_index = Rc::new(RefCell::new(
          write.open_table(graph_info.edges_destination_index_definition())?,
        ));

        self.select_edges_from_tables(
          query,
          directivity,
          edges_table,
          nodes_table,
          edges_source_index,
          edges_destination_index,
        )
      }
    }
  }
  fn compute_statistics(&self, transaction: &mut Self::TransactionBox)
    -> Result<super::Statistics>
  {
    let mut edges_count = 0;
    let mut nodes_count = 0;
    let mut labels = Vec::new();
    let mut properties_count = 0;

    for n in self.select_nodes(
      transaction,
      &"default".into(),
      super::SelectNodeQuery::select_all(),
    )?
    {
      nodes_count += 1;
      for l in n.labels.iter()
      {
        if !labels.contains(l)
        {
          labels.push(l.to_owned());
        }
      }
      properties_count += n
        .properties
        .iter()
        .filter(|(_, v)| **v != value::Value::Null)
        .count();
    }
    for e in self.select_edges(
      transaction,
      &"default".into(),
      super::SelectEdgeQuery::select_all(),
      graph::EdgeDirectivity::Directed,
    )?
    {
      edges_count += 1;

      properties_count += e
        .edge
        .properties
        .iter()
        .filter(|(_, v)| **v != value::Value::Null)
        .count();
    }

    Ok(super::Statistics {
      nodes_count,
      edges_count,
      labels_nodes_count: labels.len(),
      properties_count,
    })
  }
}
