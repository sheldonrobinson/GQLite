/// Represent errors.
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Error
{
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
  #[error("Key {key} cannot be found in a path in a ValueMap.")]
  MissingKeyInPath
  {
    key: String
  },
  #[error("Path cannot have null key.")]
  MissingKey,
  #[error("Invalid table dimensions.")]
  InvalidTableDimensions,
  #[error("Out of range access.")]
  InvalidRange,
}
