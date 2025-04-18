use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::{borrow::Borrow, collections::HashMap};

use persy::PersyError;

use crate::graph;
use crate::Error;
use crate::Result;

//   ____                 _     ___        __
//  / ___|_ __ __ _ _ __ | |__ |_ _|_ __  / _| ___
// | |  _| '__/ _` | '_ \| '_ \ | || '_ \| |_ / _ \
// | |_| | | | (_| | |_) | | | || || | | |  _| (_) |
//  \____|_|  \__,_| .__/|_| |_|___|_| |_|_|  \___/
//                 |_|

struct GraphInfo
{
  name: String,
  nodes_segment: persy::SegmentId,
  nodes_uuid_index: String,
  edges_segment: persy::SegmentId,
  edges_uuid_index: String,
  edges_source_uuid_index: String,
  edges_destination_uuid_index: String,
}

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

fn persy_id_serialize<S>(x: &persy::PersyId, s: S) -> std::result::Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  s.serialize_str(x.to_string().as_str())
}

fn persy_id_deserialize<'de, D>(deserializer: D) -> std::result::Result<persy::PersyId, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let buf = String::deserialize(deserializer)?;
  match buf.parse::<persy::PersyId>()
  {
    Ok(pid) => Ok(pid),
    Err(err) => Err(serde::de::Error::custom(format!(
      "Failed to parse PersyId: {}",
      err
    ))),
  }
}

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
  #[serde(
    serialize_with = "persy_id_serialize",
    deserialize_with = "persy_id_deserialize"
  )]
  pub source: persy::PersyId,
  #[serde(
    serialize_with = "persy_id_serialize",
    deserialize_with = "persy_id_deserialize"
  )]
  pub destination: persy::PersyId,
  pub label: String,
  pub properties: graph::ValueObject,
}

//  ____       _           _   _   _           _       ___
// / ___|  ___| | ___  ___| |_| \ | | ___   __| | ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  \| |/ _ \ / _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |\  | (_) | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_| \_|\___/ \__,_|\___|\__\_\\__,_|\___|_|   \__, |
//                                                                          |___/

#[derive(Default)]
pub(crate) struct SelectNodeQuery<'a, TKeys, TLabels>
where
  TKeys: Iterator<Item = &'a crate::graph::Key>,
  TLabels: Iterator<Item = &'a String>,
{
  keys: Option<TKeys>,
  labels: Option<TLabels>,
}

impl
  SelectNodeQuery<
    'static,
    core::slice::Iter<'static, crate::graph::Key>,
    core::slice::Iter<'static, String>,
  >
{
  pub(crate) fn select_all() -> Self
  {
    Self::default()
  }
}

impl<'a, TKeys: Iterator<Item = &'a crate::graph::Key>>
  SelectNodeQuery<'a, TKeys, core::slice::Iter<'a, String>>
{
  pub(crate) fn select_keys(keys: TKeys) -> Self
  {
    Self {
      keys: Some(keys),
      labels: None,
    }
  }
}

impl<'a, TLabels: Iterator<Item = &'a String>>
  SelectNodeQuery<'a, core::slice::Iter<'a, crate::graph::Key>, TLabels>
{
  pub(crate) fn select_labels(labels: TLabels) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels),
    }
  }
}

//  ____       _           _   _____    _             ___
// / ___|  ___| | ___  ___| |_| ____|__| | __ _  ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  _| / _` |/ _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |__| (_| | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_____\__,_|\__, |\___|\__\_\\__,_|\___|_|   \__, |
//                                        |___/                            |___/

#[derive(Default)]
pub(crate) struct SelectEdgeQuery<'a, T: Iterator<Item = &'a crate::graph::Key>>
{
  keys: Option<T>,
}

impl SelectEdgeQuery<'static, core::slice::Iter<'static, crate::graph::Key>>
{
  pub(crate) fn select_all() -> Self
  {
    Self::default()
  }
}

