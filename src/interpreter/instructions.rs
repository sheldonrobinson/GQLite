pub(crate) enum Instruction
{
  Push{ value: crate::graph::Value },
  Create{ variables: Vec<Option<String>> },
}
