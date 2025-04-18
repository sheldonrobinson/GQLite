pub(crate) mod compiler;
pub(crate) mod evaluators;
mod instructions;

type Program = Vec<instructions::Block>;
