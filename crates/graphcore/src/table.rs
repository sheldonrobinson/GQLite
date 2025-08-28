use crate::{prelude::*, Value, ValueTryIntoRef};

/// Table of values
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Table
{
  headers: Vec<String>,
  data: Vec<crate::Value>,
}

impl Table
{
  /// Create a new table, with the given headers and data.
  /// It is assumed that number of column is equalt to headers, and the number
  /// of row is equal to length of data divided by number of columns.
  pub fn new(headers: Vec<String>, data: Vec<crate::Value>) -> Result<Table>
  {
    if data.len() % headers.len() != 0
    {
      Err(error::Error::InvalidTableDimensions)?;
    }
    Ok(Table { headers, data })
  }
  /// Create an empty table
  pub fn empty() -> Table
  {
    Table {
      headers: Default::default(),
      data: Default::default(),
    }
  }
  /// Return true is empty
  pub fn is_empty(&self) -> bool
  {
    self.data.is_empty()
  }
  /// Access to the headers of the table
  pub fn headers(&self) -> &Vec<String>
  {
    &self.headers
  }
  /// number of rows
  pub fn rows(&self) -> usize
  {
    self.data.len() / self.headers.len()
  }
  /// number of columns
  pub fn columns(&self) -> usize
  {
    self.headers.len()
  }
  /// value access
  pub fn get<T>(&self, row: usize, column: usize) -> Result<&T>
  where
    Value: ValueTryIntoRef<T>,
  {
    let v = self.value(row, column)?;
    v.try_into_ref()
  }
  /// value access
  pub fn get_owned<T>(&self, row: usize, column: usize) -> Result<T>
  where
    Value: ValueTryIntoRef<T>,
    T: Clone,
  {
    let v = self.value(row, column)?;
    v.try_into_ref().map(T::to_owned)
  }
  /// value access
  pub fn value(&self, row: usize, column: usize) -> Result<&Value>
  {
    if column > self.headers.len()
    {
      Err(error::Error::InvalidRange)
    }
    else
    {
      self
        .data
        .get(row * self.headers.len() + column)
        .ok_or(error::Error::InvalidRange)
    }
  }
  /// Create an iterator over the rows in the table
  pub fn row_iter(&self) -> RowIter<'_>
  {
    RowIter {
      data: &self.data,
      columns: self.columns(),
      index: 0,
      row_count: self.rows(),
    }
  }
}

impl From<Table> for value::Value
{
  fn from(val: Table) -> Self
  {
    let mut rows: Vec<value::Value> = Default::default();
    let cl = val.headers.len();
    rows.push(val.headers.into());
    let mut it = val.data.into_iter();
    loop
    {
      let first = it.next();
      if let Some(first) = first
      {
        let mut row = Vec::with_capacity(cl);
        row.push(first);
        for _ in 1..cl
        {
          row.push(it.next().unwrap());
        }
        rows.push(row.into());
      }
      else
      {
        break;
      }
    }
    rows.into()
  }
}

/// View of a row
#[derive(Debug)]
pub struct RowView<'a>
{
  row: &'a [value::Value], // length = columns.len()
}

impl<'a> RowView<'a>
{
  /// Iterate over the value of the row view
  pub fn iter(&self) -> std::slice::Iter<'a, value::Value>
  {
    self.row.iter()
  }
  /// value access
  pub fn value(&self, column: usize) -> Result<&Value>
  {
    self.row.get(column).ok_or(error::Error::InvalidRange)
  }
  /// value access
  pub fn get<T>(&self, column: usize) -> Result<&T>
  where
    Value: ValueTryIntoRef<T>,
  {
    let v = self.value(column)?;
    v.try_into_ref()
  }
  /// value access
  pub fn get_owned<T>(&self, column: usize) -> Result<T>
  where
    Value: ValueTryIntoRef<T>,
    T: Clone,
  {
    let v = self.value(column)?;
    v.try_into_ref().map(T::to_owned)
  }
}

/// Iteratpr over rows of a table
pub struct RowIter<'a>
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

/// Convenient macro for creating tables.
///
/// Example:
///
/// ```rust
/// # use graphcore::{Table, table};
/// let table: Table = table!(("a", "b"), [["row0", "row0"], ["row1", "row2"]]);
/// ```
#[macro_export]
macro_rules! table {
  () => (
      $crate::Table::empty()
  );
  (($($header:expr),+ $(,)?), []) => (
    $crate::Table::new(
      vec![$($header.into()),+],
      vec![]
    ).unwrap()
  );
  (($($header:expr),+ $(,)?), [$([$($x:expr),+ $(,)?]),*]) => (
    {
      let headers = vec![$($header.into()),+];
      let mut data = Vec::new();
      $(
          data.extend(vec![$($x.into()),+]);
      )*
      $crate::Table::new(headers, data).unwrap()
    }
  );
}

#[cfg(test)]
mod tests
{
  use crate::Table;

  #[test]
  fn test_table_macro()
  {
    let tb = table!();
    assert_eq!(tb, Table::empty());
    let tb = table!(("a", "b"), []);
    assert_eq!(tb.rows(), 0);
    assert_eq!(tb.columns(), 2);
    assert_eq!(*tb.headers(), vec!["a".to_string(), "b".to_string()]);
    let tb = table!(("a", "b"), [[1, 2], [3, 4]]);
    assert_eq!(tb.rows(), 2);
    assert_eq!(tb.columns(), 2);
    assert_eq!(*tb.headers(), vec!["a".to_string(), "b".to_string()]);
    assert_eq!(*tb.value(0, 0).unwrap(), 1.into());
    assert_eq!(*tb.value(0, 1).unwrap(), 2.into());
    assert_eq!(*tb.value(1, 0).unwrap(), 3.into());
    assert_eq!(*tb.value(1, 1).unwrap(), 4.into());
    assert_eq!(*tb.get::<i64>(0, 0).unwrap(), 1);
    assert_eq!(*tb.get::<i64>(0, 1).unwrap(), 2);
    assert_eq!(*tb.get::<i64>(1, 0).unwrap(), 3);
    assert_eq!(*tb.get::<i64>(1, 1).unwrap(), 4);
  }
}
