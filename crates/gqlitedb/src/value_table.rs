use std::{fmt::Debug, mem};

use crate::prelude::*;

pub(crate) type ColId = usize;

pub(crate) trait RowInterface: Debug
{
  fn get(&self, index: usize) -> Result<&value::Value>;
  fn get_owned(&self, index: usize) -> Result<value::Value>
  {
    Ok(self.get(index)?.to_owned())
  }
  #[cfg(test)]
  fn len(&self) -> usize;
}

pub(crate) trait MutableRowInterface: RowInterface
{
  /// Set the value.
  fn set(&mut self, index: usize, value: value::Value) -> Result<()>;
  /// Set the value, if it is not set.
  fn set_if_unset(&mut self, index: usize, value: value::Value) -> Result<()>;
}

#[derive(Debug, Default, Clone, Hash, PartialEq)]
pub(crate) struct Row
{
  values: Vec<value::Value>,
}

impl Row
{
  /// Creates a new Row from an initial vector and a total size.
  /// If `extra_size` > initial.len(), it will fill with default values.
  pub(crate) fn new(mut values: Vec<value::Value>, extra_size: usize) -> Self
  {
    values.resize(extra_size + values.len(), Default::default());
    Row { values }
  }
  pub(crate) fn len(&self) -> usize
  {
    self.values.len()
  }
  /// Take the value at the given index, and replace it in the current row with an invalid value.
  pub(crate) fn take(&mut self, index: usize) -> Result<value::Value>
  {
    if index < self.values.len()
    {
      Ok(mem::replace(&mut self.values[index], value::Value::Null))
    }
    else
    {
      Err(
        InternalError::InvalidIndex {
          index,
          length: self.values.len(),
        }
        .into(),
      )
    }
  }
  /// Extend the row to the targeted size
  pub(crate) fn extended(self, target_size: usize) -> Result<Self>
  {
    let values_len = self.values.len();
    if target_size < values_len
    {
      Err(
        error::InternalError::InvalidRowLength {
          got: target_size,
          expected: values_len,
        }
        .into(),
      )
    }
    else
    {
      Ok(self.extended_by(target_size - values_len))
    }
  }
  pub(crate) fn extended_by(self, extension: usize) -> Self
  {
    Self::new(self.values, extension)
  }
}

// Implement the RowInterface trait for Row
impl RowInterface for Row
{
  fn get(&self, index: usize) -> Result<&value::Value>
  {
    self.values.get(index).ok_or_else(|| {
      InternalError::InvalidIndex {
        index,
        length: self.values.len(),
      }
      .into()
    })
  }
  #[cfg(test)]
  fn len(&self) -> usize
  {
    self.values.len()
  }
}

impl MutableRowInterface for Row
{
  fn set(&mut self, index: usize, value: value::Value) -> Result<()>
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
  fn set_if_unset(&mut self, index: usize, value: value::Value) -> Result<()>
  {
    let values_length = self.values.len();
    let elem = self
      .values
      .get_mut(index)
      .ok_or_else(|| InternalError::InvalidIndex {
        index,
        length: values_length,
      })?;
    if elem.is_null()
    {
      *elem = value;
    }
    Ok(())
  }
}

impl From<Row> for value::Value
{
  fn from(v: Row) -> Self
  {
    value::Value::Array(v.values)
  }
}

impl FromIterator<value::Value> for Row
{
  fn from_iter<I: IntoIterator<Item = value::Value>>(iter: I) -> Self
  {
    Row::new(iter.into_iter().collect(), 0)
  }
}

impl IntoIterator for Row
{
  type Item = value::Value;
  type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
  fn into_iter(self) -> Self::IntoIter
  {
    self.values.into_iter()
  }
}

pub(crate) trait Header
{
  fn columns(&self) -> usize;
}

impl Header for usize
{
  fn columns(&self) -> usize
  {
    *self
  }
}

impl Header for Vec<String>
{
  fn columns(&self) -> usize
  {
    self.len()
  }
}

#[derive(Debug)]
pub(crate) struct ValueTable<HeaderType>
where
  HeaderType: Header,
{
  header: HeaderType,
  row_count: usize,
  data: Vec<value::Value>,
}

