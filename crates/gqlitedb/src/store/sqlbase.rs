use std::collections::HashSet;

use crate::{prelude::*, store::EdgeResult, utils::hex};

mod sqlmetadata;
mod sqlqueries;
mod sqlstore;
mod sqlvalue;

pub(crate) use sqlmetadata::{SqlMetaDataQueries, SqlMetaDataStore};
pub(crate) use sqlqueries::SqlQueries;
pub(crate) use sqlstore::{Row, SqlStore};
pub(crate) use sqlvalue::{FromSqlValue, IntoBindings, SqlValue};

ccutils::alias!(pub(crate) PersistentKey, graph::Key, derive: Debug, PartialEq);

/// Implementation of a GQLite store for all type that implements the SqlStore trait
impl<TStore> store::Store for TStore
where
  TStore: SqlStore + SqlMetaDataQueries + SqlQueries,
{
  type TransactionBox = <Self as SqlStore>::TransactionBox;
  fn begin_read(&self) -> Result<Self::TransactionBox>
  {
    <Self as SqlStore>::begin_sql_read(self)
  }
  fn begin_write(&self) -> Result<Self::TransactionBox>
  {
    <Self as SqlStore>::begin_sql_write(self)
  }
  fn graphs_list(&self, transaction: &mut Self::TransactionBox) -> Result<Vec<String>>
  {
    self.get_metadata_value_json(transaction, "graphs")
  }
  fn create_graph(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    ignore_if_exists: bool,
  ) -> Result<()>
  {
    let graph_name = graph_name.as_ref();
    let mut graphs_list = self.graphs_list(transaction)?;
    if graphs_list.iter().any(|s| s == graph_name)
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
    self.execute_batch(transaction, Self::graph_create_query(graph_name)?)?;
    graphs_list.push(graph_name.to_owned());
    self.set_metadata_value_json(transaction, "graphs", &graphs_list)?;
    Ok(())
  }
  fn drop_graph(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    if_exists: bool,
  ) -> Result<()>
  {
    let graph_name = graph_name.as_ref();
    let mut graphs_list = self.graphs_list(transaction)?;
    if graphs_list.iter().any(|s| s == graph_name)
    {
      self.execute_batch(transaction, Self::graph_delete(graph_name)?)?;
      graphs_list.retain(|x| x != graph_name);
      self.set_metadata_value_json(transaction, "graphs", &graphs_list)?;

      Ok(())
    }
    else if if_exists
    {
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
  fn create_nodes<'a, T: IntoIterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    nodes_iter: T,
  ) -> Result<()>
  {
    for x in nodes_iter
    {
      self.execute(
        transaction,
        Self::node_create_query(&graph_name)?.as_str(),
        (
          &x.key(),
          &serde_json::to_string(x.labels())?,
          &serde_json::to_string(x.properties())?,
        ),
      )?;
    }
    Ok(())
  }
  fn delete_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: store::SelectNodeQuery,
    detach: bool,
  ) -> Result<()>
  {
    let graph_name = graph_name.as_ref();
    let nodes = self.select_nodes(transaction, graph_name, query)?;
    let nodes_keys: Vec<String> = nodes.into_iter().map(|x| hex(x.key())).collect();
    if detach
    {
      self.execute(
        transaction,
        Self::edge_delete_by_nodes_query(graph_name, &nodes_keys)?,
        (),
      )?;
    }
    else
    {
      let count = self.query_row(
        transaction,
        Self::edge_count_for_nodes_query(graph_name, &nodes_keys)?,
        (),
        |row| row.get::<usize>(0),
      )?;
      if count > 0
      {
        return Err(error::RunTimeError::DeleteConnectedNode.into());
      }
    }
    self.execute(
      transaction,
      Self::node_delete_query(graph_name, nodes_keys)?,
      (),
    )?;
    Ok(())
  }
  fn update_node(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    node: &crate::graph::Node,
  ) -> Result<()>
  {
    self.execute(
      transaction,
      Self::node_update_query(graph_name)?,
      (
        &node.key(),
        &serde_json::to_string(node.labels())?,
        &serde_json::to_string(node.properties())?,
      ),
    )?;
    Ok(())
  }
  fn select_nodes(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: store::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
    let mut query_bindings = Vec::<SqlValue>::new();
    let keys_var = query
      .keys
      .map(|keys| {
        let hex_keys = keys.iter().map(|key| hex(*key)).collect::<Vec<_>>();
        query_bindings.push(serde_json::to_string(&hex_keys)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let labels_var = query
      .labels
      .map(|labels| {
        query_bindings.push(serde_json::to_string(&labels)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let properties_var = query
      .properties
      .map(|properties| {
        query_bindings.push(serde_json::to_string(&properties)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let mut nodes: Vec<graph::Node> = Default::default();
    self.query_rows(
      transaction,
      Self::node_select_query(graph_name, keys_var, labels_var, properties_var)?,
      query_bindings,
      |row| {
        let key: graph::Key = row.get::<graph::Key>(0)?;
        let labels = serde_json::from_str(&row.get::<String>(1)?)?;
        let properties = serde_json::from_str(&row.get::<String>(2)?)?;
        nodes.push(graph::Node::new(key, labels, properties));
        Ok(())
      },
    )?;

    Ok(nodes)
  }
  fn create_edges<'a, T: IntoIterator<Item = &'a crate::graph::SinglePath>>(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    edges_iter: T,
  ) -> Result<()>
  {
    for x in edges_iter
    {
      self.execute(
        transaction,
        Self::edge_create_query(graph_name.as_ref())?,
        (
          &x.key(),
          &serde_json::to_string(x.labels())?,
          &serde_json::to_string(x.properties())?,
          &x.source().key(),
          &x.destination().key(),
        ),
      )?;
    }
    Ok(())
  }
  fn delete_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    query: store::SelectEdgeQuery,
    directivity: crate::graph::EdgeDirectivity,
  ) -> Result<()>
  {
    let graph_name = graph_name.as_ref();
    let edges = self.select_edges(transaction, graph_name, query, directivity)?;
    let edges_keys: Vec<String> = edges.into_iter().map(|x| hex(x.path.key())).collect();
    self.execute(
      transaction,
      Self::edge_delete_query(graph_name, &edges_keys)?,
      (),
    )?;
    Ok(())
  }
  fn update_edge(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
    edge: &crate::graph::Edge,
  ) -> Result<()>
  {
    self.execute(
      transaction,
      Self::edge_update_query(graph_name)?,
      (
        &edge.key(),
        serde_json::to_string(edge.labels())?,
        serde_json::to_string(edge.properties())?,
      ),
    )?;
    Ok(())
  }
  fn select_edges(
    &self,
    transaction: &mut Self::TransactionBox,
    graph_name: impl AsRef<str>,
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

    let mut query_bindings = Vec::<SqlValue>::new();

    // Edge queries
    let edge_keys_var = query
      .keys
      .map(|keys| {
        let hex_keys = keys.iter().map(|key| hex(*key)).collect::<Vec<_>>();
        query_bindings.push(serde_json::to_string(&hex_keys)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let edge_labels_var = query
      .labels
      .map(|labels| {
        query_bindings.push(serde_json::to_string(&labels)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let edge_properties_var = query
      .properties
      .map(|properties| {
        query_bindings.push(serde_json::to_string(&properties)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;

    // Left node queries
    let left_node_keys_var = query
      .source
      .keys
      .map(|keys| {
        let hex_keys = keys.iter().map(|key| hex(*key)).collect::<Vec<_>>();
        query_bindings.push(serde_json::to_string(&hex_keys)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let left_node_labels_var = query
      .source
      .labels
      .map(|labels| {
        query_bindings.push(serde_json::to_string(&labels)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let left_node_properties_var = query
      .source
      .properties
      .map(|properties| {
        query_bindings.push(serde_json::to_string(&properties)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;

    // Right node queries
    let right_node_keys_var = query
      .destination
      .keys
      .map(|keys| {
        let hex_keys = keys.iter().map(|key| hex(*key)).collect::<Vec<_>>();
        query_bindings.push(serde_json::to_string(&hex_keys)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let right_node_labels_var = query
      .destination
      .labels
      .map(|labels| {
        query_bindings.push(serde_json::to_string(&labels)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;
    let right_node_properties_var = query
      .destination
      .properties
      .map(|properties| {
        query_bindings.push(serde_json::to_string(&properties)?.into());
        Ok::<_, Error>(query_bindings.len())
      })
      .transpose()?;

    let mut edges: Vec<EdgeResult> = Default::default();
    let mut edges_keys: HashSet<u128> = Default::default();

    // Execute query
    self.query_rows(
      transaction,
      Self::edge_select_query(
        graph_name,
        is_undirected,
        table_suffix,
        edge_keys_var,
        edge_labels_var,
        edge_properties_var,
        left_node_keys_var,
        left_node_labels_var,
        left_node_properties_var,
        right_node_keys_var,
        right_node_labels_var,
        right_node_properties_var,
      )?,
      query_bindings,
      |row| {
        let edge_key: graph::Key = row.get(0)?;
        let n_left_key: graph::Key = row.get(4)?;
        let n_right_key: graph::Key = row.get(7)?;

        // This ensure that if (a)-[]->(a) the edge is returned only once. But matching [a]-[]-[b] return the edge twice.
        if n_left_key == n_right_key && edges_keys.contains(&edge_key.uuid())
        {
          return Ok(());
        }
        edges_keys.insert(edge_key.uuid());
        let edge_labels = serde_json::from_str(&row.get::<String>(1)?)?;
        let edge_properties = serde_json::from_str(&row.get::<String>(2)?)?;

        let n_left_labels = serde_json::from_str(&row.get::<String>(5)?)?;
        let n_left_properties = serde_json::from_str(&row.get::<String>(6)?)?;

        let n_right_labels = serde_json::from_str(&row.get::<String>(8)?)?;
        let n_right_properties = serde_json::from_str(&row.get::<String>(9)?)?;

        let source = graph::Node::new(n_left_key, n_left_labels, n_left_properties);
        let destination = graph::Node::new(n_right_key, n_right_labels, n_right_properties);
        let reversed = row.get::<u32>(3)? == 1;
        let (source, destination) = if reversed
        {
          (destination, source)
        }
        else
        {
          (source, destination)
        };

        edges.push(EdgeResult {
          path: graph::Path::new(edge_key, source, edge_labels, edge_properties, destination),
          reversed,
        });
        Ok(())
      },
    )?;

    Ok(edges)
  }
  fn compute_statistics(&self, transaction: &mut Self::TransactionBox)
    -> Result<store::Statistics>
  {
    let (nodes_count, edges_count, labels_nodes_count, properties_count) = self.query_row(
      transaction,
      Self::compute_statistics_query("default")?,
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
