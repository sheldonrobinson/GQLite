#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]
#![allow(clippy::result_large_err)]

mod error;
mod graph;
mod prelude;
mod serialize_with;
mod table;
mod timestamp;
mod value;

pub use error::Error;
pub use graph::{Edge, Key, Node, SinglePath};
pub use table::Table;
pub use timestamp::TimeStamp;
pub use value::{Value, ValueMap, ValueTryIntoRef};
