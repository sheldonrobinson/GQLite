pub(crate) use graphcore::SinglePath;
pub use graphcore::{Edge, Key, Node, SinglePath as Path};

pub use graphcore::labels;

#[derive(Debug, Clone, Copy)]
pub(crate) enum EdgeDirectivity
{
  Undirected,
  Directed,
}