impl<HeaderType> ValueTable<HeaderType>
where
  HeaderType: Header,
{
  pub(crate) fn new(header: HeaderType) -> Self
  {
    Self {
      header,
      row_count: 0,
      data: Default::default(),
    }
  }
  /// Add the full row to the table, return an error if the size of the row doesn't match the column.
  pub(crate) fn add_full_row(&mut self, mut row: Row) -> Result<()>
  {
    if row.len() != self.header.columns()
    {
      Err(
        error::InternalError::InvalidNumberColumns {
          actual: row.len(),
          expected: self.header.columns(),
        }
        .into(),
      )
    }
    else
    {
      self.row_count += 1;
      self.data.append(&mut row.values);
      Ok(())
    }
  }
  /// Add the row to the table, if the row is larger, the row is truncated, if the row is smaller, an error is returned.
  pub(crate) fn add_truncated_row(&mut self, row: Row) -> Result<()>
  {
    if row.len() < self.header.columns()
    {
      Err(
        error::InternalError::InvalidNumberColumns {
          actual: row.len(),
          expected: self.header.columns(),
        }
        .into(),
      )
    }
    else
    {
      self.row_count += 1;
      self
        .data
        .extend(row.values.into_iter().take(self.header.columns()));
      Ok(())
    }
  }

  pub fn add_truncated_rows(&mut self, rows: Vec<Row>) -> Result<()>
  {
    for row in rows.into_iter()
    {
      self.add_truncated_row(row)?;
    }
    Ok(())
  }

  #[allow(dead_code)]
  pub fn row_count(&self) -> usize
  {
    self.row_count
  }

  /// Create a RowView into a specific row
  #[allow(dead_code)]
  pub fn row_view(&self, row_index: usize) -> Option<RowView<'_>>
  {
    if self.header.columns() == 0 && row_index >= self.row_count
    {
      return None;
    }
    let columns = self.header.columns();
    let start = row_index.checked_mul(columns)?;
    let end = start + columns;
    if end <= self.data.len()
    {
      Some(RowView {
        row: &self.data[start..end],
      })
    }
    else
    {
      None
    }
  }
  /// Create an iterator over the rows in the table
  pub(crate) fn row_iter(&self) -> RowIter<'_>
  {
    RowIter {
      data: &self.data,
      columns: self.header.columns(),
      index: 0,
      row_count: self.row_count,
    }
  }
  /// Transform into an iterator over the rows in the table
  pub(crate) fn into_row_iter(self) -> IntoRowIter
  {
    IntoRowIter {
      data: self.data.into_iter(),
      columns: self.header.columns(),
      row_count: self.row_count,
      index: 0,
    }
  }
  pub(crate) fn remove_first_rows(&mut self, n: usize)
  {
    let n = n * self.header.columns();
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
    let n = n * self.header.columns();
    self.data.truncate(n);
  }
}

/// A view into a single row of the ValueTable
#[derive(Debug)]
pub(crate) struct RowView<'a>
{
  row: &'a [value::Value], // length = columns.len()
}

impl<'a> RowView<'a>
{
  #[allow(dead_code)]
  /// Create an owned Row by cloning the values in this row view
  pub fn to_row(&self) -> Row
  {
    Row {
      values: self.row.to_vec(),
    }
  }
  #[allow(dead_code)]
  /// Create an owned Row by cloning the values in this row view, and extend it to the given size
  pub fn to_extended_row(&self, length: usize) -> Result<Row>
  {
    Row {
      values: self.row.to_vec(),
    }
    .extended(length)
  }
}

impl<'a> RowInterface for RowView<'a>
{
  fn get(&self, index: usize) -> Result<&value::Value>
  {
    self.row.get(index).ok_or_else(|| {
      InternalError::InvalidIndex {
        index,
        length: self.row.len(),
      }
      .into()
    })
  }
  #[cfg(test)]
  fn len(&self) -> usize
  {
    self.row.len()
  }
}

/// Iterator over the row of the value table that yields RowViews
pub(crate) struct RowIter<'a>
{
  data: &'a Vec<value::Value>,
  columns: usize,
  index: usize,
  row_count: usize,
}

impl<'a> Iterator for RowIter<'a>
{
  type Item = RowView<'a>;

  fn next(&mut self) -> Option<Self::Item>
  {
    if self.columns == 0
    {
      if self.index < self.row_count
      {
        self.index += 1;
        Some(RowView {
          row: &self.data[0..0],
        })
      }
      else
      {
        None
      }
    }
    else
    {
      let start = self.index * self.columns;
      let end = start + self.columns;

      if self.columns == 0 || end > self.data.len()
      {
        return None;
      }

      let rv = RowView {
        row: &self.data[start..end],
      };

      self.index += 1;

      Some(rv)
    }
  }
}

pub(crate) struct IntoRowIter
{
  data: std::vec::IntoIter<value::Value>,
  columns: usize,
  /// index used to output empty rows, when columns is 0
  index: usize,
  row_count: usize,
}

impl Iterator for IntoRowIter
{
  type Item = Row;

