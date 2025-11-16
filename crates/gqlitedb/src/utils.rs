use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

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

pub(crate) fn hex(key: impl Into<graph::Key>) -> String
{
  format!("{:032X}", key.into().uuid())
}

#[cfg(test)]
mod tests
{
  use crate::prelude::*;
  #[test]
  fn test_hex()
  {
    assert_eq!(
      super::hex(graph::Key::new(18580062510968287067562660977870108180)),
      "0DFA63CEE7484B0DBFC407697F77F614"
    );
    assert_eq!(
      super::hex(graph::Key::new(0)),
      "00000000000000000000000000000000"
    );
  }
}
