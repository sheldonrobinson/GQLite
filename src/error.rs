#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("An error occured while serialization to Cbor: {0}")]
  CborSerialisationError(#[from] ciborium::ser::Error<std::io::Error>),
  #[error("An error occured while deserialization from Cbor: {0}")]
  CborDeserialisationError(#[from] ciborium::de::Error<std::io::Error>),
  // #[error("the data for key `{0}` is not available")]
  // Redaction(String),
  // #[error("invalid header (expected {expected:?}, found {found:?})")] InvalidHeader {
    // expected: String,
    // found: String,
  // },
  #[error("Parse error: {0}")]
  ParseError(#[from] pest::error::Error<crate::parser::Rule>),
  #[error("Store error: {0}")]
  StoreError(String),
  #[error("Unexpected expression from the parser: {1} in {0}")]
  UnxpectedExpression(&'static str, String),
  #[error("Unknown node")]
  UnknownNode,
  #[error("Unknown variable {0}")]
  UnknownVariable(String),
  #[error("Unknown error at {0}")]
  Unknown(&'static str),
  #[error("Unimplemented error at {0}")]
  Unimplemented(&'static str),
}
