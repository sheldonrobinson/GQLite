use std::collections::BTreeMap;

use crate::{functions, graph};

#[derive(Debug)]
pub(crate) enum Instruction
{
  CreateNodeLiteral
  {
    labels: Vec<String>,
  },
  CreateEdgeLiteral
  {
    labels: Vec<String>,
  },
  FunctionCall
  {
    function: functions::Function,
    arguments_count: usize,
  },
  Push
  {
    value: crate::graph::Value,
  },
  GetVariable
  {
    name: String,
  },
  GetParameter
  {
    name: String,
  },
  CreateArray
  {
    length: usize,
  },
  CreateMap
  {
    keys: Vec<String>,
  },
  MemberAccess
  {
    path: Vec<String>,
  },
  Duplicate,
  Rot3,        // If a is the top of the stack, then a b c -> b c a
  InverseRot3, // If a is the top of the stack, then a b c -> c a b
  Swap,
  Drop,
  AndBinaryOperator,
  OrBinaryOperator,
  NotUnaryOperator,
  EqualBinaryOperator,
  NotEqualBinaryOperator,
}

pub(crate) type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub(crate) struct CreateAction
{
  pub(crate) instructions: Instructions,
  pub(crate) variables: Vec<Option<String>>,
}

#[derive(Debug)]
pub(crate) enum Block
{
  Create
  {
    actions: Vec<CreateAction>
  },
  MatchNode
  {
    instructions: Instructions,
    variable: Option<String>,
    filter: Instructions,
  },
  MatchEdge
  {
    instructions: Instructions,
    left_variable: Option<String>,
    edge_variable: Option<String>,
    right_variable: Option<String>,
    path_variable: Option<String>,
    filter: Instructions,
    directivity: graph::EdgeDirectivity,
  },
  Return
  {
    variables: Vec<(String, Instructions)>,
  },
  Call
  {
    arguments: Instructions,
    name: String,
  },
  With
  {
    all: bool,
    variables: Vec<(String, Instructions)>,
  },
  Unwind
  {
    name: String,
    instructions: Instructions,
  },
}
