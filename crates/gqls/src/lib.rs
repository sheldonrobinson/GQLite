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

/// Base trait for Elements
pub trait Element: Sync + Send
{
  /// Access to the query interface
  fn query_interface(&self) -> &dyn QueryInterface;
  /// Access to the key referencing the element
  fn element_key(&self) -> graphcore::Key;
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
  }
}
