use crate::{prelude::*, value_table::ColId};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
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
    value: value::Value,
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
  SubtractionBinaryOperator,
  MultiplicationBinaryOperator,
  DivisionBinaryOperator,
  ModuloBinaryOperator,
  ExponentBinaryOperator,
}

pub(crate) type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub(crate) struct CreateAction
{
  pub(crate) instructions: Instructions,
  pub(crate) variables: Vec<Option<ColId>>,
}

#[derive(Debug)]
pub(crate) enum BlockMatch
{
  MatchNode
  {
    instructions: Instructions,
    variable: Option<ColId>,
    filter: Instructions,
  },
  MatchEdge
  {
    instructions: Instructions,
    left_variable: Option<ColId>,
    edge_variable: Option<ColId>,
    right_variable: Option<ColId>,
    path_variable: Option<ColId>,
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

/// R(eturn)W(ith)Expression are expressions computed in Return or With blocks.
#[derive(Debug)]
pub(crate) struct RWExpression
{
  /// Id of the column where to get the value from
  pub(crate) col_id: ColId,
  /// Instructions to compute this expression
  pub(crate) instructions: Instructions,
  /// Potential aggregations used by this expression
  pub(crate) aggregations: Vec<(value_table::ColId, RWAggregation)>,
}

/// Update one node/edge.
#[derive(Debug)]
pub(crate) enum UpdateOne
{
  SetProperty
  {
    /// Target is the column containing the node/edge to modify
    target: ColId,
    path: Vec<String>,
    instructions: Instructions,
  },
  AddProperty
  {
    /// Target is the column containing the node/edge to modify
    target: ColId,
    path: Vec<String>,
    instructions: Instructions,
  },
  RemoveProperty
  {
    /// Target is the column containing the node/edge to modify
    target: ColId,
    path: Vec<String>,
  },
  AddLabels
  {
    /// Target is the column containing the node/edge to modify
    target: ColId,
    labels: Vec<String>,
  },
  RemoveLabels
  {
    /// Target is the column containing the node/edge to modify
    target: ColId,
    labels: Vec<String>,
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
  /// How many variables are persistent, after the block has been executed.
  pub persistent_variables: usize,
  /// How many temporary variables are used by the block.
  pub temporary_variables: usize,
}

impl VariablesSizes
{
  /// Return the total size, `persistent+temporary` variables
  pub(crate) fn total_size(&self) -> usize
  {
    self.temporary_variables + self.persistent_variables
  }
}

#[derive(Debug)]
pub(crate) enum Block
{
  CreateGraph
  {
    name: String
  },
  DropGraph
  {
    name: String, if_exists: bool
  },
  UseGraph
  {
    name: String
  },
  Create
  {
    actions: Vec<CreateAction>,
    variables_size: VariablesSizes,
  },
  Match
  {
    blocks: Vec<BlockMatch>,
    filter: Instructions,
    optional: bool,
    variables_size: VariablesSizes,
  },
  Return
  {
    variables: Vec<(String, RWExpression)>,
    filter: Instructions,
    modifiers: Modifiers,
    variables_size: VariablesSizes,
  },
  Call
  {
    #[allow(dead_code)]
    arguments: Instructions,
    name: String,
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
    col_id: ColId,
    instructions: Instructions,
    variables_size: VariablesSizes,
  },
  Delete
  {
    detach: bool,
    instructions: Vec<Instructions>,
  },
  Update
  {
    updates: Vec<UpdateOne>,
    variables_size: VariablesSizes,
  },
}
