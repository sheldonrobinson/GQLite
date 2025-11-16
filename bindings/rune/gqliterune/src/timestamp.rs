use rune::{support::Result, Any, Ref};

ccutils::alias!(
  #[derive(Any)]
  #[rune(item = ::gqlite)]
  #[doc = "Timestamp"]
  pub TimeStamp,
  graphcore::TimeStamp);

impl TimeStamp
{
  /// Parse a string into a timestamp
  #[rune::function(path = Self::parse)]
  pub fn parse(date: Ref<str>) -> Result<TimeStamp>
  {
    Ok(TimeStamp(graphcore::TimeStamp::parse(date.as_ref())?))
  }
  /// Convert timestamp to string
  #[rune::function]
  pub fn to_string(&self) -> String
  {
    self.0.to_string()
  }
}

impl From<TimeStamp> for graphcore::Value
{
  fn from(value: TimeStamp) -> Self
  {
    value.0.into()
  }
}
