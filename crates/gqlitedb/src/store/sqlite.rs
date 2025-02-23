use askama::Template;
use std::rc::Rc;

use crate::{store, Result};

pub(crate) struct Store
{
  connection: Rc<rusqlite::Connection>,
}

impl Store
{
  /// Crate a new store, with a default graph
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Store>
  {
    use store::Store;
    let connection = Rc::new(rusqlite::Connection::open(path)?);

    let mut s = Self { connection };
    s.create_graph("default", true)?;
    Ok(s)
  }
}

pub(crate) struct Transaction
{
  connection: Rc<rusqlite::Connection>,
}

impl Drop for Transaction
{
  fn drop(&mut self)
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

mod templates
{
  use askama::Template;
  #[derive(Template)]
  #[template(path = "sql/sqlite/graph_create.sql", escape = "none")]
  pub(super) struct GraphCreate
  {
    pub graph_name: String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_create.sql", escape = "none")]
  pub(super) struct NodeCreate
  {
    pub graph_name: String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/node_delete.sql", escape = "none")]
  pub(super) struct NodeDelege
  {
    pub graph_name: String,
    pub what: String,
  }
  #[derive(Template)]
  #[template(path = "sql/sqlite/edge_delete_by_nodes.sql", escape = "none")]
  pub(super) struct EdgeDeleteByNodes
  {
    pub graph_name: String,
    pub what: String,
  }
}

impl store::Store for Store
{
  type Transaction = Transaction;
  fn begin(&self) -> Result<Self::Transaction>
  {
    let connection = self.connection.clone();
    connection.execute("BEGIN", ())?;
    Ok(Transaction { connection })
  }
  fn commit(&self, transaction: Self::Transaction) -> Result<()>
  {
    transaction.connection.execute("COMMIT", ())?;
    Ok(())
  }
  fn create_graph(&mut self, name: impl Into<String>, _ignore_if_exists: bool) -> Result<()>
  {
    self.connection.execute(
      templates::GraphCreate {
        graph_name: name.into(),
      }
      .render()?
      .as_str(),
      (),
    )?;
    Ok(())
  }
  fn create_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    transaction: &mut Self::Transaction,
    graph_name: &String,
    nodes_iter: T,
  ) -> Result<()>
  {
    for x in nodes_iter
    {
      transaction.connection.execute(
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
      );
    }
    Ok(())
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
