#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]

mod expression;
pub mod prelude;
mod utils;

use itertools::Itertools as _;
use std::borrow::Borrow;

pub use graphcore::{array, labels, value_map, ValueMap};

use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Error
{
  #[error("{0}")]
  GraphCore(#[from] graphcore::Error),
  #[error("Askama: {0}")]
  AskamaError(#[from] askama::Error),
  #[error("Unknwon node.")]
  UnknownNode,
  #[error("Unknwon edge.")]
  UnknownEdge,
}

type Result<T, E = Error> = std::result::Result<T, E>;

use askama::Template;

/// Trait for passing variables to a function.
pub trait Variables
{
  /// Fill the vector of variables
  fn fill(self, vector: &mut Vec<Variable>);
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

impl Variables for Variable
{
  fn fill(self, vector: &mut Vec<Variable>)
  {
    vector.push(self);
  }
}

impl Variables for Vec<Variable>
{
  fn fill(mut self, vector: &mut Vec<Variable>)
  {
    vector.append(&mut self);
  }
}

macro_rules! __key {
  ($idx:tt) => {
    Variable
  };
}

macro_rules! impl_variables {
  ($n:tt $($idx:tt),*) => {
      impl Variables
          for ($(__key!($idx),)*)
      {
          fn fill(self, vector: &mut Vec<Variable>) {
              $(
                  vector.push(self.$idx);
              )*
          }
      }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all_variables {
  ($($n:tt $($idx:tt),*;)*) => {
      $(impl_variables!($n $($idx),*);)*
  };
}

impl_all_variables! {
  2 0, 1;
  3 0, 1, 2;
  4 0, 1, 2, 3;
  5 0, 1, 2, 3, 4;
  6 0, 1, 2, 3, 4, 5;
  7 0, 1, 2, 3, 4, 5, 6;
  8 0, 1, 2, 3, 4, 5, 6, 7;
  9 0, 1, 2, 3, 4, 5, 6, 7, 8;
  10 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
  11 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10;
  12 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11;
  13 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12;
  14 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13;
  15 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14;
  16 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15;
  17 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16;
  18 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17;
  19 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18;
  20 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19;
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

impl CreateNodes for Vec<(Vec<String>, graphcore::ValueMap)>
{
  type Output = Vec<Variable>;
  fn fill(self, builder: &mut Builder) -> Self::Output
  {
    let mut out = Vec::<Variable>::new();
    for (l, p) in self
    {
      out.push(builder.create_node(l, p))
    }
    out
  }
}

impl CreateEdges for Vec<(Variable, Vec<String>, graphcore::ValueMap, Variable)>
{
  type Output = Vec<Variable>;
  fn fill(self, builder: &mut Builder) -> Self::Output
  {
    let mut out = Vec::<Variable>::new();
    for (s, l, p, d) in self
    {
      out.push(builder.create_edge(s, l, p, d));
    }
    out
  }
}

macro_rules! impl_create_edges {
  ($n:tt $($idx:tt $s:ident $l:ident $p:ident $d:ident),*) => {
      impl<$($s, $l, $p, $d),*> CreateEdges
          for ($(($s, $l, $p, $d),)*)
      where
          $($s: Into<Variable>,
            $l: Into<Vec<String>>,
            $p: Into<graphcore::ValueMap>,
            $d: Into<Variable>),*
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

mod templates
{
  use askama::Template;

  use crate::Variable;

  // Graph related templates
  #[derive(Template)]
  #[template(path = "oc/node_pattern.oc", escape = "none")]
  pub(super) struct NodePattern<'a>
  {
    pub var: &'a Variable,
    pub labels: &'a Vec<String>,
    pub properties_binding: &'a String,
  }
  #[derive(Template)]
  #[template(path = "oc/edge_pattern.oc", escape = "none")]
  pub(super) struct EdgePattern<'a>
  {
    pub source: &'a Variable,
    pub var: &'a Variable,
    pub labels: &'a Vec<String>,
    pub properties_binding: &'a String,
    pub destination: &'a Variable,
  }
  #[derive(Template)]
  #[template(path = "oc/delete.oc", escape = "none")]
  pub(super) struct Delete<'a>
  {
    pub variables: &'a Vec<Variable>,
  }
}

/// Hold the name of a variable in the query.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct Variable
{
  suffix: &'static str,
  id: usize,
}

impl std::fmt::Display for Variable
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{}{}", self.suffix, self.id)
  }
}

#[derive(Debug, Default)]
struct CreateStatement
{
  nodes: Vec<(Variable, graphcore::Node)>,
  edges: Vec<(Variable, (Variable, graphcore::Edge, Variable))>,
}

#[derive(Debug, Default)]
struct MatchStatement
{
  nodes: Vec<(Variable, graphcore::Node)>,
  edges: Vec<(Variable, (Variable, graphcore::Edge, Variable))>,
}

#[derive(Debug)]
struct WhereStatement
{
  expression: Box<Expression>,
}

#[derive(Debug, Default)]
struct ReturnStatement
{
  values: Vec<(Variable, Option<String>, String)>,
}

#[derive(Debug, Default)]
struct DeleteStatement
{
  variables: Vec<Variable>,
}

#[derive(Debug)]
enum Statement
{
  Create(CreateStatement),
  Match(MatchStatement),
  Where(WhereStatement),
  Return_(ReturnStatement),
  Delete(DeleteStatement),
}

/// Structure for building queries.
#[derive(Debug, Default)]
pub struct Builder
{
  statements: Vec<Statement>,
  variables_count: usize,
}

macro_rules! last_statement {
  ($fname: ident, $statement: ty, $ename: path) => {
    fn $fname(&mut self) -> &mut $statement
    {
      // Ensure the last statement is a $statement
      let needs_push = match self.statements.last()
      {
        Some($ename(_)) => false,
        _ => true,
      };

      if needs_push
      {
        self.statements.push($ename(<$statement>::default()));
      }

      // Single, non-overlapping mutable borrow
      match self.statements.last_mut().unwrap()
      {
        $ename(statement) => statement,
        _ => unreachable!(), // we just pushed a statement above
      }
    }
  };
}

impl Builder
{
  last_statement! {last_create_statement, CreateStatement, Statement::Create}
  last_statement! {last_match_statement, MatchStatement, Statement::Match}
  last_statement! {last_return_statement, ReturnStatement, Statement::Return_}
  last_statement! {last_delete_statement, DeleteStatement, Statement::Delete}

  fn next_variable(&mut self, suffix: &'static str) -> Variable
  {
    self.variables_count += 1;
    Variable {
      suffix,
      id: self.variables_count,
    }
  }
  /// Create a node variable
  pub fn node_variable(&mut self) -> Variable
  {
    self.next_variable("n")
  }
  /// Create a node
  pub fn create_node(
    &mut self,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
  ) -> Variable
  {
    let var = self.node_variable();
    let node = graphcore::Node::new(Default::default(), labels.into(), properties.into());
    let cs = self.last_create_statement();
    cs.nodes.push((var, node));
    var
  }
  /// Create multiple nodes
  pub fn create_nodes<T>(&mut self, t: T) -> T::Output
  where
    T: CreateNodes,
  {
    t.fill(self)
  }
  /// Create an edge
  pub fn create_edge(
    &mut self,
    source: impl Into<Variable>,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
    destination: impl Into<Variable>,
  ) -> Variable
  {
    let var = self.next_variable("e");
    let edge = (
      source.into(),
      graphcore::Edge::new(Default::default(), labels.into(), properties.into()),
      destination.into(),
    );
    let cs = self.last_create_statement();
    cs.edges.push((var, edge));
    var
  }
  /// Create multiple edges
  pub fn create_edges<T>(&mut self, t: T) -> T::Output
  where
    T: CreateEdges,
  {
    t.fill(self)
  }
  /// Select a node
  pub fn match_node(
    &mut self,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
  ) -> Variable
  {
    let var = self.node_variable();
    let ss = self.last_match_statement();
    ss.nodes.push((
      var,
      graphcore::Node::new(Default::default(), labels.into(), properties.into()),
    ));
    var
  }
  /// Select an edge
  pub fn match_edge(
    &mut self,
    source: impl Into<Option<Variable>>,
    labels: impl Into<Vec<String>>,
    properties: impl Into<graphcore::ValueMap>,
    destination: impl Into<Option<Variable>>,
  ) -> Variable
  {
    let var = self.next_variable("e");
    let source = source.into().unwrap_or_else(|| self.next_variable("n"));
    let destination = destination
      .into()
      .unwrap_or_else(|| self.next_variable("n"));
    let ss = self.last_match_statement();
    ss.edges.push((
      var,
      (
        source,
        graphcore::Edge::new(Default::default(), labels.into(), properties.into()),
        destination,
      ),
    ));
    var
  }
  /// Add a where statement
  pub fn where_statement(&mut self, expression: Expression)
  {
    let expression = Box::new(expression);
    self
      .statements
      .push(Statement::Where(WhereStatement { expression }));
  }
  /// Add a return statement to the query for the given variable, accessible in the column with the given name
  pub fn return_variable(&mut self, variable: Variable, name: impl Into<String>)
  {
    self
      .last_return_statement()
      .values
      .push((variable, None, name.into()));
  }
  /// Add a return statement to the query for the given variable, accessible in the column with the given name
  pub fn return_property<I, S>(&mut self, variable: Variable, path: I, name: impl Into<String>)
  where
    I: IntoIterator<Item = S>,
    S: Borrow<str> + std::fmt::Display,
  {
    self.last_return_statement().values.push((
      variable,
      Some(path.into_iter().join(".")),
      name.into(),
    ));
  }
  /// Add a delete statement for variables
  pub fn delete(&mut self, variables: impl Variables)
  {
    let delete_statement = self.last_delete_statement();
    variables.fill(&mut delete_statement.variables);
  }
  /// Generate an OpenCypher Query.
  pub fn into_oc_query(self) -> Result<(String, graphcore::ValueMap)>
  {
    let mut q = String::new();
    let mut bindings = graphcore::ValueMap::new();
    let mut space = false;
    for st in self.statements
    {
      if space
      {
        q += " ";
      }
      else
      {
        space = true;
      }
      match st
      {
        Statement::Create(create) =>
        {
          q += "CREATE ";
          let mut comma = false;
          // Create nodes
          for (var, node) in create.nodes.into_iter()
          {
            let properties_binding = format!("$b{}", bindings.len());
            if comma
            {
              q += ", "
            }
            else
            {
              comma = true;
            }
            q += templates::NodePattern {
              var: &var,
              labels: node.labels(),
              properties_binding: &properties_binding,
            }
            .render()
            .unwrap()
            .as_str();
            bindings.insert(properties_binding, node.properties().to_owned().into());
          }
          // Create edges
          for (var, (source, edge, destination)) in create.edges.into_iter()
          {
            let properties_binding = format!("$b{}", bindings.len());
            if comma
            {
              q += ", "
            }
            else
            {
              comma = true;
            }
            q += templates::EdgePattern {
              var: &var,
              source: &source,
              destination: &destination,
              labels: edge.labels(),
              properties_binding: &properties_binding,
            }
            .render()
            .unwrap()
            .as_str();
            bindings.insert(properties_binding, edge.properties().to_owned().into());
          }
        }
        Statement::Match(select) =>
        {
          q += "MATCH ";
          let mut comma = false;
          // Create nodes
          for (var, node) in select.nodes.into_iter()
          {
            let properties_binding = format!("$b{}", bindings.len());
            if comma
            {
              q += ", "
            }
            else
            {
              comma = true;
            }
            q += templates::NodePattern {
              var: &var,
              labels: node.labels(),
              properties_binding: &properties_binding,
            }
            .render()
            .unwrap()
            .as_str();
            bindings.insert(properties_binding, node.properties().to_owned().into());
          }
          // Create edges
          for (var, (source, edge, destination)) in select.edges.into_iter()
          {
            let properties_binding = format!("$b{}", bindings.len());
            if comma
            {
              q += ", "
            }
            else
            {
              comma = true;
            }
            q += templates::EdgePattern {
              var: &var,
              source: &source,
              destination: &destination,
              labels: edge.labels(),
              properties_binding: &properties_binding,
            }
            .render()
            .unwrap()
            .as_str();
            bindings.insert(properties_binding, edge.properties().to_owned().into());
          }
        }
        Statement::Where(where_statment) =>
        {
          q += &format!(
            "WHERE {}",
            where_statment.expression.into_oc_query(&mut bindings)
          )
        }
        Statement::Return_(return_statement) =>
        {
          q += "RETURN ";
          let mut comma = false;
          // Create nodes
          for (var, path, name) in return_statement.values.into_iter()
          {
            if comma
            {
              q += ", "
            }
            else
            {
              comma = true;
            }
            match path
            {
              Some(path) => q += &format!("{}.{} AS {}", var, path, name),
              None => q += &format!("{} AS {}", var, name),
            }
          }
        }
        Statement::Delete(delete_statement) =>
        {
          q += templates::Delete {
            variables: &delete_statement.variables,
          }
          .render()
          .unwrap()
          .as_str();
        }
      }
    }

    Ok((q, bindings))
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
    let (n2, n3) = b.create_nodes((
      (labels!("b"), ValueMap::default()),
      (labels!("c"), ValueMap::default()),
    ));
    let _ = b.create_edge(n1, labels!("d"), ValueMap::default(), n2);
    let (_, _) = b.create_edges((
      (n1, labels!("e"), ValueMap::default(), n3),
      (n2, labels!("f"), ValueMap::default(), n3),
    ));
    let (q, b) = b.into_oc_query().unwrap();
    assert_eq!(q, "CREATE (n1:a $b0), (n2:b $b1), (n3:c $b2), (n1)-[e4:d $b3]->(n2), (n1)-[e5:e $b4]->(n3), (n2)-[e6:f $b5]->(n3)".to_string());
    assert_eq!(
      b,
      value_map!("$b0" => value_map!(), "$b1" => value_map!(), "$b2" => value_map!(), "$b3" => value_map!(), "$b4" => value_map!(), "$b5" => value_map!(), )
    );
  }
  #[test]
  fn test_to_oc_query()
  {
    let connection = gqlitedb::Connection::builder().create().unwrap();
    let mut b = Builder::default();
    let (n2, n3) = b.create_nodes((
      (labels!("b"), value_map!("id" => 3)),
      (labels!("c"), ValueMap::default()),
    ));
    b.create_edge(n2, labels!("d"), ValueMap::default(), n3);
    b.create_edge(n2, labels!("e"), value_map!("id" => 5), n3);
    let (query, parameters) = b.into_oc_query().unwrap();
    connection.execute_oc_query(query, parameters).unwrap();

    let r = connection
      .execute_oc_query("MATCH (a:b) RETURN a", Default::default())
      .unwrap();
    let r: Table = r.try_into().unwrap();
    let r: &Node = r.get(0, 0).unwrap();
    assert_eq!(*r.labels(), labels!("b"));
    assert_eq!(*r.properties(), value_map!("id" => 3));

    let r = connection
      .execute_oc_query("MATCH ()-[a:e]->() RETURN a", Default::default())
      .unwrap();
    let r: Table = r.try_into().unwrap();
    let r: &Edge = r.get(0, 0).unwrap();
    assert_eq!(*r.labels(), labels!("e"));
    assert_eq!(*r.properties(), value_map!("id" => 5));
  }
  #[test]
  fn test_select()
  {
    let mut b = Builder::default();
    let n1 = b.match_node(labels!("a"), value_map!("id" => 3));
    let n2 = b.node_variable();
    let e1 = b.match_edge(n1, labels!("b"), value_map!("id" => 2), n2);
    b.return_variable(n1, "n1");
    b.return_variable(e1, "e1");
    b.return_variable(n2, "n2");
    let (q, b) = b.into_oc_query().unwrap();
    assert_eq!(
      q,
      "MATCH (n1:a $b0), (n1)-[e3:b $b1]->(n2) RETURN n1 AS n1, e3 AS e1, n2 AS n2".to_string()
    );
    assert_eq!(
      b,
      value_map!("$b0" => value_map!("id" => 3), "$b1" => value_map!("id" => 2) )
    );
  }
  #[test]
  fn test_delete()
  {
    let mut b = Builder::default();
    let n1 = b.node_variable();
    let n2 = b.node_variable();
    let n3 = b.node_variable();
    let n4 = b.node_variable();
    let n5 = b.node_variable();

    b.delete(n1);
    b.delete((n2, n3));
    b.delete(vec![n4, n5]);

    let (q, b) = b.into_oc_query().unwrap();
    assert_eq!(q, "DELETE n1, n2, n3, n4, n5".to_string());
    assert_eq!(b, value_map!());
  }
  #[test]
  fn test_where()
  {
    use crate::expression_builder as eb;
    let vid = graphcore::Key::default();
    let mut b = Builder::default();
    let v1 = b.match_node(labels!(), value_map!());
    b.where_statement(eb::equal(eb::function_call("id", (v1,)), vid.into()));
    let (q, b) = b.into_oc_query().unwrap();
    assert_eq!(q, "MATCH (n1: $b0) WHERE (id(n1)) = ($l1)".to_string());
    assert_eq!(b, value_map!("$b0" => value_map!(), "$l1" => vid));
  }
}
