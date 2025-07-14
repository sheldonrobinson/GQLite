//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite
//! ======================================================================
//!
//! Implementation of GQL (Graph Query Language), embeddable in applications.
//!
//! Add to your crate, using `cargo add gqlitedb`. Check-out [Connection]
//! for an example of use.

#![warn(missing_docs)]
#![allow(dead_code)]
#![deny(warnings)]

mod aggregators;
#[cfg(feature = "capi")]
mod capi;
mod compiler;
mod connection;
mod consts;
mod error;
mod functions;
mod graph;
mod interpreter;
mod parser;
mod prelude;
mod serialize_with;
mod store;
mod utils;
mod value;
mod value_table;

#[cfg(test)]
pub(crate) mod tests;

pub use {
  connection::Connection,
  error::{CompileTimeError, Error, RunTimeError, StoreError},
  graph::{Edge, Node, Path},
  value::{Value, ValueMap},
};

/// GQLite Result alias. Usable as a standard `Result<T, E>` or default to gqlite::Error with `Result<T>`
pub type Result<T, E = error::export::Error> = std::result::Result<T, E>;
