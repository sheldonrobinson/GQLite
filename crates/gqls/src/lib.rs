#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]

use std::collections::HashSet;

pub use anyhow;
pub use gqb;
pub use graphcore;

#[cfg(feature = "rune")]
pub mod rune;

mod generic_node;

pub use gqls_macros::generate_module;
#[cfg(feature = "rune")]
pub use gqls_macros::generate_rune_module;

pub use generic_node::GenericNode;

/// True if every &str in b occurs as a String in a (duplicates ignored).
pub fn contains_all(a: &[String], b: &[&str]) -> bool
{
  let set: HashSet<&str> = a.iter().map(String::as_str).collect();
  b.iter().all(|&s| set.contains(s))
}
/// Query interface to a database.
pub trait QueryInterface: Sync + Send
{
  /// Execute the query from the builder.
  fn execute_builder(
    &self,
    builder: gqb::Builder,
  ) -> Result<Option<graphcore::Table>, anyhow::Error>;
  /// Clone the interface
  fn clone_interface(&self) -> Box<dyn QueryInterface>;
}

/// Type of the element
pub enum ElementType
{
  /// It is a node
  Node,
  /// It is an edge
  Edge,
}

/// Base trait for Elements
pub trait Element: Sync + Send
{
  /// Access to the query interface
  fn query_interface(&self) -> &dyn QueryInterface;
  /// Name of the graph where the element is stored
  fn graph_name(&self) -> &String;
  /// Access to the key referencing the element
  fn element_key(&self) -> graphcore::Key;
  /// Access the type of the element
  fn element_type(&self) -> ElementType;
}

/// Base trait for nodes
pub trait Node: Element + Sized
{
  /// Create the element from the key
  fn from_node(
    node: graphcore::Node,
    query_interface: Box<dyn QueryInterface>,
    graph_name: impl Into<String>,
  ) -> Result<Self, anyhow::Error>;
  /// Convert into a generic node
  fn into_generic_node(self) -> GenericNode;
  /// Vector of labels for the node
  fn labels(node: Option<&Self>) -> Vec<String>;
}

/// Base trait for edges
pub trait Edge: Element
{
  /// Vector of labels for the node
  fn labels() -> Vec<String>;
}

#[cfg(feature = "gqlite")]
mod gqlitedb_impl
{
  use std::sync::Arc;

  use crate::QueryInterface;

  impl QueryInterface for Arc<gqlitedb::Connection>
  {
    fn execute_builder(
      &self,
      builder: gqb::Builder,
    ) -> Result<Option<graphcore::Table>, anyhow::Error>
    {
      let (q, b) = builder.into_oc_query()?;
      let qr = self.execute_oc_query(q, b)?;
      match qr
      {
        gqlitedb::QueryResult::Table(tbl) => Ok(Some(tbl)),
        _ => Ok(None),
      }
    }
    fn clone_interface(&self) -> Box<dyn QueryInterface>
    {
      Box::new(self.clone())
    }
  }
}

#[cfg(test)]
mod tests
{

  use super::*;
  use std::sync::Arc;

  generate_module!(test_module, "test.gqls");
  #[cfg(feature = "rune")]
  generate_rune_module!(test_rune_module, crate::tests::test_module, "test.gqls");

