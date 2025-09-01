//! Errors used for gqlite.
use crate::prelude::*;

/// Represent compile time errors.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
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
  ParseError(#[from] Box<pest::error::Error<crate::parser::parser_impl::Rule>>),
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
  #[error("UnknownFunction: {name}.")]
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
#[allow(missing_docs)]
#[non_exhaustive]
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
  #[error("UnknownFunction: {name}.")]
  UnknownFunction
  {
    name: String
  },
  #[error("InvalidBinaryOperands: operands for binary operation are not compatible.")]
  InvalidBinaryOperands,
  #[error("InvalidNegationOperands: operands for negation operation are not compatible.")]
  InvalidNegationOperands,
  #[error("Invalid value cast, cannot cast {value} to {typename}.")]
  InvalidValueCast
  {
    value: Box<crate::Value>,
    typename: &'static str,
  },
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
  #[error("DuplicatedGraph: {graph_name} already exists.")]
  DuplicatedGraph
  {
    graph_name: String
  },
  #[error("UnknownGraph: {graph_name} does not exists.")]
  UnknownGraph
  {
    graph_name: String
  },
  #[error("Key {key} cannot be found in a path in a ValueMap.")]
  MissingKeyInPath
  {
    key: String
  },
  #[error("Path cannot have null key.")]
  MissingKey,
}

