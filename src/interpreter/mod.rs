pub(crate) mod compiler;
pub(crate) mod evaluators;
mod instructions;
pub(crate) mod validator;

type Program = Vec<instructions::Block>;
