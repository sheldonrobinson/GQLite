use std::fmt;

use jiff::{tz::TimeZone, Zoned};
use serde::{Deserialize, Serialize};

ccutils::alias!(#[doc = "Represent a timestamp"] pub TimeStamp, Zoned, display, derive: Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord);

impl TimeStamp
{
  /// Create a new time stamp with the given unix timestamp and  offset
  pub fn from_unix_timestamp(
    seconds: i64,
    nanoseconds: i32,
    offset_whole_seconds: i32,
  ) -> Result<TimeStamp, crate::Error>
  {
    let ts = jiff::Timestamp::new(seconds, nanoseconds)?;
    Ok(Self(ts.to_zoned(jiff::tz::TimeZone::fixed(
      jiff::tz::Offset::from_seconds(offset_whole_seconds)?,
    ))))
  }

  /// Convert from the current time zone to Utc (aka offset == 0)
  pub fn to_utc(&self) -> TimeStamp
  {
    Self(self.with_time_zone(TimeZone::UTC))
  }
  /// Return the unix timestamp
  pub fn unix_timestamp(&self) -> (i64, i32)
  {
    let ts = self.0.timestamp();
    (ts.as_second(), ts.subsec_nanosecond())
  }
  /// Return the number of whole seconds in the offset
  pub fn offset_seconds(&self) -> i32
  {
    self.0.offset().seconds()
  }
  /// Return the year in the timestamp timezone
  pub fn year(&self) -> i16
  {
    self.0.year()
  }
  /// Return the month in the timestamp timezone
  pub fn month(&self) -> i8
  {
    self.0.month()
  }
  /// Return the day in the timestamp timezone
  pub fn day(&self) -> i8
  {
    self.0.day()
  }
  /// Return the hour in the timestamp timezone
  pub fn hour(&self) -> i8
  {
    self.0.hour()
  }
  /// Return the minute in the timestamp timezone
  pub fn minute(&self) -> i8
  {
    self.0.minute()
  }
  /// Return the second in the timestamp timezone
  pub fn second(&self) -> i8
  {
    self.0.second()
  }
  /// Return the microseconds in the timestamp timezone
  pub fn microsecond(&self) -> i16
  {
    self.0.microsecond()
  }
  /// Parse the time string
  pub fn parse(date: &str) -> Result<TimeStamp, crate::Error>
  {
    Ok(TimeStamp(date.parse()?))
  }
}

// Serde impl

impl Serialize for TimeStamp
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.0.to_string().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for TimeStamp
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    use serde::de::Error;
    let s = String::deserialize(deserializer)?;
    TimeStamp::parse(&s)
      .map_err(|e| D::Error::custom(format!("Error deserializing timestamp: {}.", e)))
  }
}
