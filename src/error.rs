#[derive(thiserror::Error, Debug)]
pub enum Error
{
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
