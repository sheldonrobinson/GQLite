//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite
//!
//! Implementation of GQL (Graph Query Language), embeddable in applications.

#![warn(missing_docs)]
// #![deny(warnings)]

mod capi;
mod connection;
mod error;
mod graph;
mod interpreter;
mod parser;
mod store;
mod value_table;

/// GQLite error
pub type Error = error::Error;

/// GQLite Result
pub type Result<T> = std::result::Result<T, Error>;

/// GQLite Connection
pub type Connection = connection::Connection;

/// GQLite Value
pub type Value = graph::Value;

#[cfg(test)]
pub(crate) mod tests
{
  use rand::Rng;
  pub(crate) fn get_tmp_file() -> Result<std::path::PathBuf, std::io::Error>
  {
    let mut rng = rand::thread_rng();
    loop
    {
      let rand: u32 = rng.gen();
      let path = std::path::PathBuf::from(format!(
        "{}/tmp_gqlite_{}",
        std::env::temp_dir().to_str().unwrap(),
        rand
      ));
      return Ok(path);
    }
  }
}
