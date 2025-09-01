//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite
//! ======================================================================
//!
//! Implementation of GQL (Graph Query Language), embeddable in applications.
//!
//! Add to your crate, using `cargo add gqlitedb`. Check-out [Connection]
//! for an example of use.

#![warn(missing_docs)]
#![deny(warnings)]
#![allow(clippy::result_large_err)]
#![allow(clippy::unnecessary_lazy_evaluations)]

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
mod query_result;
mod store;
mod utils;
mod value;
mod value_table;

#[cfg(test)]
pub(crate) mod tests;

pub use {
  connection::{Backend, Connection},
  error::{CompileTimeError, Error, RunTimeError, StoreError},
  graph::{labels, Edge, Node, Path},
  query_result::QueryResult,
  value::{array, value_map, Value, ValueMap, ValueTryIntoRef},
};

pub use graphcore::{table, Table};

/// GQLite Result alias. Usable as a standard `Result<T, E>` or default to gqlite::Error with `Result<T>`
pub type Result<T, E = error::export::Error> = std::result::Result<T, E>;
