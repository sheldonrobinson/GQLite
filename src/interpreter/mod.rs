pub(crate) mod compiler;
pub(crate) mod evaluators;
pub(crate) mod expression_analyser;
mod instructions;
pub(crate) mod validator;

type Program = Vec<instructions::Block>;
