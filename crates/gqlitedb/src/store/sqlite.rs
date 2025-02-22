use crate::{store, Result};

pub(crate) struct Store
{
  connection: sqlite::Connection,
}

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    let connection = sqlite::open(path)?;

    let mut s = Store { connection };
    s.create_graph("default", true)?;
    Ok(s)
  }
}

pub(crate) struct Transaction {}

impl store::Store for Store
{
  type Transaction = Transaction;
  fn begin(&self) -> Result<Self::Transaction>
  {
    Ok(Transaction {})
  }
  fn commit(&self, _transaction: Self::Transaction) -> Result<()>
  {
    Ok(())
  }
  fn create_graph(&mut self, name: impl Into<String>, _ignore_if_exists: bool) -> Result<()> {}
  fn create_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    nodes_iter: T,
  ) -> Result<()>
  {
  }
  fn delete_nodes(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    query: store::SelectNodeQuery,
    detach: bool,
  ) -> Result<()>
  {
  }
  fn update_node(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    node: &crate::graph::Node,
  ) -> Result<()>
  {
  }
  fn select_nodes(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    query: store::SelectNodeQuery,
  ) -> Result<Vec<crate::graph::Node>>
  {
  }
  fn create_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    edges_iter: T,
  ) -> Result<()>
  {
  }
  fn delete_edges(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    query: store::SelectEdgeQuery,
    directivity: crate::graph::EdgeDirectivity,
  ) -> Result<()>
  {
  }
  fn update_edge(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    edge: &crate::graph::Edge,
  ) -> Result<()>
  {
  }
  fn select_edges(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    query: store::SelectEdgeQuery,
    directivity: crate::graph::EdgeDirectivity,
  ) -> Result<Vec<store::EdgeResult>>
  {
  }
  fn compute_statistics(&self, transaction: &mut Self::Transaction) -> Result<store::Statistics> {}
}
