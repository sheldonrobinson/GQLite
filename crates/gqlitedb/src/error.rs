//! Errors used for gqlite.

/// Represent compile time errors.
#[derive(thiserror::Error, Debug)]
pub enum CompileTimeError
{
  /// Floating point overflow
  #[error("FloatingPointOverflow: '{text}' is too large.")]
  FloatingPointOverflow
  {
    text: String
  },
  /// Integer overflow
  #[error("IntegerOverflow: '{text}' is too large.")]
  IntegerOverflow
  {
    text: String
  },
  /// Parse error
  #[error("ParseError: '{0}'")]
  ParseError(#[from] pest::error::Error<crate::parser::parser::Rule>),
  /// Variable is not defined
  #[error("UndefinedVariable: Unknown variable '{name}'.")]
  UndefinedVariable
  {
    name: String
  },
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
  /// Too few or too many arguments
  #[error("InvalidNumberOfArguments: Invalid number of arguments for function '{function_name}' got {got} expected {expected}.")]
  InvalidNumberOfArguments
  {
    function_name: &'static str,
    got: usize,
    expected: usize,
  },
  #[error("NonConstantExpression: statement expect a constant expression.")]
  NonConstantExpression,
  #[error("InvalidArgumentType: invalid argument type.")]
  InvalidArgumentType,
}

/// Runtime errors.
#[derive(thiserror::Error, Debug)]
pub enum RunTimeError
{
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
    "InvalidArgument: Function '{function_name}' expected argument {index} of type {expected_type} but got {value}."
  )]
  InvalidArgument
  {
    function_name: &'static str,
    index: usize,
    expected_type: &'static str,
    value: String,
  },
  #[error(
    "MapElementAccessByNonString: attempt to accessing a map value using a non-string value."
  )]
  MapElementAccessByNonString,
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
  #[error("OutOfBound: index is out of bound for array.")]
  OutOfBound,
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
  #[error("Invalid value cast, cannot cast {value} to {typename}.")]
  InvalidValueCast
  {
    value: crate::Value,
    typename: &'static str,
  },
  #[error("Code is not reachable in {context}.")]
  Unreachable
  {
    context: &'static str
  },
  #[error("Invalid number of columns in row {actual} but expected {expected}.")]
  InvalidNumberColumns
  {
    actual: usize, expected: usize
  },
  #[error("Unknown variable '{name}'.")]
  UnknownVariable
  {
    name: String
  },
  #[error("Invalid index {index} access of a vector of length {length}.")]
  InvalidIndex
  {
    index: usize, length: usize
  },
  #[error("Invalid row length got {got} expected {expected}.")]
  InvalidRowLength
  {
    got: usize, expected: usize
  },
  #[error("Some variables were declared, but not set. Set variables are {set_variables:?}, all variables are {all_variables:?}")]
  NotAllVariablesAreSet
  {
    set_variables: Vec<String>,
    all_variables: Vec<String>,
  },
  #[error("A generic error occured {0}.")]
  GenericStdError(#[from] Box<dyn std::error::Error>),
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError
{
  #[error("UnknownBackend: backend '{backend}' is unknown.")]
  UnknownBackend
  {
    backend: String
  },
  #[error("UnavailableBackend: backend '{backend}' is unavailable, and was not built.")]
  UnavailableBackend
  {
    backend: &'static str
  },
  #[error("OpeningError: could not open database, got the following error messages from the backends: {errors}")]
  OpeningError
  {
    errors: String
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
  /// Connection error
  #[error("ConnectionError")]
  ConnectionError(#[from] ConnectionError),
  /// Error that should not occurs and most likely correspond to a bug
  #[error("Internal: {0}")]
  Internal(#[from] InternalError),

  // Errors from redb
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
  #[error("Empty stack {0}")]
  EmptyStack(String),
  #[error("Unknown error at {0}")]
  Unknown(&'static str),
  #[error("Internal error at {0}")]
  InternalError(&'static str),
  #[error("Unimplemented error at {0}")]
  Unimplemented(&'static str),
  #[error("Infallible.")]
  Infallible(#[from] Infallible),
}

#[allow(dead_code)]
pub(crate) fn show_backtrace<T>(t: T) -> T
{
  println!("{:#?}", std::backtrace::Backtrace::capture());
  t
}

impl From<pest::error::Error<crate::parser::parser::Rule>> for Error
{
  fn from(value: pest::error::Error<crate::parser::parser::Rule>) -> Self
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

/// Merge a list of error into a string error message
pub(crate) fn vec_to_error<E: std::fmt::Display>(errs: &Vec<Error>) -> String
{
  let errs: Vec<String> = errs.iter().map(|x| format!("'{}'", x)).collect();
  errs.join(", ")
}

pub(crate) fn parse_int_error_to_compile_error<'a>(
  text: &'a str,
  e: std::num::ParseIntError,
) -> crate::Error
{
  use std::num::IntErrorKind;
  match e.kind()
  {
    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => CompileTimeError::IntegerOverflow {
      text: text.to_owned(),
    }
    .into(),
    _ => e.into(),
  }
}

/// Convenient macro for mapping errors, for instance, from internal error to runtime error:
///
/// ```notest
///   v.try_into()
///     .map_err(|e| error::map_error!(e, Error::Internal(InternalError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))?;
/// ```
macro_rules! map_error {
  ($err:expr, $source:pat => $destination:expr) => {{
    use crate::error::*;
    match $err
    {
      $source => $destination.into(),
      o => o,
    }
  }};
}

use std::convert::Infallible;

pub(crate) use map_error;
