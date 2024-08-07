use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum Instruction
{
  CreateNodeLiteral
  {
    labels: Vec<String>,
  },
  CreateEdgeLiteral
  {
    label: Option<String>,
  },
  Push
  {
    value: crate::graph::Value,
  },
  GetVariable
  {
    name: String,
  },
  CreateMap
  {
    keys: Vec<String>,
  },
  Duplicate,
  Rot3,
}

pub(crate) type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub(crate) enum Block
{
  Create
  {
    instructions: Instructions,
    variables: Vec<Option<String>>,
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
  },
  Return
  {
    variables: HashMap<String, Instructions>,
  },
}
