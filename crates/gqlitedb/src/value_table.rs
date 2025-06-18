use crate::prelude::*;

pub(crate) type ColId = usize;

pub(crate) trait RowInterface
{
  fn set(&mut self, index: usize, value: graph::Value) -> Result<()>;
  fn get(&self, index: usize) -> Result<&graph::Value>;
}

pub(crate) struct Row
{
  values: Vec<graph::Value>,
}

impl Row
{
  /// Creates a new Row from an initial vector and a total size.
  /// If `extra_size` > initial.len(), it will fill with default values.
  pub fn new(mut values: Vec<graph::Value>, extra_size: usize) -> Self
  {
    values.resize(extra_size + values.len(), Default::default());
    Row { values }
  }
  fn len(&self) -> usize
  {
    self.values.len()
  }
}

// Implement the RowInterface trait for Row
impl RowInterface for Row
{
  fn set(&mut self, index: usize, value: graph::Value) -> Result<()>
  {
    let values_length = self.values.len();
    let elem = self
      .values
      .get_mut(index)
      .ok_or_else(|| InternalError::InvalidIndex {
        index,
        length: values_length,
      })?;
    *elem = value;
    Ok(())
  }

  fn get(&self, index: usize) -> Result<&graph::Value>
  {
    self.values.get(index).ok_or(
      InternalError::InvalidIndex {
        index,
        length: self.values.len(),
      }
      .into(),
    )
  }
}

#[derive(Debug)]
pub(crate) struct ValueTable
{
  columns: Vec<String>,
  data: Vec<graph::Value>,
}

impl ValueTable
{
  pub(crate) fn new() -> Self
  {
    Self {
      columns: Default::default(),
      data: Default::default(),
    }
  }
  pub(crate) fn add_row(&mut self, mut row: Row) -> Result<()>
  {
    if row.len() != self.columns.len()
    {
      Err(
        error::InternalError::InvalidNumberColumns {
          actual: row.len(),
          expected: self.columns.len(),
        }
        .into(),
      )
    }
    else
    {
      self.data.append(&mut row.values);
      Ok(())
    }
  }

  pub fn row_count(&self) -> usize
  {
    if self.columns.is_empty()
    {
      0
    }
    else
    {
      self.data.len() / self.columns.len()
    }
  }

  /// Create a mutable RowView into a specific row
  pub fn row_view(&mut self, row_index: usize) -> Option<RowView<'_>>
  {
    let num_cols = self.columns.len();
    let start = row_index.checked_mul(num_cols)?;
    let end = start + num_cols;
    if end <= self.data.len()
    {
      Some(RowView {
        row: &mut self.data[start..end],
      })
    }
    else
    {
      None
    }
  }

  // pub(crate) fn iter(&self) -> core::slice::Iter<'_, Row>
  // {
  //   self.data.iter()
  // }
  // pub(crate) fn first_row(&self) -> Option<&Row>
  // {
  //   self.data.first()
  // }
  // pub(crate) fn remove_first_rows(&mut self, n: usize)
  // {
  //   if n < self.data.len()
  //   {
  //     self.data.drain(0..n);
  //   }
  //   else
  //   {
  //     self.data.clear();
  //   }
  // }
  // pub(crate) fn truncate(&mut self, n: usize)
  // {
  //   self.data.truncate(n);
  // }
  // pub(crate) fn sort_by_cached_key<K, F>(&mut self, f: F)
  // where
  //   F: FnMut(&Row) -> K,
  //   K: Ord,
  // {
  //   self.data.sort_by_cached_key(f);
  // }
}

/// A mutable view into a single row of the ValueTable
pub struct RowView<'a>
{
  row: &'a mut [graph::Value], // length = columns.len()
}

impl<'a> RowView<'a>
{
  /// Create an owned Row by cloning the values in this row view
  pub fn to_row(&self) -> Row
  {
    Row {
      values: self.row.to_vec(),
    }
  }
}

impl<'a> RowInterface for RowView<'a>
{
  fn set(&mut self, index: usize, value: graph::Value) -> Result<(), InternalError>
  {
    let values_length = self.row.len();
    let elem = self
      .row
      .get_mut(index)
      .ok_or_else(|| InternalError::InvalidIndex {
        index,
        length: values_length,
      })?;
    *elem = value;
    Ok(())
  }
  fn get(&self, index: usize) -> Result<&graph::Value>
  {
    self.row.get(index).ok_or(
      InternalError::InvalidIndex {
        index,
        length: self.row.len(),
      }
      .into(),
    )
  }
}

// impl IntoIterator for ValueTable
// {
//   type IntoIter = <Vec<Row> as IntoIterator>::IntoIter;
//   type Item = Row;
//   fn into_iter(self) -> Self::IntoIter
//   {
//     self.data.into_iter()
//   }
// }

// impl FromIterator<Row> for ValueTable
// {
//   fn from_iter<T: IntoIterator<Item = Row>>(iter: T) -> Self
//   {
//     Self {
//       data: iter.into_iter().collect(),
//     }
//   }
// }

// impl From<Vec<HashMap<String, crate::graph::Value>>> for ValueTable
// {
//   fn from(value: Vec<HashMap<String, crate::graph::Value>>) -> Self
//   {
//     Self { data: value }
//   }
// }
