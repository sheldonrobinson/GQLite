use rune::Any;

/// Workaround lack of runtime support for u128: https://github.com/rune-rs/rune/issues/960
#[derive(Any)]
#[rune(item = ::gqlite)]
pub struct Key
{
  key: graphcore::Key,
}

impl Key
{
  /// Create a new key
  pub fn new(key: graphcore::Key) -> Self
  {
    Self { key }
  }
  /// Access the underlying graphcore key
  pub fn key(&self) -> graphcore::Key
  {
    self.key
  }
  /// Implementation of comparison equal
  #[rune::function(protocol = PARTIAL_EQ)]
  pub fn partial_eq(&self, other: &Self) -> bool
  {
    self.key == other.key
  }
}

impl From<graphcore::Key> for Key
{
  fn from(value: graphcore::Key) -> Self
  {
    Self { key: value }
  }
}
