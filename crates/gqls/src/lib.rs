#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]
pub use gqls_macros::generate_module;

pub use anyhow;
pub use gqb;
pub use graphcore;

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
  Edge
}

/// Base trait for Elements
pub trait Element: Sync + Send
{
  /// Access to the query interface
  fn query_interface(&self) -> &dyn QueryInterface;
  /// Access to the key referencing the element
  fn element_key(&self) -> graphcore::Key;
  /// Access the type of the element
  fn element_type(&self) -> ElementType;
}

pub trait Node: Element
{
  /// Create the element from the key
  fn from_key(key: graphcore::Key, query_interface: Box<dyn QueryInterface>) -> Self;
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

  #[test]
  #[cfg(feature = "gqlite")]
  fn it_works()
  {
    use gqb::{labels, value_map};
    use gqlitedb::TimeStamp;

    let connection = Arc::new(gqlitedb::Connection::create(Default::default()).unwrap());
    let graph = test_module::Graph::new(connection.clone());

    use test_module::elements::*;

    // Test creating person
    let bob_marley = graph.create_person("Bob", "Marley").unwrap();
    assert_eq!(bob_marley.first_name().unwrap(), "Bob".to_string());
    assert_eq!(bob_marley.last_name().unwrap(), "Marley".to_string());
    let r = connection
      .execute_oc_query("MATCH (n:Person) RETURN n", value_map!())
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
      .execute_oc_query("MATCH (n:Post) RETURN n", value_map!())
      .unwrap();
    let t = r.try_into_table().unwrap();
    assert_eq!(t.rows(), 1);
    let n: graphcore::Node = t.value(0, 0).unwrap().try_into().unwrap();
    assert_eq!(*n.labels(), labels!["Message", "Post"]);
    assert_eq!(
      *n.properties(),
      value_map!("creationDate" => format!("{}", timestamp), "browserUsed" => "FireGoupil", "imageFile" => ())
    );

    // Test creating edge
    let edge_bm_post = graph.create_likes(&bob_marley, &post).unwrap();
    assert_eq!(
      edge_bm_post.source().element_key(),
      bob_marley.element_key()
    );
    assert_eq!(edge_bm_post.destination().element_key(), post.element_key());
    let r = connection
      .execute_oc_query("MATCH p = ()-[]->() RETURN p", Default::default())
      .unwrap();
    let t = r.try_into_table().unwrap();
    assert_eq!(t.rows(), 1);
    let edge_q: &graphcore::SinglePath = &t.get(0, 0).unwrap();
    assert_eq!(edge_q.source().key(), bob_marley.element_key());
    assert_eq!(edge_q.key(), edge_bm_post.element_key());
    assert_eq!(edge_q.destination().key(), post.element_key());

    // Create an other edge
    let edge_bm_bm = graph
      .create_knows(&bob_marley, &bob_marley, timestamp.clone())
      .unwrap();
    assert_eq!(edge_bm_bm.source().element_key(), bob_marley.element_key());
    assert_eq!(
      edge_bm_bm.destination().element_key(),
      bob_marley.element_key()
    );
    assert_eq!(edge_bm_bm.creation_date().unwrap(), timestamp);
  }
}
