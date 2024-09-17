use std::collections::BTreeMap;

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
  Rot3,
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
  },
  MatchEdge
  {
    instructions: Instructions,
    left_variable: Option<String>,
    edge_variable: Option<String>,
    right_variable: Option<String>,
    path_variable: Option<String>,
  },
  Return
  {
    variables: BTreeMap<String, Instructions>,
  },
  Call
  {
    arguments: Instructions,
    name: String,
  },
  With
  {
    all: bool,
    variables: BTreeMap<String, Instructions>,
  },
  Unwind
  {
    name: String,
    instructions: Instructions,
  },
}