  #[test]
  #[cfg(feature = "gqlite")]
  fn test_orm_rust_api()
  {
    use gqb::{labels, value_map};
    use gqlitedb::TimeStamp;

    let connection = Arc::new(gqlitedb::Connection::create(Default::default()).unwrap());
    let graph = test_module::Graph::new(connection.clone(), "test").unwrap();

    use test_module::elements::*;

    // Test creating person
    let bob_marley = graph.create_person("Bob", "Marley").unwrap();
    assert_eq!(bob_marley.first_name().unwrap(), "Bob".to_string());
    assert_eq!(bob_marley.last_name().unwrap(), "Marley".to_string());
    let r = connection
      .execute_oc_query("USE test MATCH (n:Person) RETURN n", value_map!())
      .unwrap();
    let t = r.try_into_table().unwrap();
    assert_eq!(t.rows(), 1);
    let n: graphcore::Node = t.value(0, 0).unwrap().try_into().unwrap();
    assert_eq!(*n.labels(), labels!["Person"]);
    assert_eq!(
      *n.properties(),
      value_map!("firstName" => "Bob", "lastName" => "Marley")
    );

    // Test creating Post
    let timestamp = TimeStamp::parse("2024-06-15T07:00:00.123000-00:00[GMT]").unwrap();
    let post = graph
      .create_post(timestamp.clone(), "FireGoupil", None)
      .unwrap();
    assert_eq!(post.creation_date().unwrap(), timestamp);
    assert_eq!(post.browser_used().unwrap(), "FireGoupil".to_string());
    assert!(post.image_file().unwrap().is_none());
    let r = connection
      .execute_oc_query("USE test MATCH (n:Post) RETURN n", value_map!())
      .unwrap();
    let t = r.try_into_table().unwrap();
    assert_eq!(t.rows(), 1);
    let n: graphcore::Node = t.value(0, 0).unwrap().try_into().unwrap();
    assert_eq!(*n.labels(), labels!["Message", "Post"]);
    assert_eq!(
      *n.properties(),
      value_map!("creationDate" => format!("{}", timestamp), "browserUsed" => "FireGoupil", "imageFile" => ())
    );

    // Test matching
    let persons = graph.match_person().unwrap();
    assert_eq!(persons.len(), 1);
    let bob_marley_2 = &persons[0];
    assert_eq!(bob_marley_2.first_name().unwrap(), "Bob");
    assert_eq!(bob_marley_2.last_name().unwrap(), "Marley");

    // Test editing property
    let timestamp = TimeStamp::parse("2024-06-14T07:10:05.123000-00:00[GMT]").unwrap();
    post.set_creation_date(timestamp.clone()).unwrap();
    assert_eq!(post.creation_date().unwrap(), timestamp);

    // Test creating edge
    let edge_bm_post = graph
      .create_likes(bob_marley.clone(), post.clone())
      .unwrap();
    assert_eq!(
      edge_bm_post.source().element_key(),
      bob_marley.element_key()
    );
    assert_eq!(edge_bm_post.destination().element_key(), post.element_key());
    let r = connection
      .execute_oc_query("USE test MATCH p = ()-[]->() RETURN p", Default::default())
      .unwrap();
    let t = r.try_into_table().unwrap();
    assert_eq!(t.rows(), 1);
    let edge_q: &graphcore::SinglePath = &t.get(0, 0).unwrap();
    assert_eq!(edge_q.source().key(), bob_marley.element_key());
    assert_eq!(edge_q.key(), edge_bm_post.element_key());
    assert_eq!(edge_q.destination().key(), post.element_key());

    // Create an other edge
    let edge_bm_bm = graph
      .create_knows(bob_marley.clone(), bob_marley.clone(), timestamp.clone())
      .unwrap();
    assert_eq!(edge_bm_bm.source().element_key(), bob_marley.element_key());
    assert_eq!(
      edge_bm_bm.destination().element_key(),
      bob_marley.element_key()
    );
    assert_eq!(edge_bm_bm.creation_date().unwrap(), timestamp);

    // Test matching edge
    let edge_q_vec = graph
      .match_knows::<test_module::nodes::Person, test_module::nodes::Person>(None, None)
      .unwrap();
    assert_eq!(edge_q_vec.len(), 1);
    let edge_q = &edge_q_vec[0];
    assert_eq!(edge_q.source().element_key(), bob_marley.element_key());
    assert_eq!(edge_q.element_key(), edge_bm_bm.element_key());
    assert_eq!(edge_q.destination().element_key(), bob_marley.element_key());

    // Test deletion
    graph.delete_edge(edge_bm_bm).unwrap();
    graph.delete_node(bob_marley).unwrap();
  }
  #[test]
  #[cfg(all(feature = "gqlite", feature = "rune"))]
  fn test_orm_rune_api()
  {
    let tester = ccutils::rune::testing::Tester::new(|rune_context| {
      rune_context
        .install(gqliterune::gqlite_module().unwrap())
        .unwrap();
      test_rune_module::install(rune_context).unwrap();
    });
    tester.eval::<()>(include_str!("test.rn")).unwrap();
  }
}
