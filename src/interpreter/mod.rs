pub(crate) mod compiler;
mod context;
pub(crate) mod evaluators;
mod instructions;

type Program = Vec<instructions::Block>;
