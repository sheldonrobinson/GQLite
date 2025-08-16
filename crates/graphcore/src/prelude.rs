pub(crate) use crate::{error, graph, serialize_with, value};
pub(crate) use error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
