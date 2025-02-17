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
  pub(crate) fn from_rows(rows: impl Iterator<Item = Row>) -> Self
  {
    Self {
      data: rows.collect(),
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
