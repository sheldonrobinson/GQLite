use std::collections::HashMap;

#[cfg(feature = "pgql")]
mod pgql;
#[cfg(feature = "redb")]
mod redb;

#[cfg(feature = "redb")]
pub(crate) use redb::{Store, Transaction};

#[cfg(feature = "pgql")]
pub(crate) use pgql::Store;

use crate::graph;

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

//  _____    _            ____                 _ _
// | ____|__| | __ _  ___|  _ \ ___  ___ _   _| | |_
// |  _| / _` |/ _` |/ _ \ |_) / _ \/ __| | | | | __|
// | |__| (_| | (_| |  __/  _ <  __/\__ \ |_| | | |_
// |_____\__,_|\__, |\___|_| \_\___||___/\__,_|_|\__|
//             |___/

pub(crate) struct EdgeResult
{
  pub(crate) edge: graph::Edge,
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
  properties: Option<HashMap<String, graph::Value>>,
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
    properties: HashMap<String, graph::Value>,
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
      if !keys.iter().any(|x| node.key == *x)
      {
        return false;
      }
    }
    if let Some(labels) = &self.labels
    {
      if !labels.iter().all(|x| node.labels.contains(x))
      {
        return false;
      }
    }
    if let Some(properties) = &self.properties
    {
      if !properties
        .iter()
        .all(|(k, v)| node.properties.get(k) == Some(v))
      {
        return false;
      }
    }
    return true;
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
  properties: Option<HashMap<String, graph::Value>>,
  source: SelectNodeQuery,
  destination: SelectNodeQuery,
}

impl SelectEdgeQuery
{
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
    properties: HashMap<String, graph::Value>,
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
  pub(crate) fn is_match(&self, edge: &graph::Edge) -> bool
  {
    if let Some(keys) = &self.keys
    {
      if !keys.iter().any(|x| edge.key == *x)
      {
        return false;
      }
    }
    if let Some(labels) = &self.labels
    {
      if !labels.iter().all(|x| edge.labels.contains(x))
      {
        return false;
      }
    }
    if let Some(properties) = &self.properties
    {
      if !properties
        .iter()
        .all(|(k, v)| edge.properties.get(k) == Some(v))
      {
        return false;
      }
    }
    return self.source.is_match(&edge.source) && self.destination.is_match(&edge.destination);
  }
}
