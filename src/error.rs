/// Represent compile time errors
#[derive(thiserror::Error, Debug)]
pub enum CompileTimeError
{
  /// Parse error
  #[error("ParseError: '{0}'")]
  ParseError(#[from] pest::error::Error<crate::parser::Rule>),
  /// This error happens if the variable is already defined
  #[error("VariableAlreadyBound: Variable '{name}' is already bound.")]
  VariableAlreadyBound
  {
    name: String
  },
  /// This error happens if the variable is already bound to a different type
  #[error(
    "VariableTypeConflict: Variable '{name}' is redefined as a variable of a different type."
  )]
  VariableTypeConflict
  {
    name: String
  },
  /// Variable is not defined
  #[error("UndefinedVariable: Unknown variable '{name}'.")]
  UndefinedVariable
  {
    name: String
  },
  /// ()-[]-() is not accepted in this context
  #[error("RequiresDirectedRelationship: edges need to be directed in this context: '{context}'.")]
  RequiresDirectedRelationship
  {
    context: &'static str
  },
}

#[derive(thiserror::Error, Debug)]
pub enum RunTimeError
{
  /// Variable is not defined
  #[error("UndefinedVariable: Unknown variable '{name}'.")]
  UndefinedVariable
  {
    name: String
  },
  /// Parameter is not known
  #[error("UnknownParameter: Unknown parameter '{name}'.")]
  UnknownParameter
  {
    name: String
  },
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError
{
  #[error("Expected a value to be a node in {context}.")]
  ExpectedNode
  {
    context: &'static str
  },
  #[error("Missing a pair from pest parsing in {context}.")]
  MissingPair
  {
    context: &'static str
  },
  #[error("Unexpected pair {pair} from pest parsing in {context}.")]
  UnexpectedPair
  {
    context: &'static str, pair: String
  },
  #[error("Unknown variable {variable} in {context}.")]
  UnknownVariable
  {
    context: &'static str,
    variable: String,
  },
  #[error("Missing value from stack in {context}.")]
  MissingStackValue
  {
    context: &'static str
  },
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

impl From<pest::error::Error<crate::parser::Rule>> for Error
{
  fn from(value: pest::error::Error<crate::parser::Rule>) -> Self
  {
    CompileTimeError::from(value).into()
  }
}
