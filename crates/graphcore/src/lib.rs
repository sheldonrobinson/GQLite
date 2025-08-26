//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) graphcore
//! ======================================================================
//!
//! Core data structure for representing and manipulating property graphs.

#![warn(missing_docs)]
#![deny(warnings)]
#![allow(clippy::result_large_err)]

mod error;
mod graph;
mod prelude;
mod serialize_with;
mod table;
mod value;

pub use error::Error;
pub use graph::{Edge, Key, Node, SinglePath};
pub use table::Table;
pub use value::{Value, ValueMap, ValueTryIntoRef};