impl<'a, T: Iterator<Item = &'a crate::graph::Key>> SelectEdgeQuery<'a, T>
{
  pub(crate) fn select_keys(keys: T) -> Self
  {
    Self { keys: Some(keys) }
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

/// Storage, aka, interface to the underlying persy store.
pub(crate) struct Store
{
  persy_store: persy::Persy,
  graphs: HashMap<String, GraphInfo>,
}

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    let path = path.as_ref();
    let create = !path.exists();
    if create
    {
      persy::Persy::create(path)?;
    }
    let mut s = Store {
      persy_store: persy::Persy::open(path, persy::Config::new())?,
      graphs: Default::default(),
    };
    if create
    {
      s.create_graph("default")?;
    }
    s.graphs.insert(
      "default".into(),
      GraphInfo {
        name: "default".into(),
        nodes_segment: s.persy_store.solve_segment_id("default_nodes")?,
        nodes_uuid_index: "default_nodes_uuid_index".into(),
        edges_segment: s.persy_store.solve_segment_id("default_edges")?,
        edges_uuid_index: "default_edges_uuid_index".into(),
        edges_source_uuid_index: "default_edges_source_uuid_index".into(),
        edges_destination_uuid_index: "default_edges_destination_uuid_index".into(),
      },
    );
    Ok(s)
  }
  pub(crate) fn create_graph(&self, name: impl Into<String>) -> Result<()>
  {
    let mut tx = self.persy_store.begin()?;
    let graph_name = name.into();
    tx.create_segment(format!("{}_nodes", graph_name).as_str())?;
    tx.create_segment(format!("{}_edges", graph_name).as_str())?;
    for index_name in [
      "_nodes_uuid_index",
      "_edges_uuid_index",
      "_edges_source_uuid_index",
      "_edges_destination_uuid_index",
    ]
    {
      tx.create_index::<u128, persy::PersyId>(
        (graph_name.clone() + index_name).as_str(),
        persy::ValueMode::Exclusive,
      )?;
    }
    tx.commit()?;
    Ok(())
  }
  pub(crate) fn begin(&self) -> Result<persy::Transaction>
  {
    let s = self.persy_store.begin()?;
    Ok(s)
  }
  /// Add node to a graph
  pub(crate) fn add_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut persy::Transaction,
    graph_name: impl Into<String>,
    nodes_iter: T,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    for x in nodes_iter
    {
      let mut data = Vec::<u8>::new();
      ciborium::into_writer(&x, &mut data)?;
      let pid = transaction.insert(graph_info.nodes_segment, &data)?;
      transaction.put::<u128, persy::PersyId>(
        graph_info.nodes_uuid_index.as_str(),
        x.key.borrow().into(),
        pid,
      )?;
    }
    Ok(())
  }
  /// Select nodes according to a given query
  pub(crate) fn select_nodes<'a, TKeys, TLabels>(
    &self,
    transaction: &mut persy::Transaction,
    graph_name: impl Into<String>,
    query: SelectNodeQuery<'a, TKeys, TLabels>,
  ) -> Result<Vec<crate::graph::Node>>
  where
    TKeys: Iterator<Item = &'a crate::graph::Key>,
    TLabels: Iterator<Item = &'a String>,
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();

    let nodes_raw = match query.keys
    {
      Some(keys_iter) => Box::new(keys_iter.map(|key| {
        let key: u128 = key.into();
        if let Some(key) =
          transaction.one::<u128, persy::PersyId>(graph_info.nodes_uuid_index.as_str(), &key)?
        {
          if let Some(v) = transaction.read(graph_info.nodes_segment, key.borrow())?
          {
            Ok(v)
          }
          else
          {
            Err(Error::UnknownNode)
          }
        }
        else
        {
          Err(Error::UnknownNode)
        }
      })) as Box<dyn Iterator<Item = Result<Vec<u8>>>>,
      None => Box::new(
        transaction
          .scan(graph_info.nodes_segment)?
          .map(|(_, content)| Ok::<Vec<u8>, crate::Error>(content)),
      ) as Box<dyn Iterator<Item = Result<Vec<u8>>>>,
    };
    let r = nodes_raw.map(|v| {
      // Ok::<graph::Node, crate::Error>(ciborium::from_reader::<graph::Node, &[u8]>(
      //   &mut v?.as_ref(),
      // )?)
      let c = ciborium::from_reader::<graph::Node, &[u8]>(&mut v?.as_ref())?;
      println!("Retrieved: {:?}", c);
      Ok::<graph::Node, crate::Error>(c)
    });
    let r = match query.labels
    {
      Some(labels) =>
      {
        let labels = labels.collect::<Vec<&'a String>>();
        r.filter(|n| match n
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
        })
        .collect::<Result<Vec<crate::graph::Node>>>()
      }
      None => r.collect::<Result<Vec<crate::graph::Node>>>(),
    }?;
    Ok(r)
  }
  fn get_node_id(
    &self,
    transaction: &mut persy::Transaction,
    graph_name: &String,
    key: crate::graph::Key,
  ) -> Result<persy::PersyId>
  {
    let graph_info = self.graphs.get(graph_name).unwrap();
    let key: u128 = key.borrow().into();
    if let Some(pid) =
      transaction.one::<u128, persy::PersyId>(graph_info.nodes_uuid_index.as_str(), &key)?
    {
      Ok(pid)
    }
    else
    {
      println!("UnknownNode {:?}", key);
      Err(Error::UnknownNode)
    }
  }
  /// Add edge
  pub(crate) fn add_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    transaction: &mut persy::Transaction,
    graph_name: impl Into<String>,
    edges_iter: T,
  ) -> Result<()>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    for x in edges_iter
    {
      let source_pid = self.get_node_id(transaction, &graph_name, x.source.key)?;
      let destination_pid = self.get_node_id(transaction, &graph_name, x.destination.key)?;
      let mut data = Vec::<u8>::new();
      ciborium::into_writer(
        &PersistentEdge {
          key: x.key,
          source: source_pid,
          destination: destination_pid,
          label: x.label.clone(),
          properties: x.properties.clone(),
        },
        &mut data,
      )?;
      let pid = transaction.insert(graph_info.edges_segment, &data)?;
      transaction.put::<u128, persy::PersyId>(
        graph_info.edges_uuid_index.as_str(),
        x.key.borrow().into(),
        pid,
      )?;
      transaction.put::<u128, persy::PersyId>(
        &graph_info.edges_source_uuid_index.as_str(),
        x.source.key.borrow().into(),
        pid,
      )?;
      transaction.put::<u128, persy::PersyId>(
        graph_info.edges_destination_uuid_index.as_str(),
        x.destination.key.borrow().into(),
        pid,
      )?;
    }
    Ok(())
  }
  fn fetch_node(
    &self,
    transaction: &mut persy::Transaction,
    graph_info: &GraphInfo,
    node_key: persy::PersyId,
  ) -> Result<graph::Node>
  {
    if let Some(v) = transaction.read(graph_info.nodes_segment, node_key.borrow())?
    {
      Ok::<graph::Node, crate::Error>(ciborium::from_reader::<graph::Node, &[u8]>(
        &mut v.as_ref(),
      )?)
    }
    else
    {
      Err(Error::UnknownNode)
    }
  }
  /// Select edges
  pub(crate) fn select_edges<'a, T: Iterator<Item = &'a crate::graph::Key>>(
    &self,
    transaction: &mut persy::Transaction,
    graph_name: impl Into<String>,
    query: SelectEdgeQuery<'a, T>,
  ) -> Result<Vec<crate::graph::Edge>>
  {
    let graph_name = graph_name.into();
    let graph_info = self.graphs.get(&graph_name).unwrap();
    let transaction = RefCell::new(transaction);
    let edges_raw = {
      let mut transaction = transaction.borrow_mut();
      match query.keys
      {
        Some(keys_iter) => Box::new(keys_iter.map(|key| {
          let key: u128 = key.into();
          if let Some(key) =
            transaction.one::<u128, persy::PersyId>(graph_info.edges_uuid_index.as_str(), &key)?
          {
            if let Some(v) = transaction.read(graph_info.edges_segment, key.borrow())?
            {
              Ok(v)
            }
            else
            {
              Err(Error::UnknownNode)
            }
          }
          else
          {
            Err(Error::UnknownNode)
          }
        })) as Box<dyn Iterator<Item = Result<Vec<u8>>>>,
        None => Box::new({
          transaction
            .scan(graph_info.edges_segment)?
            .map(|(_, content)| Ok::<Vec<u8>, crate::Error>(content))
        }) as Box<dyn Iterator<Item = Result<Vec<u8>>>>,
      }
      .collect::<Result<Vec<Vec<u8>>>>()?
    };
    let r = edges_raw
      .into_iter()
      .map(|v| {
        Ok::<graph::Edge, crate::Error>({
          let edge = ciborium::from_reader::<PersistentEdge, &[u8]>(&mut v.as_ref())?;
          let mut transaction = transaction.borrow_mut();
          graph::Edge {
            key: edge.key,
            source: self.fetch_node(&mut transaction, graph_info, edge.source)?,
            destination: self.fetch_node(&mut transaction, graph_info, edge.destination)?,
            label: edge.label,
            properties: edge.properties,
          }
        })
      })
      .collect::<Result<Vec<crate::graph::Edge>>>()?;
    Ok(r)
  }
  pub(crate) fn compute_statistics(
    &self,
    transaction: &mut persy::Transaction,
  ) -> Result<Statistics>
  {
    let mut edges_count = 0;
    let mut nodes_count = 0;
    let mut labels = Vec::new();
    let mut properties_count = 0;

    for n in self.select_nodes(transaction, "default", SelectNodeQuery::select_all())?
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
    for e in self.select_edges(transaction, "default", SelectEdgeQuery::select_all())?
    {
      edges_count += 1;

      properties_count += e
        .properties
        .iter()
        .filter(|(_, v)| **v != graph::Value::Invalid)
        .count();
    }

    Ok(Statistics {
      nodes_count,
      edges_count,
      labels_nodes_count: labels.len(),
      properties_count,
    })
  }
}

