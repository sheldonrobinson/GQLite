use crate::prelude::*;

pub(crate) trait SqlQueries
{
  /// Query for creating a new graph.
  fn graph_create_query(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for deleting a  graph.
  fn graph_delete(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for creating a new node
  fn node_create_query(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for deleting the nodes.
  fn node_delete_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>;
  /// Query for updating a node.
  fn node_update_query(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for selecting a node.
  fn node_select_query(
    graph_name: impl AsRef<str>,
    keys_var: Option<usize>,
    labels_var: Option<usize>,
    properties_var: Option<usize>,
  ) -> Result<String>;
  /// Query for createing an edge.
  fn edge_create_query(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for deleting the edges.
  fn edge_delete_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>;
  /// Query for updating a node.
  fn edge_update_query(graph_name: impl AsRef<str>) -> Result<String>;
  /// Query for deleting edges for the given nodes.
  fn edge_delete_by_nodes_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>;
  /// Query for the number of edges connected to the nodes.
  fn edge_count_for_nodes_query(
    graph_name: impl AsRef<str>,
    keys: impl AsRef<Vec<String>>,
  ) -> Result<String>;
  /// Query for selecting an edge.
  #[allow(clippy::too_many_arguments)]
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
  ) -> Result<String>;
  /// Query for computing statistics.
  fn compute_statistics_query(graph_name: impl AsRef<str>) -> Result<String>;
}
