//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite
//!
//! Implementation of GQL (Graph Query Language), embeddable in applications.

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

/// GQLite error
pub type Error = error::Error;

/// GQLite Result
pub type Result<T, E = error::export::Error> = std::result::Result<T, E>;

/// GQLite Connection
pub type Connection = connection::Connection;

/// GQLite Value
pub type Value = value::Value;

/// GQLite ValueMap
pub type ValueMap = value::ValueMap;