// Error

impl<T> From<persy::PE<T>> for crate::Error
where
  T: Into<PersyError>,
{
  fn from(value: persy::PE<T>) -> Self
  {
    match value
    {
      persy::PE::PE(err) => Error::StoreError(err.into().to_string()),
    }
  }
}

#[cfg(test)]
mod tests
{
  use std::borrow::BorrowMut;

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
      .add_nodes(tx.borrow_mut(), "default", nodes.iter())
      .unwrap();
    tx.commit().unwrap();

    let selected_nodes = store
      .select_nodes(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectNodeQuery::select_keys([nodes[0].key].iter()),
      )
      .unwrap();

    assert_eq!(selected_nodes.len(), 1);
    assert_eq!(nodes[0], selected_nodes[0]);
    // Add a single node with label
    let selected_nodes = store
      .select_nodes(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectNodeQuery::select_labels(["not".to_string()].iter()),
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
      label: "!".into(),
      properties: crate::properties!("existence" => true),
    };
    let store = super::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

    let mut tx = store.begin().unwrap();
    store
      .add_edges(tx.borrow_mut(), "default", [edge.clone()].iter())
      .expect_err("expect missing node");
    store
      .add_nodes(
        tx.borrow_mut(),
        "default",
        [source_node, destination_node].iter(),
      )
      .unwrap();
    store
      .add_edges(tx.borrow_mut(), "default", [edge.clone()].iter())
      .unwrap();
    tx.commit().unwrap();

    let selected_edges = store
      .select_edges(
        store.begin().unwrap().borrow_mut(),
        "default",
        crate::store::SelectEdgeQuery::select_keys([edge.key].iter()),
      )
      .unwrap();

    assert_eq!(1, selected_edges.len());
    assert_eq!(edge, selected_edges[0]);
  }
}
