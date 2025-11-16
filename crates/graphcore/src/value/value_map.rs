use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
  hash::Hash,
  ops::{Deref, DerefMut},
};

use crate::{prelude::*, Value};

type ValueMapInner = std::collections::HashMap<String, Value>;

/// A map of values.
#[derive(Debug, PartialEq, Default, Clone, Deserialize, Serialize)]
pub struct ValueMap(ValueMapInner);

impl ValueMap
{
  /// Create a default value map.
  pub fn new() -> Self
  {
    Self::default()
  }
  pub(crate) fn remove_null(self) -> Self
  {
    self
      .into_iter()
      .filter(|(_, v)| !v.is_null())
      .map(|(k, v)| (k, v.remove_null()))
      .collect()
  }
  /// Remove a value in the given path.
  pub fn remove_value<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
  ) -> Result<()>
  {
    if let Some(field) = field
    {
      if let Some(next_field) = path.next()
      {
        let v = self.get_mut(field);
        match v
        {
          Some(Value::Map(o)) =>
          {
            o.remove_value(Some(next_field), path)?;
          }
          None =>
          {}
          _ => Err(Error::MissingKeyInPath {
            key: field.to_owned(),
          })?,
        }
      }
      else
      {
        self.remove(field);
      }
    }
    else
    {
      Err(Error::MissingKey)?
    }
    Ok(())
  }
  /// Add a value at the given path.
  pub fn add_values<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
    value: ValueMap,
  ) -> Result<()>
  {
    if let Some(field) = field
    {
      let v = self.get_mut(field);

      if let Some(next_field) = path.next()
      {
        match v
        {
          Some(Value::Map(o)) =>
          {
            o.add_values(Some(next_field), path, value)?;
          }
          None =>
          {
            let mut o = ValueMap::new();
            o.set_value(Some(next_field), path, value.remove_null().into())?;
            self.insert(field.to_owned(), o.into());
          }
          _ => Err(Error::MissingKeyInPath {
            key: field.to_owned(),
          })?,
        }
      }
      else
      {
        match v
        {
          Some(v) => match v
          {
            Value::Map(object) =>
            {
              for (k, v) in value.into_iter()
              {
                if v.is_null()
                {
                  object.remove(&k);
                }
                else
                {
                  object.insert(k, v);
                }
              }
            }
            _ => Err(Error::MissingKeyInPath {
              key: field.to_owned(),
            })?,
          },
          None =>
          {
            self.insert(field.to_owned(), value.remove_null().into());
          }
        }
      }
    }
    else
    {
      for (k, v) in value.into_iter()
      {
        if v.is_null()
        {
          self.remove(&k);
        }
        else
        {
          self.insert(k, v);
        }
      }
    }
    Ok(())
  }
  /// Set the value in the given path. Unlike add_value, it will complain if there is no existing value.
  pub fn set_value<'a>(
    &mut self,
    field: Option<&'a String>,
    mut path: impl Iterator<Item = &'a String>,
    value: Value,
  ) -> Result<()>
  {
    if let Some(field) = field
    {
      let v = self.get_mut(field);

      if let Some(next_field) = path.next()
      {
        match v
        {
          Some(Value::Map(o)) =>
          {
            o.set_value(Some(next_field), path, value)?;
          }
          None =>
          {
            if !value.is_null()
            {
              let mut o = ValueMap::new();
              o.set_value(Some(next_field), path, value)?;
              self.insert(field.to_owned(), o.into());
            }
          }
          _ => Err(Error::MissingKeyInPath {
            key: field.to_owned(),
          })?,
        }
      }
      else if value.is_null()
      {
        self.remove(field);
      }
      else
      {
        match v
        {
          Some(v) =>
          {
            *v = value;
          }
          None =>
          {
            self.insert(field.to_owned(), value);
          }
        }
      }

      Ok(())
    }
    else
    {
      match value
      {
        Value::Map(o) =>
        {
          *self = o;
          Ok(())
        }
        _ => Err(Error::MissingKey),
      }
    }
  }
}

impl Hash for ValueMap
{
  fn hash<H: std::hash::Hasher>(&self, state: &mut H)
  {
    for (k, v) in self.0.iter().sorted_by(|(lk, _), (rk, _)| lk.cmp(rk))
    {
      k.hash(state);
      v.hash(state);
    }
  }
}

impl Deref for ValueMap
{
  type Target = ValueMapInner;
  fn deref(&self) -> &Self::Target
  {
    &self.0
  }
}

impl DerefMut for ValueMap
{
  fn deref_mut(&mut self) -> &mut Self::Target
  {
    &mut self.0
  }
}

impl FromIterator<(String, Value)> for ValueMap
{
  /// Constructs a `HashMap<K, V>` from an iterator of key-value pairs.
  ///
  /// If the iterator produces any pairs with equal keys,
  /// all but one of the corresponding values will be dropped.
  fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> ValueMap
  {
    let mut map = ValueMapInner::with_hasher(Default::default());
    map.extend(iter);
    ValueMap(map)
  }
}

impl IntoIterator for ValueMap
{
  type Item = (String, Value);
  type IntoIter = <ValueMapInner as IntoIterator>::IntoIter;
  fn into_iter(self) -> Self::IntoIter
  {
    self.0.into_iter()
  }
}

impl<const N: usize> From<[(String, Value); N]> for ValueMap
{
  fn from(arr: [(String, Value); N]) -> Self
  {
    Self(ValueMapInner::from_iter(arr))
  }
}

impl std::fmt::Display for ValueMap
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    write!(f, "{{")?;
    self.iter().enumerate().for_each(|(n, (k, v))| {
      if n == 0
      {
        write!(f, "{}: {}", k, v).unwrap();
      }
      else
      {
        write!(f, ", {}: {}", k, v).unwrap();
      }
    });
    write!(f, "}}")
  }
}