  fn next(&mut self) -> Option<Self::Item>
  {
    if self.columns == 0
    {
      if self.index < self.row_count
      {
        self.index += 1;
        Some(Row::default())
      }
      else
      {
        None
      }
    }
    else
    {
      let v: Vec<_> = self.data.by_ref().take(self.columns).collect();
      if v.is_empty()
      {
        None
      }
      else
      {
        Some(Row::new(v, 0))
      }
    }
  }
}

fn from_row_iterator<T>(iter: impl IntoIterator<Item = T>) -> Result<ValueTable<usize>>
where
  T: TryInto<Row>,
  crate::prelude::ErrorType: From<<T as TryInto<Row>>::Error>,
{
  let mut data = Vec::new();
  let mut header = None;
  let mut row_count = 0;

  let iter = iter.into_iter();
  let (size_hint, _) = iter.size_hint();

  for row in iter
  {
    let row = row.try_into()?;
    row_count += 1;
    match header
    {
      Some(header) =>
      {
        if header != row.len()
        {
          return Err(
            InternalError::InvalidRowLength {
              got: row.len(),
              expected: header,
            }
            .into(),
          );
        }
      }
      None =>
      {
        header = Some(row.len());
        data.reserve(row.len() * size_hint);
      }
    }

    data.extend(row.values);
  }

  let header = header.unwrap_or_default();
  Ok(ValueTable {
    header,
    data,
    row_count,
  })
}

impl FromIterator<Row> for Result<ValueTable<usize>>
{
  fn from_iter<I: IntoIterator<Item = Row>>(iter: I) -> Self
  {
    from_row_iterator(iter)
  }
}

pub(crate) struct RowResult(pub Result<Row>);

impl TryInto<Row> for RowResult
{
  type Error = ErrorType;
  fn try_into(self) -> Result<Row>
  {
    self.0
  }
}

impl FromIterator<RowResult> for Result<ValueTable<usize>>
{
  fn from_iter<I: IntoIterator<Item = RowResult>>(iter: I) -> Self
  {
    from_row_iterator(iter)
  }
}

impl TryFrom<Vec<Row>> for ValueTable<usize>
{
  type Error = crate::prelude::ErrorType;
  fn try_from(value: Vec<Row>) -> Result<Self>
  {
    value.into_iter().collect()
  }
}

#[cfg(test)]
mod tests
{
  use super::*;
  #[test]
  fn test_table_0_column()
  {
    let mut table = ValueTable::new(0);

    // Test emptiness
    assert_eq!(table.row_count(), 0);
    let mut it = table.row_iter();
    assert!(it.next().is_none());
    assert!(table.row_view(0).is_none());

    // Add a row
    table.add_full_row(Default::default()).unwrap();
    assert_eq!(table.row_count(), 1);
    let mut it = table.row_iter();
    let first_row = it.next().unwrap();
    assert_eq!(first_row.len(), 0);
    assert!(it.next().is_none());
    assert!(table.row_view(0).is_some());
    assert!(table.row_view(1).is_none());
  }
  #[test]
  fn test_table_1_column()
  {
    let mut table = ValueTable::new(1);

    // Test emptiness
    assert_eq!(table.row_count(), 0);
    let mut it = table.row_iter();
    assert!(it.next().is_none());
    assert!(table.row_view(0).is_none());

    // Add a row
    table.add_full_row(Row::new(vec![1.into()], 0)).unwrap();
    assert_eq!(table.row_count(), 1);
    let mut it = table.row_iter();
    let first_row = it.next().unwrap();
    assert_eq!(first_row.len(), 1);
    assert_eq!(*first_row.get(0).unwrap(), (1 as i64).into());
    assert!(it.next().is_none());
    let first_row = table.row_view(0).unwrap();
    assert_eq!(first_row.len(), 1);
    assert_eq!(*first_row.get(0).unwrap(), (1 as i64).into());
    assert!(table.row_view(1).is_none());
  }
  #[test]
  fn test_row()
  {
    let row = Row::new(vec!["a".into(), 1.0.into()], 1);
    assert_eq!(row.len(), 3);
    assert_eq!(*row.get(0).unwrap(), "a".into());
    assert_eq!(*row.get(1).unwrap(), 1.0.into());
    assert!(row.get(2).unwrap().is_null());
    assert!(row.get(3).is_err());

    let row = row.extended(4).unwrap();
    assert_eq!(row.len(), 4);
    assert_eq!(*row.get(0).unwrap(), "a".into());
    assert_eq!(*row.get(1).unwrap(), 1.0.into());
    assert!(row.get(2).unwrap().is_null());
    assert!(row.get(3).unwrap().is_null());
    assert!(row.get(4).is_err());
  }
}
