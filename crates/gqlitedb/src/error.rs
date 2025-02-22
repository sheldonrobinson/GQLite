//! Errors used for gqlite.

/// Represent compile time errors.
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
  #[error("NoSingleRelationshipType: an edge type need to be specified.")]
  NoSingleRelationshipType,
  #[error("NotComparable: values are not comparable.")]
  NotComparable,
  #[error("UnknownFunction: {name}")]
  UnknownFunction
  {
    name: String
  },
  #[error("InvalidAggregation: aggregation is not accepted in this expression.")]
  InvalidAggregation,
  #[error("ColumnNameConflict: Column '{name}' is duplicated.")]
  ColumnNameConflict
  {
    name: String
  },
  #[error("InvalidDelete: invalid delete argument, expected node or edge.")]
  InvalidDelete,
  #[error("NonConstantExpression: statement expect a constant expression.")]
  NonConstantExpression,
  #[error("InvalidArgumentType: invalid argument type.")]
  InvalidArgumentType,
}

/// Runtime errors.
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
  /// Too few or too many arguments
  #[error("InvalidNumberOfArguments: Invalid number of arguments for function '{function_name}' got {got} expected {expected}.")]
  InvalidNumberOfArguments
  {
    function_name: &'static str,
    got: usize,
    expected: usize,
  },
  /// Parameter is not known
  #[error(
    "ExpectedEdge: Function '{function_name}' expected argument {index} of type {expected_type} but got {value}."
  )]
  InvalidArgument
  {
    function_name: &'static str,
    index: usize,
    expected_type: &'static str,
    value: String,
  },
  #[error("NotComparable: values are not comparable.")]
  NotComparable,
  /// Edge has no label
  #[error("MissingEdgeLabel")]
  MissingEdgeLabel,
  #[error("UnknownFunction: {name}")]
  UnknownFunction
  {
    name: String
  },
  #[error("InvalidBinaryOperands: operands for binary operation are not compatible.")]
  InvalidBinaryOperands,
  #[error("InvalidNegationOperands: operands for negation operation are not compatible.")]
  InvalidNegationOperands,
  #[error("InvalidDelete: invalid delete argument, expected node or edge.")]
  InvalidDelete,
  #[error("DeleteConnectedNode: node is still connected and cannot be deleted.")]
  DeleteConnectedNode,
  #[error("NegativeIntegerArgument: statement expect a positive integer.")]
  NegativeIntegerArgument,
  #[error("InvalidArgumentType: invalid argument type.")]
  InvalidArgumentType,
}

/// Internal errors, should be treated as bugs.
#[derive(thiserror::Error, Debug)]
pub enum InternalError
{
  #[error("Aggregation state is missing.")]
  MissingAggregationState,
  #[error("Aggregation is missing an argument.")]
  MissingAggregationArgument,
  #[error("Aggregations are missing.")]
  MissingAggregations,
  #[error("Expected a value to be a node in {context}.")]
  ExpectedNode
  {
    context: &'static str
  },
  #[error("Expected a value to be an edge in {context}.")]
  ExpectedEdge
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
  #[error("Path pattern was used in create expression in {context}.")]
  PathPatternInCreateExpression
  {
    context: &'static str
  },
  #[error("Invalid create labels expression {context}.")]
  InvalidCreateLabels
  {
    context: &'static str
  },
  #[error("Expected graph value {context}.")]
  ExpectedGraphValue
  {
    context: &'static str
  },
  #[error("Expected node query {context}.")]
  ExpectedNodeQuery
  {
    context: &'static str
  },
  #[error("Expected edge query {context}.")]
  ExpectedEdgeQuery
  {
    context: &'static str
  },
  #[error("Empty stack.")]
  EmptyStack,
  #[error("Invalid value cast")]
  InvalidValueCast,
  #[error("Code is not reachable in {context}.")]
  Unreachable
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

  // Errors from dependencies
  #[cfg(feature = "redb")]
  #[error("ReDB: {0}")]
  ReDBError(#[from] redb::Error),
  #[cfg(feature = "redb")]
  #[error("ReDB:Storage: {0}")]
  ReDBStorageError(#[from] redb::StorageError),
  #[cfg(feature = "redb")]
  #[error("ReDB:DatabaseError: {0}")]
  ReDBDatabaseError(#[from] redb::DatabaseError),
  #[cfg(feature = "redb")]
  #[error("ReDB:TransactionError: {0}")]
  ReDBTransactionError(#[from] redb::TransactionError),
  #[cfg(feature = "redb")]
  #[error("ReDB:TableError: {0}")]
  ReDBTableError(#[from] redb::TableError),
  #[cfg(feature = "redb")]
  #[error("ReDB:CommitError: {0}")]
  ReDBCommitError(#[from] redb::CommitError),

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
  #[error("Unknown edge")]
  UnknownEdge,
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

#[allow(dead_code)]
pub(crate) fn show_backtrace<T>(t: T) -> T
{
  println!("{:#?}", std::backtrace::Backtrace::capture());
  t
}

impl From<pest::error::Error<crate::parser::Rule>> for Error
{
  fn from(value: pest::error::Error<crate::parser::Rule>) -> Self
  {
    CompileTimeError::from(value).into()
  }
}

//   ____                      _      _____
//  / ___| ___ _ __   ___ _ __(_) ___| ____|_ __ _ __ ___  _ __ ___
// | |  _ / _ \ '_ \ / _ \ '__| |/ __|  _| | '__| '__/ _ \| '__/ __|
// | |_| |  __/ | | |  __/ |  | | (__| |___| |  | | | (_) | |  \__ \
//  \____|\___|_| |_|\___|_|  |_|\___|_____|_|  |_|  \___/|_|  |___/

pub(crate) trait GenericErrors: Into<Error>
{
  fn unknown_function(name: impl Into<String>) -> Self;
  fn not_comparable() -> Self;
}

impl GenericErrors for CompileTimeError
{
  fn unknown_function(name: impl Into<String>) -> Self
  {
    Self::UnknownFunction { name: name.into() }
  }
  fn not_comparable() -> Self
  {
    Self::NotComparable
  }
}

impl GenericErrors for RunTimeError
{
  fn unknown_function(name: impl Into<String>) -> Self
  {
    Self::UnknownFunction { name: name.into() }
  }
  fn not_comparable() -> Self
  {
    Self::NotComparable
  }
}
