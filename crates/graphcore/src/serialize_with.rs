use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

const NAN: f64 = f64::NAN;

pub fn serialize_float<S>(x: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  if x.is_nan()
  {
    serializer.serialize_str("NaN")
  }
  else
  {
    serializer.serialize_f64(*x)
  }
}

struct FloatDeserializeVisitor;

impl<'de> Visitor<'de> for FloatDeserializeVisitor
{
  type Value = f64;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
  {
    formatter.write_str("a float or the string \"NaN\"")
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v as f64)
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v as f64)
  }

  fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v as f64)
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v as f64)
  }

  fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v as f64)
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v)
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    if v == "NaN"
    {
      Ok(NAN)
    }
    else
    {
      Err(E::invalid_value(de::Unexpected::Str(v), &self))
    }
  }
}

pub fn deserialize_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(FloatDeserializeVisitor)
}