/// Internal errors, should be treated as bugs.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
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
  GenericStdError(#[from] Box<dyn std::error::Error + Sync + Send>),
  #[error("Not a write transaction")]
  NotWriteTransaction,
  #[error("Missing metadata {key}")]
  MissingMetadata
  {
    key: String
  },
  #[error("Invalid aggregation state")]
  InvalidAggregationState,

  #[error("Invalid query result cast")]
  InvalidQueryResultCast,

  // Third-party
  #[error("Missing element in iterator.")]
  MissingElementIterator,

  // Errors from askama
  #[cfg(feature = "sqlite")]
  #[error("Askama: {0}")]
  AskamaError(#[from] askama::Error),

  // Serialization errors
  #[error("An error occured while serialization to Cbor: {0}")]
  CborSerialisationError(#[from] ciborium::ser::Error<std::io::Error>),
  #[error("An error occured while deserialization from Cbor: {0}")]
  CborDeserialisationError(#[from] ciborium::de::Error<std::io::Error>),
  #[error("JSon error: {0}")]
  JsonError(#[from] serde_json::Error),

  // Parser related error
  #[error("Missing function name")]
  MissingFunctionName,
  #[error("Unexpected expression from the parser: {1} in {0}")]
  UnxpectedExpression(&'static str, String),

  // Technical debt: following errors need to be reviewed
  #[error("Parse int error: {0}")]
  ParseFloatError(#[from] std::num::ParseFloatError),
  #[error("Parse int error: {0}")]
  ParseIntError(#[from] std::num::ParseIntError),
  #[error("Unknown node")]
  UnknownNode,
  #[error("Unknown edge")]
  UnknownEdge,
  #[error("Unimplemented error at {0}")]
  Unimplemented(&'static str),
  #[error("Infallible.")]
  Infallible(#[from] std::convert::Infallible),
  #[error("Poison error {0}.")]
  Poison(String),
  #[error("IOError: {0}.")]
  IOError(#[from] std::io::Error),
}

/// Error in the store backend.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum StoreError
{
  // Errors from sqlite
  #[cfg(feature = "sqlite")]
  #[error("Sqlite: {0}")]
  SqliteError(#[from] rusqlite::Error),

  #[cfg(feature = "redb")]
  #[error("redb: {0}")]
  RedbError(#[from] redb::Error),

  #[cfg(feature = "redb")]
  #[error("redb: {0}")]
  Redb2Error(#[from] redb2::Error),

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
  #[error("OpeningError: could not open database, got the following error messages from the backends: {errors}.")]
  OpeningError
  {
    errors: String
  },
  #[error("DuplicatedGraph: attempt at creating an existing graph: {graph_name}.")]
  DuplicatedGraph
  {
    graph_name: String
  },
  #[error("UnknownGraph: {graph_name} is not known.")]
  UnknownGraph
  {
    graph_name: String
  },
  #[error(
    "IncompatibleVersion: could not open database, got version {actual} but expected {expected}."
  )]
  IncompatibleVersion
  {
    actual: utils::Version,
    expected: utils::Version,
  },
  #[error("Invalid database format: {0}.")]
  InvalidFormat(String),
}

/// GQLite errors
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error
{
  /// Error that occurs during compilation
  #[error("CompileTime: {0}")]
  CompileTime(#[from] CompileTimeError),
  /// Error that occurs during runtime
  #[error("RunTime: {0}")]
  RunTime(#[from] RunTimeError),
  /// Store error
  #[error("StoreError: {0}")]
  StoreError(#[from] StoreError),
  /// Error that should not occurs and most likely correspond to a bug
  #[error("Internal: {0}")]
  Internal(#[from] InternalError),
}

ccutils::assert_impl_all!(Error: Send, Sync);

impl Error
{
  /// Return the underlying error (match WithBacktrace API)
  pub fn error(&self) -> &Error
  {
    self
  }
  #[cfg(not(feature = "_backtrace"))]
  pub(crate) fn split_error(self) -> (Error, ())
  {
    (self, ())
  }
  #[cfg(not(feature = "_backtrace"))]
  pub(crate) fn make_error(error: Error, _: ()) -> Error
  {
    error
  }
}

impl From<graphcore::Error> for Error
{
  fn from(value: graphcore::Error) -> Self
  {
    match value
    {
      graphcore::Error::InvalidBinaryOperands => RunTimeError::InvalidBinaryOperands.into(),
      graphcore::Error::InvalidNegationOperands => RunTimeError::InvalidNegationOperands.into(),
      graphcore::Error::InvalidValueCast { value, typename } =>
      {
        RunTimeError::InvalidValueCast { value, typename }.into()
      }
      graphcore::Error::MissingKey => RunTimeError::MissingKey.into(),
      graphcore::Error::MissingKeyInPath { key } => RunTimeError::MissingKeyInPath { key }.into(),
      _ => InternalError::Unimplemented("From graphcore::Error to graphcore::Error.").into(),
    }
  }
}

//  _____                   __        ___ _   _     ____             _    _
// | ____|_ __ _ __ ___  _ _\ \      / (_) |_| |__ | __ )  __ _  ___| | __ |_ _ __ __ _  ___ ___
// |  _| | '__| '__/ _ \| '__\ \ /\ / /| | __| '_ \|  _ \ / _` |/ __| |/ / __| '__/ _` |/ __/ _ \
// | |___| |  | | | (_) | |   \ V  V / | | |_| | | | |_) | (_| | (__|   <| |_| | | (_| | (_|  __/
// |_____|_|  |_|  \___/|_|    \_/\_/  |_|\__|_| |_|____/ \__,_|\___|_|\_\\__|_|  \__,_|\___\___|

#[derive(Debug)]
#[cfg(feature = "_backtrace")]
pub struct ErrorWithBacktrace
{
  error: Error,
  backtrace: std::backtrace::Backtrace,
}

#[cfg(feature = "_backtrace")]
impl ErrorWithBacktrace
{
  /// Return the underlying error
  pub fn error(&self) -> &Error
  {
    &self.error
  }
  pub(crate) fn split_error(self) -> (Error, std::backtrace::Backtrace)
  {
    (self.error, self.backtrace)
  }
  pub(crate) fn make_error(error: Error, backtrace: std::backtrace::Backtrace) -> Self
  {
    Self { error, backtrace }
  }
}

#[cfg(feature = "_backtrace")]
impl std::fmt::Display for ErrorWithBacktrace
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    self.error.fmt(f)
  }
}

#[cfg(feature = "_backtrace")]
impl<T> From<T> for ErrorWithBacktrace
where
  T: Into<Error>,
{
  fn from(value: T) -> Self
  {
    Self {
      error: value.into(),
      backtrace: std::backtrace::Backtrace::capture(),
    }
  }
}

//   ____                              _
//  / ___|___  _ ____   _____ _ __ ___(_) ___  _ __
// | |   / _ \| '_ \ \ / / _ \ '__/ __| |/ _ \| '_ \
// | |__| (_) | | | \ V /  __/ |  \__ \ | (_) | | | |
//  \____\___/|_| |_|\_/ \___|_|  |___/_|\___/|_| |_|

impl<T> From<std::sync::PoisonError<T>> for Error
{
  fn from(value: std::sync::PoisonError<T>) -> Self
  {
    InternalError::Poison(format!("{:?}", value)).into()
  }
}

impl From<pest::error::Error<crate::parser::parser_impl::Rule>> for Error
{
  fn from(value: pest::error::Error<crate::parser::parser_impl::Rule>) -> Self
  {
    CompileTimeError::from(Box::new(value)).into()
  }
}

macro_rules! error_as_internal {
  ($err_type:ty) => {
    impl From<$err_type> for crate::prelude::ErrorType
    {
      fn from(value: $err_type) -> Self
      {
        let err: crate::error::InternalError = value.into();
        err.into()
      }
    }
  };
}

pub(crate) use error_as_internal;

macro_rules! error_as_store {
  ($err_type:ty) => {
    impl From<$err_type> for crate::prelude::ErrorType
    {
      fn from(value: $err_type) -> Self
      {
        let err: crate::error::StoreError = value.into();
        err.into()
      }
    }
  };
}

pub(crate) use error_as_store;

error_as_internal! {ciborium::ser::Error<std::io::Error>}
error_as_internal! {ciborium::de::Error<std::io::Error>}
error_as_internal! {serde_json::Error}
error_as_internal! {std::num::ParseFloatError}

#[cfg(feature = "redb")]
mod _trait_impl_redb
{
  super::error_as_store! {redb::Error}
  macro_rules! redb_error_as_store {
    ($err_type:ty) => {
      impl From<$err_type> for crate::prelude::ErrorType
      {
        fn from(value: $err_type) -> Self
        {
          let redb_err: redb::Error = value.into();
          let err: crate::error::StoreError = redb_err.into();
          err.into()
        }
      }
    };
  }
  super::error_as_store! {redb2::Error}
  macro_rules! redb2_error_as_store {
    ($err_type:ty) => {
      impl From<$err_type> for crate::prelude::ErrorType
      {
        fn from(value: $err_type) -> Self
        {
          let redb_err: redb2::Error = value.into();
          let err: crate::error::StoreError = redb_err.into();
          err.into()
        }
      }
    };
  }
  redb_error_as_store! {redb::StorageError}
  redb_error_as_store! {redb::DatabaseError}
  redb_error_as_store! {redb::TransactionError}
  redb_error_as_store! {redb::TableError}
  redb_error_as_store! {redb::CommitError}
  redb2_error_as_store! {redb2::DatabaseError}
  redb2_error_as_store! {redb2::UpgradeError}
}
#[cfg(feature = "sqlite")]
mod _trait_impl_sqlite
{
  error_as_store! {rusqlite::Error}
  error_as_internal! {askama::Error}
}

/// Merge a list of error into a string error message
pub(crate) fn vec_to_error(errs: &[ErrorType]) -> String
{
  let errs: Vec<String> = errs.iter().map(|x| format!("'{}'", x)).collect();
  errs.join(", ")
}

pub(crate) fn parse_int_error_to_compile_error(
  text: &str,
  e: std::num::ParseIntError,
) -> crate::prelude::ErrorType
{
  use std::num::IntErrorKind;
  match e.kind()
  {
    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => CompileTimeError::IntegerOverflow {
      text: text.to_owned(),
    }
    .into(),
    _ => InternalError::ParseIntError(e).into(),
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
    let error: crate::prelude::ErrorType = $err;
    let (error, meta) = error.split_error();
    match error
    {
      $source => ErrorType::make_error($destination.into(), meta),
      o => ErrorType::make_error(o, meta),
    }
  }};
}

pub(crate) use map_error;

use crate::prelude::ErrorType;

impl From<std::convert::Infallible> for Error
{
  fn from(value: std::convert::Infallible) -> Self
  {
    InternalError::Infallible(value).into()
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
}

impl GenericErrors for CompileTimeError
{
  fn unknown_function(name: impl Into<String>) -> Self
  {
    Self::UnknownFunction { name: name.into() }
  }
}

impl GenericErrors for RunTimeError
{
  fn unknown_function(name: impl Into<String>) -> Self
  {
    Self::UnknownFunction { name: name.into() }
  }
}

/// GQLite Result
#[cfg(not(feature = "_backtrace"))]
pub(crate) mod export
{
  pub type Error = super::Error;
}

/// GQLite Result
#[cfg(feature = "_backtrace")]
pub(crate) mod export
{
  pub type Error = super::ErrorWithBacktrace;
}
