use std::collections::{BTreeMap, HashMap};

use crate::{aggregators, functions, graph};

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
  CreateNodeQuery
  {
    labels: Vec<String>,
  },
  CreateEdgeQuery
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
  XorBinaryOperator,
  NegationUnaryOperator,
  NotUnaryOperator,
  IsNullUnaryOperator,
  EqualBinaryOperator,
  NotEqualBinaryOperator,
  InferiorBinaryOperator,
  SuperiorBinaryOperator,
  InferiorEqualBinaryOperator,
  SuperiorEqualBinaryOperator,
  InBinaryOperator,
  NotInBinaryOperator,
  AdditionBinaryOperator,
  SubstractionBinaryOperator,
  MultiplicationBinaryOperator,
  DivisionBinaryOperator,
  ModuloBinaryOperator,
}

pub(crate) type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub(crate) struct CreateAction
{
  pub(crate) instructions: Instructions,
  pub(crate) variables: Vec<Option<String>>,
}

#[derive(Debug)]
pub(crate) enum BlockMatch
{
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
}

#[derive(Debug)]
pub(crate) struct RWAggregation
{
  pub(crate) init_instructions: Instructions,
  pub(crate) argument_instructions: Instructions,
  pub(crate) aggregator: aggregators::Aggregator,
}

#[derive(Debug)]
pub(crate) struct RWExpression
{
  pub(crate) name: String,
  pub(crate) instructions: Instructions,
  pub(crate) aggregations: HashMap<String, RWAggregation>,
}

#[derive(Debug)]
pub(crate) enum UpdateOne
{
  SetProperty
  {
    target: String,
    path: Vec<String>,
    instructions: Instructions,
  },
  AddProperty
  {
    target: String,
    path: Vec<String>,
    instructions: Instructions,
  },
  RemoveProperty
  {
    target: String, path: Vec<String>
  },
  AddLabels
  {
    target: String, labels: Vec<String>
  },
}

#[derive(Debug)]
pub(crate) enum Block
{
  Create
  {
    actions: Vec<CreateAction>
  },
  BlockMatch
  {
    blocks: Vec<BlockMatch>,
    filter: Instructions,
    optional: bool,
  },
  Return
  {
    variables: Vec<RWExpression>
  },
  Call
  {
    arguments: Instructions,
    name: String,
  },
  With
  {
    all: bool,
    variables: Vec<RWExpression>,
  },
  Unwind
  {
    name: String,
    instructions: Instructions,
  },
  Delete
  {
    detach: bool,
    instructions: Vec<Instructions>,
  },
  Update
  {
    updates: Vec<UpdateOne>
  },
}
