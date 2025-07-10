use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Version
{
  pub major: u16,
  pub minor: u16,
  #[serde(alias = "revision", alias = "release")]
  pub patch: u16,
}

impl Debug for Version
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    <Self as Display>::fmt(self, f)
  }
}

impl Display for Version
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
  }
}
