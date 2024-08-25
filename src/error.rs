use pest::error;

/// Represent compile time errors
#[derive(thiserror::Error, Debug)]
pub enum CompileTimeError
{
  /// This error happens
  #[error("VariableAlreadyBound: Variable {name} is already bound.")]
  VariableAlreadyBound
  {
    name: String
  },
}

#[derive(thiserror::Error, Debug)]
pub enum RunTimeError {}

#[derive(thiserror::Error, Debug)]
pub enum InternalError
{
  #[error("Expected a value to be a node in {0}")]
  ExpectedNode(&'static str),
}

/// GQLite errors
#[derive(thiserror::Error, Debug)]
pub enum Error
{
  /// Error that occurs during compilation
  #[error("CompileTime: {0}")]
  CompileTime(#[from] CompileTimeError),
  /// Error that occurs during runtime
  #[error("RunTime: {0}")]
  RunTime(#[from] RunTimeError),
  /// Error that should not occurs and most likely correspond to a bug
  #[error("Internal: {0}")]
  Internal(#[from] InternalError),

  // Following errors need reviews, and most would fall in the internal error category
  #[error("An error occured while serialization to Cbor: {0}")]
  CborSerialisationError(#[from] ciborium::ser::Error<std::io::Error>),
  #[error("An error occured while deserialization from Cbor: {0}")]
  CborDeserialisationError(#[from] ciborium::de::Error<std::io::Error>),
  #[error("Parse error: {0}")]
  ParseError(#[from] pest::error::Error<crate::parser::Rule>),
  #[error("Parse int error: {0}")]
  ParseFloatError(#[from] std::num::ParseFloatError),
  #[error("Parse int error: {0}")]
  ParseIntError(#[from] std::num::ParseIntError),
  #[error("Store error: {0}")]
  StoreError(String),
  #[error("Unexpected expression from the parser: {1} in {0}")]
  UnxpectedExpression(&'static str, String),
  #[error("Unknown node")]
  UnknownNode,
  #[error("Unknown variable {0}")]
  UnknownVariable(String),
  #[error("Empty stack {0}")]
  EmptyStack(String),
  #[error("Unknown error at {0}")]
  Unknown(&'static str),
  #[error("Internal error at {0}")]
  InternalError(&'static str),
  #[error("Unimplemented error at {0}")]
  Unimplemented(&'static str),
}
