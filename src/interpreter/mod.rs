pub(crate) mod compiler;
pub(crate) mod executer;
mod instructions;

type Program = Vec<instructions::Instruction>;
