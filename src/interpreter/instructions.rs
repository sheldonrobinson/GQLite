use std::collections::HashMap;

pub(crate) enum Instruction
{
  Push { value: crate::graph::Value },
  GetVariable { name: String },
}

pub(crate) type Instructions = Vec<Instruction>;

pub(crate) enum Block
{
  Create { instructions: Instructions, variables: Vec<Option<String>> },
  Match { instructions: Instructions, variables: Vec<Option<String>> },
  Return { variables: HashMap<String, Instructions> }
}
