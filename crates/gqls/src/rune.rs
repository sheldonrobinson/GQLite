//! Rune support module
pub use rune::{support::Result, Any, Context, Hash, Ref, TypeHash as _, Value};

use crate::ElementType;

/// Metadata associated with an element, used by the macro generator.
pub struct ElementMetadata
{
  /// Type
  pub element_type: ElementType,
  /// List of labels
  pub labels: &'static [&'static str],
}

/// Use to cast Rune arguments to Rust arguments
pub trait IntoRustArgument
{
  /// Rust type corresponding to the rune argument
  type RustType;
  /// Convert to rust
  fn into_rust_argument(self) -> Self::RustType;
}

impl IntoRustArgument for Ref<str>
{
  type RustType = String;
  fn into_rust_argument(self) -> Self::RustType
  {
    self.to_string()
  }
}

impl IntoRustArgument for Ref<gqliterune::TimeStamp>
{
  type RustType = graphcore::TimeStamp;
  fn into_rust_argument(self) -> Self::RustType
  {
    self.clone().into()
  }
}

impl IntoRustArgument for f64
{
  type RustType = f64;
  fn into_rust_argument(self) -> Self::RustType
  {
    self
  }
}

impl<T: IntoRustArgument> IntoRustArgument for Option<T>
{
  type RustType = Option<T::RustType>;
  fn into_rust_argument(self) -> Self::RustType
  {
    self.map(|x| x.into_rust_argument())
  }
}
