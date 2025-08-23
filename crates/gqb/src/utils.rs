//! Utilities for graph query building.

/// Make a string into a valid label.
pub trait ToGraphLabel
{
  /// Make a string into a valid label.
  fn to_graph_label(&self) -> String;
}

fn to_graph_label_impl(string: &str) -> String
{
  string
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '_'
      {
        c
      }
      else
      {
        '_'
      }
    })
    .collect()
}

impl<T> ToGraphLabel for T
where
  T: AsRef<str>,
{
  fn to_graph_label(&self) -> String
  {
    to_graph_label_impl(self.as_ref())
  }
}

#[cfg(test)]
mod test
{
  use super::ToGraphLabel as _;
  #[test]
  fn test_to_graph_label()
  {
    assert_eq!("?%899asadA".to_graph_label(), "__899asadA".to_string());
    assert_eq!(
      "?%899asadA".to_string().to_graph_label(),
      "__899asadA".to_string()
    );
  }
}
