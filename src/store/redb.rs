use itertools::Itertools;
use pest::pratt_parser::Op;
use redb::{ReadableTable, ReadableTableMetadata};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error, graph, Result};

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
  pub properties: graph::ValueObject,
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
  edge_data: Vec<u8>,
  reversed: Option<bool>,
  source_id: Option<graph::Key>,
  destination_id: Option<graph::Key>,
}

impl EdgeIdResult
{
  fn new(
    edge_data: Vec<u8>,
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
  fn is_reversed(&self, edge: &graph::Edge) -> bool
  {
    match self.reversed
    {
      Some(v) => v,
      None => match self.source_id
      {
        Some(v) => edge.destination.key == v,
        None => match self.destination_id
        {
          Some(v) => edge.source.key == v,
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
    let v = self.get_required(key, error::Error::UnknownNode)?;
    let c = ciborium::from_reader::<graph::Node, &[u8]>(&mut v.value())?;
    Ok::<graph::Node, crate::Error>(c)
  }
}

trait TableExtension<K, V>
where
  K: redb::Key + 'static,
  V: redb::Value + 'static,
{
  fn get_required<'a>(
    &self,
    key: impl std::borrow::Borrow<K::SelfType<'a>>,
    err: error::Error,
  ) -> Result<redb::AccessGuard<V>>;
}

impl<'txn, K, V> TableExtension<K, V> for redb::Table<'txn, K, V>
where
  K: redb::Key + 'static,
  V: redb::Value + 'static,
{
  fn get_required<'a>(
    &self,
    key: impl std::borrow::Borrow<K::SelfType<'a>>,
    err: error::Error,
  ) -> Result<redb::AccessGuard<V>>
  {
    self.get(key)?.ok_or_else(|| error::show_backtrace(err))
  }
}

//   ____                 _     ___        __
//  / ___|_ __ __ _ _ __ | |__ |_ _|_ __  / _| ___
// | |  _| '__/ _` | '_ \| '_ \ | || '_ \| |_ / _ \
// | |_| | | | (_| | |_) | | | || || | | |  _| (_) |
//  \____|_|  \__,_| .__/|_| |_|___|_| |_|_|  \___/
//                 |_|

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
  fn nodes_table_definition<'a, 'b>(&'a self) -> redb::TableDefinition<'a, graph::Key, &'b [u8]>
  {
    redb::TableDefinition::new(&self.nodes_table)
  }
  fn edges_table_definition<'a, 'b>(&'a self) -> redb::TableDefinition<'a, graph::Key, &'b [u8]>
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

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

/// Storage, aka, interface to the underlying redb store.
pub(crate) struct Store
{
  redb_store: redb::Database,
  graphs: HashMap<String, GraphInfo>,
}

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    let path = path.as_ref();
    let mut s = Store {
      redb_store: redb::Database::create(path)?,
      graphs: Default::default(),
    };
    s.create_graph("default", true)?;
    Ok(s)
  }
  pub(crate) fn create_graph(
    &mut self,
    name: impl Into<String>,
    _ignore_if_exists: bool,
  ) -> Result<()>
  {
    let gi = GraphInfo::new(name);

    let tx = self.redb_store.begin_write()?;
    tx.open_table(gi.nodes_table_definition())?;
    tx.open_table(gi.edges_table_definition())?;
    tx.open_table(gi.edges_source_index_definition())?;
    tx.open_table(gi.edges_destination_index_definition())?;
    tx.commit()?;

    self.graphs.insert("default".into(), gi);

    Ok(())
  }
  pub(crate) fn begin(&self) -> Result<redb::WriteTransaction>
  {
    let s = self.redb_store.begin_write()?;
    Ok(s)
  }
  /// Create nodes and add them to a graph
  pub(crate) fn create_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    nodes_iter: T,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let mut table = transaction.open_table(graph_info.nodes_table_definition())?;
    let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
    let mut table_destination =
      transaction.open_table(graph_info.edges_destination_index_definition())?;
    for x in nodes_iter
    {
      let mut data = Vec::<u8>::new();
      ciborium::into_writer(&x, &mut data)?;
      table.insert(x.key, data.as_slice())?;
      table_source.insert(x.key, vec![])?;
      table_destination.insert(x.key, vec![])?;
    }
    Ok(())
  }
  /// Delete nodes according to a given query
  pub(crate) fn delete_nodes(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    query: super::SelectNodeQuery,
    detach: bool,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();

    if query.is_select_all()
    {
      if detach
      {
        transaction.delete_table(graph_info.edges_table_definition())?;
        transaction.delete_table(graph_info.edges_source_index_definition())?;
        transaction.delete_table(graph_info.edges_destination_index_definition())?;
        transaction.open_table(graph_info.edges_table_definition())?;
        transaction.open_table(graph_info.edges_source_index_definition())?;
        transaction.open_table(graph_info.edges_destination_index_definition())?;
      }
      else
      {
        let edge_table = transaction.open_table(graph_info.edges_table_definition())?;
        if edge_table.len()? > 0
        {
          return Err(error::RunTimeError::DeleteConnectedNode.into());
        }
      }
      transaction.delete_table(graph_info.nodes_table_definition())?;
      transaction.open_table(graph_info.nodes_table_definition())?;
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
          .select_nodes(transaction, &graph_name, query)?
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
        // Check if the nodes are disconnected
        let table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
        let table_destination =
          transaction.open_table(graph_info.edges_destination_index_definition())?;

        for key in node_keys.iter()
        {
          if !table_source
            .get_required(key, error::Error::UnknownNode)?
            .value()
            .is_empty()
            || !table_destination
              .get_required(key, error::Error::UnknownNode)?
              .value()
              .is_empty()
          {
            return Err(error::RunTimeError::DeleteConnectedNode.into());
          }
        }
      }
      // Delete the nodes
      let mut table_nodes = transaction.open_table(graph_info.nodes_table_definition())?;
      let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
      let mut table_destination =
        transaction.open_table(graph_info.edges_destination_index_definition())?;
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
  pub(crate) fn select_nodes(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    query: super::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let nodes_table = transaction.open_table(graph_info.nodes_table_definition())?;
    self.select_nodes_from_table(&nodes_table, query)
  }
  fn select_nodes_from_table<'txn>(
    &self,
    nodes_table: &redb::Table<'txn, graph::Key, &[u8]>,
    query: super::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
    let nodes_raw = match query.keys
    {
      Some(keys) => Box::new(
        keys
          .into_iter()
          .map(|key| Ok(nodes_table.get_required(key, error::Error::UnknownNode)?)),
      ) as Box<dyn Iterator<Item = Result<redb::AccessGuard<'_, &[u8]>>>>,
      None => Box::new({
        nodes_table.range::<graph::Key>(..)?.into_iter().map(|r| {
          let (_, v) = r?;
          Ok(v)
        })
      }) as Box<dyn Iterator<Item = Result<redb::AccessGuard<'_, &[u8]>>>>,
    };
    let r = nodes_raw.map(|v| {
      let c = ciborium::from_reader::<graph::Node, &[u8]>(&mut v?.value())?;
      Ok::<graph::Node, crate::Error>(c)
    });
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
  /// Add edge
  pub(crate) fn create_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    edges_iter: T,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let mut table = transaction.open_table(graph_info.edges_table_definition())?;
    let mut table_source = transaction.open_table(graph_info.edges_source_index_definition())?;
    let mut table_destination =
      transaction.open_table(graph_info.edges_destination_index_definition())?;

    for x in edges_iter
    {
      let mut data = Vec::<u8>::new();
      ciborium::into_writer(
        &PersistentEdge {
          key: x.key,
          source: x.source.key,
          destination: x.destination.key,
          labels: x.labels.clone(),
          properties: x.properties.clone(),
        },
        &mut data,
      )?;
      table.insert(x.key, data.as_slice())?;
      let mut keys = table_source
        .remove(x.source.key)?
        .ok_or(error::Error::UnknownNode)?
        .value();
      keys.push(x.key);
      table_source.insert(x.source.key, keys)?;
      let mut keys = table_destination
        .remove(x.destination.key)?
        .ok_or(error::Error::UnknownNode)?
        .value();
      keys.push(x.key);
      table_destination.insert(x.destination.key, keys)?;
    }
    Ok(())
  }
  /// Delete nodes according to a given query
  pub(crate) fn delete_edges(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    query: super::SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let edges = self.select_edges(transaction, graph_name, query, directivity)?;

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
        .ok_or_else(|| error::show_backtrace(error::Error::UnknownNode))?
        .value();
      v.retain(|x| *x != e.edge.key);
      table_source.insert(sk, v)?;

      let mut v = table_destination
        .remove(sk)?
        .ok_or_else(|| error::show_backtrace(error::Error::UnknownNode))?
        .value();
      v.retain(|x| *x != e.edge.key);
      table_destination.insert(sk, v)?;
    }
    Ok(())
  }
  /// Select edges
  pub(crate) fn select_edges(
    &self,
    transaction: &mut redb::WriteTransaction,
    graph_name: impl Into<String>,
    query: super::SelectEdgeQuery,
    directivity: graph::EdgeDirectivity,
  ) -> Result<Vec<super::EdgeResult>>
  {
    if query.source.is_select_none() || query.destination.is_select_none()
    {
      return Ok(Default::default());
    }
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let edges_table = transaction.open_table(graph_info.edges_table_definition())?;
    let nodes_table = transaction.open_table(graph_info.nodes_table_definition())?;

    let edges_source_index = Rc::new(RefCell::new(
      transaction.open_table(graph_info.edges_source_index_definition())?,
    ));
    let edges_destination_index = Rc::new(RefCell::new(
      transaction.open_table(graph_info.edges_destination_index_definition())?,
    ));

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
              .ok_or_else(|| error::show_backtrace(error::Error::UnknownNode))?
              .value()
              .to_vec(),
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
              Ok(EdgeIdResult::new(
                v.value().to_vec(),
                Some(false),
                None,
                None,
              ))
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
                    .get_required(n.key, error::Error::UnknownNode)
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
                  .get_required(nkey, error::Error::UnknownNode)?
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
                          .get_required(k, error::Error::UnknownEdge)?
                          .value()
                          .to_vec(),
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
                  .get_required(nkey, error::Error::UnknownNode)?
                  .value()
                  .iter()
                {
                  let uniq_k = (k.to_owned(), Some(nkey), None);
                  if !edge_ids.contains(&uniq_k)
                  {
                    edges_raw.push(EdgeIdResult::new(
                      edges_table
                        .get_required(k, error::Error::UnknownEdge)?
                        .value()
                        .to_vec(),
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
                  .get_required(nkey, error::Error::UnknownNode)?
                  .value()
                  .iter()
                {
                  let uniq_k = (k.to_owned(), None, Some(nkey));
                  if !edge_ids.contains(&uniq_k)
                  {
                    edges_raw.push(EdgeIdResult::new(
                      edges_table
                        .get_required(k, error::Error::UnknownEdge)?
                        .value()
                        .to_vec(),
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
      Ok::<super::EdgeResult, crate::Error>({
        let edge = ciborium::from_reader::<PersistentEdge, &[u8]>(&mut &v.edge_data.as_ref())?;

        let source = nodes_table.get_node(edge.source)?;
        let destination = nodes_table.get_node(edge.destination)?;

        let edge = graph::Edge {
          key: edge.key,
          source,
          destination,
          labels: edge.labels,
          properties: edge.properties,
        };
        let reversed = v.is_reversed(&edge);
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
  pub(crate) fn compute_statistics(
    &self,
    transaction: &mut redb::WriteTransaction,
  ) -> Result<super::Statistics>
  {
    let mut edges_count = 0;
    let mut nodes_count = 0;
    let mut labels = Vec::new();
    let mut properties_count = 0;

    for n in self.select_nodes(transaction, "default", super::SelectNodeQuery::select_all())?
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
        .filter(|(_, v)| **v != graph::Value::Invalid)
        .count();
    }
    for e in self.select_edges(
      transaction,
      "default",
      super::SelectEdgeQuery::select_all(),
      graph::EdgeDirectivity::Directed,
    )?
    {
      edges_count += 1;

      properties_count += e
        .edge
        .properties
        .iter()
        .filter(|(_, v)| **v != graph::Value::Invalid)
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

#[cfg(test)]
mod tests
{
  use std::{borrow::BorrowMut, default};

  use crate::graph;

  #[test]
  fn test_add_nodes()
  {
    // Add a single empty node
    let nodes = [
      crate::graph::Node {
        labels: crate::labels!("hello", "world"),
        properties: crate::properties!("key" => 42i64),
        key: crate::graph::Key::default(),
      },
      crate::graph::Node {
        labels: crate::labels!("not"),
        properties: Default::default(),
        key: crate::graph::Key::default(),
      },
    ];
    let store = super::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

    let mut tx = store.begin().unwrap();
    store
      .create_nodes(tx.borrow_mut(), "default", nodes.iter())
      .unwrap();
    tx.commit().unwrap();

    let selected_nodes = store
      .select_nodes(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectNodeQuery::select_keys([nodes[0].key]),
      )
      .unwrap();

    assert_eq!(selected_nodes.len(), 1);
    assert_eq!(nodes[0], selected_nodes[0]);
    // Add a single node with label
    let selected_nodes = store
      .select_nodes(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectNodeQuery::select_labels(["not".to_string()]),
      )
      .unwrap();

    assert_eq!(selected_nodes.len(), 1);
    assert_eq!(nodes[1], selected_nodes[0]);
  }
  #[test]
  fn test_add_edges()
  {
    let source_node = crate::graph::Node {
      labels: crate::labels!("hello"),
      properties: crate::properties!("key" => 42i64),
      key: crate::graph::Key::default(),
    };
    let destination_node = crate::graph::Node {
      labels: crate::labels!("world"),
      properties: crate::properties!("key" => 12i64),
      key: crate::graph::Key::default(),
    };
    let edge = crate::graph::Edge {
      source: source_node.clone(),
      destination: destination_node.clone(),
      key: crate::graph::Key::default(),
      labels: vec!["!".into()],
      properties: crate::properties!("existence" => true),
    };
    let store = super::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

    let mut tx = store.begin().unwrap();
    store
      .create_edges(tx.borrow_mut(), "default", [edge.clone()].iter())
      .expect_err("expect missing node");
    store
      .create_nodes(
        tx.borrow_mut(),
        "default",
        [source_node, destination_node].iter(),
      )
      .unwrap();
    store
      .create_edges(tx.borrow_mut(), "default", [edge.clone()].iter())
      .unwrap();
    tx.commit().unwrap();

    let selected_edges = store
      .select_edges(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectEdgeQuery::select_keys([edge.key]),
        graph::EdgeDirectivity::Directed,
      )
      .unwrap();

    assert_eq!(1, selected_edges.len());
    assert_eq!(edge, selected_edges[0].edge);
    assert!(!selected_edges[0].reversed);

    let selected_edges = store
      .select_edges(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
          crate::store::SelectNodeQuery::select_all(),
          vec![],
          Default::default(),
          crate::store::SelectNodeQuery::select_all(),
        ),
        graph::EdgeDirectivity::Directed,
      )
      .unwrap();

    assert_eq!(1, selected_edges.len());
    assert_eq!(edge, selected_edges[0].edge);
    assert!(!selected_edges[0].reversed);

    let selected_edges = store
      .select_edges(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
          crate::store::SelectNodeQuery::select_labels_properties(vec![], Default::default()),
          vec![],
          Default::default(),
          crate::store::SelectNodeQuery::select_labels_properties(vec![], Default::default()),
        ),
        graph::EdgeDirectivity::Directed,
      )
      .unwrap();

    assert_eq!(1, selected_edges.len());
    assert_eq!(edge, selected_edges[0].edge);
    assert!(!selected_edges[0].reversed);
  }
}
