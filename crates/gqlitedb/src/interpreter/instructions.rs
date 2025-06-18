use std::collections::HashMap;

use crate::prelude::*;

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
    value: graph::Value,
  },
  GetVariable
  {
    col_id: value_table::ColId,
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
  IndexAccess,
  RangeAccess
  {
    start: bool,
    end: bool,
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
  pub(crate) col_id: value_table::ColId,
  pub(crate) instructions: Instructions,
  pub(crate) aggregations: HashMap<value_table::ColId, RWAggregation>,
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
  RemoveLabels
  {
    target: String, labels: Vec<String>
  },
}

#[derive(Debug)]
pub(crate) struct OrderBy
{
  pub asc: bool,
  pub instructions: Instructions,
}

#[derive(Debug)]
pub(crate) struct Modifiers
{
  pub limit: Option<Instructions>,
  pub skip: Option<Instructions>,
  pub order_by: Vec<OrderBy>,
}

#[derive(Debug)]
pub(crate) struct VariablesSizes
{
  /// How many temporary variables are used by the block.
  pub temporary_variables: usize,
  /// How many variables are persistent, after the block has been executed.
  pub persistent_variables: usize,
}

#[derive(Debug)]
pub(crate) enum Block
{
  Create
  {
    actions: Vec<CreateAction>,
    variables_size: VariablesSizes,
  },
  BlockMatch
  {
    blocks: Vec<BlockMatch>,
    filter: Instructions,
    optional: bool,
    variables_size: VariablesSizes,
  },
  Return
  {
    variables: Vec<RWExpression>,
    filter: Instructions,
    modifiers: Modifiers,
    variables_size: VariablesSizes,
  },
  Call
  {
    arguments: Instructions,
    name: String,
    variables_size: VariablesSizes,
  },
  With
  {
    variables: Vec<RWExpression>,
    filter: Instructions,
    modifiers: Modifiers,
    variables_size: VariablesSizes,
  },
  Unwind
  {
    name: String,
    instructions: Instructions,
    variables_size: VariablesSizes,
  },
  Delete
  {
    detach: bool,
    instructions: Vec<Instructions>,
    variables_size: VariablesSizes,
  },
  Update
  {
    updates: Vec<UpdateOne>,
    variables_size: VariablesSizes,
  },
}
