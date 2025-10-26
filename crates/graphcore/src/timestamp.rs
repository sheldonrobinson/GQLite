use std::fmt;

use serde::{Deserialize, Serialize};
use time::{
  convert::{Nanosecond, Second},
  format_description::well_known::Rfc3339,
  OffsetDateTime, UtcOffset,
};

/// Represent a timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeStamp(time::OffsetDateTime);

impl fmt::Display for TimeStamp
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    // RFC3339 helper returns a String, so we forward it
    self.0.format(&Rfc3339).map_err(|_| fmt::Error)?.fmt(f)
  }
}

impl TimeStamp
{
  /// Create a new time stamp with the given unix timestamp and  offset
  pub fn from_unix_timestamp(
    seconds: i64,
    nanoseconds: u32,
    offset_whole_seconds: i32,
  ) -> Result<TimeStamp, crate::Error>
  {
    let unix_timestamp =
      (seconds as i128 * Nanosecond::per_t::<i128>(Second)) + nanoseconds as i128;
    let local = OffsetDateTime::from_unix_timestamp_nanos(unix_timestamp)?;
    Ok(TimeStamp(OffsetDateTime::new_in_offset(
      local.date(),
      local.time(),
      UtcOffset::from_whole_seconds(offset_whole_seconds)?,
    )))
  }

  /// Convert from the current time zone to Utc (aka offset == 0)
  pub fn to_utc(self) -> TimeStamp
  {
    Self(self.0.to_utc().into())
  }
  /// Return the unix timestamp
  pub fn unix_timestamp(self) -> (i64, u32)
  {
    (self.0.unix_timestamp(), self.0.nanosecond())
  }
  /// Return the number of whole seconds in the offset
  pub fn offset_whole_seconds(self) -> i32
  {
    self.0.offset().whole_seconds()
  }
  /// Return the year in the timestamp timezone
  pub fn year(self) -> i32
  {
    self.0.year()
  }
  /// Return the month in the timestamp timezone
  pub fn month(self) -> u8
  {
    self.0.month().into()
  }
  /// Return the day in the timestamp timezone
  pub fn day(self) -> u8
  {
    self.0.day()
  }
  /// Return the hour in the timestamp timezone
  pub fn hour(self) -> u8
  {
    self.0.hour()
  }
  /// Return the minute in the timestamp timezone
  pub fn minute(self) -> u8
  {
    self.0.minute()
  }
  /// Return the second in the timestamp timezone
  pub fn second(self) -> u8
  {
    self.0.second()
  }
  /// Return the microseconds in the timestamp timezone
  pub fn microsecond(self) -> u32
  {
    self.0.microsecond()
  }
}

// Serde impl

impl Serialize for TimeStamp
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.0.format(&Rfc3339).unwrap().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for TimeStamp
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let dt = OffsetDateTime::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)?;
    Ok(TimeStamp(dt))
  }
}
