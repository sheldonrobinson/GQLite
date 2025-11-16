#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]

mod error;
pub mod gqls;
pub mod prelude;

pub use error::Error;

/// Result type used by this crate
pub type Result<T, E = error::Error> = std::result::Result<T, E>;
