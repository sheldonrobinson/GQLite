use std::collections::HashMap;

pub(crate) enum Instruction {
  CreateNode { labels: Vec<String> },
  CreateEdge { label: String },
  Push { value: crate::graph::Value },
  GetVariable { name: String },
  CreateMap { keys: Vec<String> },
}

pub(crate) type Instructions = Vec<Instruction>;

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
