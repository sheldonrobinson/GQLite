//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) gqb
//! ===================================================================
//! `graph query builder (gqb)` is a crate with a builder for conveniently create queries using a high-level API.

#![deny(missing_docs)]
#![deny(warnings)]

use std::collections::HashMap;

#[derive(Debug, Default)]
struct CreateStatement
{
  nodes: Vec<graphcore::Key>,
  edges: Vec<graphcore::Key>,
}

#[derive(Debug)]
enum Statement
{
  Create(CreateStatement),
}

/// Structure for building queries.
#[derive(Debug, Default)]
pub struct Builder
{
  statements: Vec<Statement>,
  nodes: HashMap<graphcore::Key, graphcore::Node>,
  edges: HashMap<graphcore::Key, (graphcore::Key, graphcore::Edge, graphcore::Key)>,
}

/// Trait for multi-node creation.
pub trait CreateNodes
{
  /// Output of multi-node creation
  type Output;
  /// Fill the builder
  fn fill(self, builder: &mut Builder) -> Self::Output;
}

/// Trait for multi-edge creation.
pub trait CreateEdges
{
  /// Output of multi-edge creation
  type Output;
  /// Fill the builder
  fn fill(self, builder: &mut Builder) -> Self::Output;
}

macro_rules! __key {
  ($idx:tt) => {
    graphcore::Key
  };
}

macro_rules! impl_create_nodes {
  ($n:tt $($idx:tt $l:ident $p:ident),*) => {
      impl<$($l, $p),*> CreateNodes
          for ($(($l, $p),)*)
      where
          $($l: Into<Vec<String>>,
            $p: Into<graphcore::ValueMap>),*
      {
          type Output = ($(__key!($idx),)*);

          fn fill(self, builder: &mut Builder) -> Self::Output {
              ($(
                  builder.create_node(
                      (self.$idx).0,
                      (self.$idx).1,
                  ),
              )*)
          }
      }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all_create_nodes {
  ($($n:tt $($idx:tt $l:ident $p:ident),*;)*) => {
      $(impl_create_nodes!($n $($idx $l $p),*);)*
  };
}

impl_all_create_nodes! {
  1 0 L0 P0;
  2 0 L0 P0, 1 L1 P1;
  3 0 L0 P0, 1 L1 P1, 2 L2 P2;
  4 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3;
  5 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4;
  6 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5;
  7 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6;
  8 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7;
  9 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8;
  10 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9;
  11 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10;
  12 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11;
  13 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12;
  14 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13;
  15 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14;
  16 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15;
  17 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16;
  18 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17;
  19 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17, 18 L18 P18;
  20 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17, 18 L18 P18, 19 L19 P19;
}

macro_rules! impl_create_edges {
  ($n:tt $($idx:tt $s:ident $l:ident $p:ident $d:ident),*) => {
      impl<$($s, $l, $p, $d),*> CreateEdges
          for ($(($s, $l, $p, $d),)*)
      where
          $($s: Into<graphcore::Key>,
            $l: Into<Vec<String>>,
            $p: Into<graphcore::ValueMap>,
            $d: Into<graphcore::Key>),*
      {
          type Output = ($(__key!($idx),)*);

          fn fill(self, builder: &mut Builder) -> Self::Output {
              ($(
                  builder.create_edge(
                      (self.$idx).0,
                      (self.$idx).1,
                      (self.$idx).2,
                      (self.$idx).3,
                  ),
              )*)
          }
      }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all {
  ($($n:tt $($idx:tt $s:ident $l:ident $p:ident $d:ident),*;)*) => {
      $(impl_create_edges!($n $($idx $s $l $p $d),*);)*
  };
}

impl_all! {
  1 0 S0 L0 P0 D0;
  2 0 S0 L0 P0 D0, 1 S1 L1 P1 D1;
  3 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2;
  4 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3;
  5 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4;
  6 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5;
  7 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6;
  8 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7;
  9 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8;
  10 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9;
  11 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10;
  12 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11;
  13 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12;
  14 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13;
  15 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14;
  16 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15;
  17 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16;
  18 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17;
  19 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17, 18 S18 L18 P18 D18;
  20 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17, 18 S18 L18 P18 D18, 19 S19 L19 P19 D19;
}

impl Builder
{
  fn last_create_statement(&mut self) -> &mut CreateStatement
  {
    if self.statements.is_empty()
    {
      self
        .statements
        .push(Statement::Create(CreateStatement::default()));
    }
    match self.statements.last_mut().unwrap()
    {
      Statement::Create(statement) => statement,
    }
  }
  /// Create a node
  pub fn create_node(
    &mut self,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
  ) -> graphcore::Key
  {
    let key = Default::default();
    let node = graphcore::Node::new(key, labels.into(), properties.into());
    let cs = self.last_create_statement();
    cs.nodes.push(key);
    self.nodes.insert(key, node);
    key
  }
  /// Create multiple nodes
  pub fn create_nodes<T>(&mut self, t: T) -> T::Output
  where
    T: CreateNodes,
  {
    t.fill(self)
  }
  /// Return a reference to a node
  pub fn node_ref(&self, key: &graphcore::Key) -> Option<&graphcore::Node>
  {
    self.nodes.get(key)
  }
  /// Create an edge
  pub fn create_edge(
    &mut self,
    source: impl Into<graphcore::Key>,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
    destination: impl Into<graphcore::Key>,
  ) -> graphcore::Key
  {
    let key = Default::default();
    let edge = (
      source.into(),
      graphcore::Edge::new(key, labels.into(), properties.into()),
      destination.into(),
    );
    let cs = self.last_create_statement();
    cs.edges.push(key);
    self.edges.insert(key, edge);
    key
  }
  /// Create multiple edges
  pub fn create_edges<T>(&mut self, t: T) -> T::Output
  where
    T: CreateEdges,
  {
    t.fill(self)
  }
  /// Return a reference to an edge
  pub fn edge_ref(&self, key: &graphcore::Key) -> Option<&graphcore::Edge>
  {
    self.edges.get(key).map(|(_, x, _)| x)
  }
}

#[cfg(test)]
mod test
{
  use super::Builder;
  use graphcore::*;
  #[test]
  fn test_create()
  {
    let mut b = Builder::default();
    let n1 = b.create_node(labels!("a"), ValueMap::default());
    assert_eq!(*b.node_ref(&n1).unwrap().labels(), labels!("a"));
    let (n2, n3) = b.create_nodes((
      (labels!("b"), ValueMap::default()),
      (labels!("c"), ValueMap::default()),
    ));
    assert_eq!(*b.node_ref(&n2).unwrap().labels(), labels!("b"));
    assert_eq!(*b.node_ref(&n3).unwrap().labels(), labels!("c"));
    let e = b.create_edge(n1, labels!("d"), ValueMap::default(), n2);
    assert_eq!(*b.edge_ref(&e).unwrap().labels(), labels!("d"));
    let (e0, e1) = b.create_edges((
      (n1, labels!("e"), ValueMap::default(), n3),
      (n2, labels!("f"), ValueMap::default(), n3),
    ));
    assert_eq!(*b.edge_ref(&e0).unwrap().labels(), labels!("e"));
    assert_eq!(*b.edge_ref(&e1).unwrap().labels(), labels!("f"));
  }
}
