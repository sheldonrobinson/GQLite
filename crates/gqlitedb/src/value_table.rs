use std::collections::HashMap;

pub(crate) type Row = std::collections::HashMap<String, crate::graph::Value>;

#[derive(Debug)]
pub(crate) struct ValueTable
{
  data: Vec<Row>,
}

impl ValueTable
{
  pub(crate) fn new() -> Self
  {
    Self {
      data: Vec::<Row>::default(),
    }
  }
  pub(crate) fn add_row(&mut self, row: Row)
  {
    self.data.push(row);
  }
  pub(crate) fn add_rows(&mut self, rows: &mut Vec<Row>)
  {
    self.data.append(rows);
  }
  pub(crate) fn iter(&self) -> core::slice::Iter<'_, Row>
  {
    self.data.iter()
  }
  pub(crate) fn first_row(&self) -> Option<&Row>
  {
    self.data.first()
  }
  pub(crate) fn remove_first_rows(&mut self, n: usize)
  {
    if n < self.data.len()
    {
      self.data.drain(0..n);
    }
    else
    {
      self.data.clear();
    }
  }
  pub(crate) fn truncate(&mut self, n: usize)
  {
    self.data.truncate(n);
  }
  pub(crate) fn sort_by_cached_key<K, F>(&mut self, f: F)
  where
    F: FnMut(&Row) -> K,
    K: Ord,
  {
    self.data.sort_by_cached_key(f);
  }
}

impl IntoIterator for ValueTable
{
  type IntoIter = <Vec<Row> as IntoIterator>::IntoIter;
  type Item = Row;
  fn into_iter(self) -> Self::IntoIter
  {
    self.data.into_iter()
  }
}

impl FromIterator<Row> for ValueTable
{
  fn from_iter<T: IntoIterator<Item = Row>>(iter: T) -> Self
  {
    Self {
      data: iter.into_iter().collect(),
    }
  }
}

impl From<Vec<HashMap<String, crate::graph::Value>>> for ValueTable
{
  fn from(value: Vec<HashMap<String, crate::graph::Value>>) -> Self
  {
    Self { data: value }
  }
}
