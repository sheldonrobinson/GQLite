pub(crate) use graphcore::SinglePath;
pub use graphcore::{Edge, Key, Node, SinglePath as Path};

#[cfg(test)]
pub(crate) use graphcore::labels;

#[derive(Debug, Clone, Copy)]
pub(crate) enum EdgeDirectivity
{
  Undirected,
  Directed,
}
