use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum Instruction {
  CreateNode { labels: Vec<String> },
  CreateEdge { label: Option<String> },
  Push { value: crate::graph::Value },
  GetVariable { name: String },
  CreateMap { keys: Vec<String> },
  // Duplicate,
}

pub(crate) type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub(crate) enum Block {
  Create {
    instructions: Instructions,
    variables: Vec<Option<String>>,
  },
  Match {
    instructions: Instructions,
    variables: Vec<Option<String>>,
  },
  Return {
    variables: HashMap<String, Instructions>,
  },
}
