use std::cmp;

use itertools::Itertools;

use crate::prelude::*;

pub(crate) enum Ordering
{
  Equal,
  Different,
  Less,
  Greater,
  ComparedNull,
  Null,
}

impl From<cmp::Ordering> for Ordering
{
  fn from(value: cmp::Ordering) -> Self
  {
    match value
    {
      cmp::Ordering::Equal => Ordering::Equal,
      cmp::Ordering::Greater => Ordering::Greater,
      cmp::Ordering::Less => Ordering::Less,
    }
  }
}

fn compare_map(lhs: &value::ValueMap, rhs: &value::ValueMap) -> Ordering
{
  if lhs.len() == rhs.len()
  {
    lhs
      .iter()
      .sorted_by(|(lk, _), (rk, _)| lk.cmp(rk))
      .zip(rhs.iter().sorted_by(|(lk, _), (rk, _)| lk.cmp(rk)))
      .map(|((lhs_k, lhs_v), (rhs_k, rhs_v))| {
        let cmp = lhs_k.cmp(rhs_k);
        match cmp
        {
          std::cmp::Ordering::Equal => compare(lhs_v, rhs_v),
          o => o.into(),
        }
      })
      .find(|p| !matches!(p, Ordering::Equal))
      .unwrap_or(Ordering::Equal)
  }
  else if lhs.len() < rhs.len()
  {
    Ordering::Less
  }
  else
  {
    Ordering::Greater
  }
}

fn compare_f64(lhs: &f64, rhs: &f64) -> Ordering
{
  if lhs.is_nan() || rhs.is_nan()
  {
    Ordering::Different
  }
  else
  {
    lhs.total_cmp(rhs).into()
  }
}

fn compare_node(lhs: &graph::Node, rhs: &graph::Node) -> Ordering
{
  lhs.key().uuid().cmp(&rhs.key().uuid()).into()
}

pub(crate) fn compare(lhs: &value::Value, rhs: &value::Value) -> Ordering
{
  use value::Value;
  match lhs
  {
    Value::Null => Ordering::ComparedNull,
    Value::Key(kl) => match rhs
    {
      Value::Key(kr) => kl.uuid().cmp(&kr.uuid()).into(),
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::Boolean(bl) => match rhs
    {
      Value::Boolean(br) => bl.cmp(br).into(),
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::Integer(il) => match rhs
    {
      Value::Integer(ir) => il.cmp(ir).into(),
      Value::Float(fr) => compare_f64(&(*il as f64), fr),
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::Float(fl) => match rhs
    {
      Value::Integer(ir) => compare_f64(fl, &(*ir as f64)),
      Value::Float(fr) => compare_f64(fl, fr),
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::String(sl) => match rhs
    {
      Value::String(sr) => sl.cmp(sr).into(),
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::Array(al) => match rhs
    {
      Value::Array(ar) =>
      {
        if al.len() < ar.len()
        {
          Ordering::Less
        }
        else if al.len() > ar.len()
        {
          Ordering::Greater
        }
        else
        {
          let mut comp_to_null = false;
          let mut compare_equal = true;
          let comp = al
            .iter()
            .zip(ar.iter())
            .find_map(|(l, r)| match compare(l, r)
            {
              Ordering::Equal => None,
              Ordering::Null | Ordering::Different =>
              {
                compare_equal = false;
                None
              }
              Ordering::Greater => Some(Ordering::Greater),
              Ordering::Less => Some(Ordering::Less),
              Ordering::ComparedNull =>
              {
                comp_to_null = true;
                None
              }
            });

          // Due to opencypher madness, if you compare [nil, 2] with [1, 2] this should return null because of the nil/1 comparison,
          // however [nil, 2] with [1, "2"] should return false because 2 != "2" 🤦
          match (comp, comp_to_null, compare_equal)
          {
            // Some imply either greater or lesser, in which case, return the value unless a comparison to null was made
            (Some(o), false, _) => o,
            (Some(_), true, _) => Ordering::ComparedNull,
            // If equal, check if a comparison to null was made
            (None, true, true) => Ordering::ComparedNull,
            (None, false, true) => Ordering::Equal,
            // If false, ignore comparison to null
            (None, _, false) => Ordering::Null,
          }
        }
      }
      Value::Null => Ordering::ComparedNull,
      _ => Ordering::Null,
    },
    Value::Map(lhs) => match rhs
    {
      Value::Map(rhs) => compare_map(lhs, rhs),
      _ => Ordering::Null,
    },
    Value::Node(lhs) => match rhs
    {
      Value::Node(rhs) => compare_node(lhs, rhs),
      _ => Ordering::Null,
    },
    Value::Edge(lhs) => match rhs
    {
      Value::Edge(rhs) => lhs.key().uuid().cmp(&rhs.key().uuid()).into(),
      _ => Ordering::Null,
    },
    Value::Path(lhs) => match rhs
    {
      Value::Path(rhs) =>
      {
        let labels_cmp = lhs.labels().cmp(rhs.labels());
        match labels_cmp
        {
          std::cmp::Ordering::Equal => match compare_map(lhs.properties(), rhs.properties())
          {
            Ordering::Equal => match compare_node(lhs.source(), rhs.source())
            {
              Ordering::Equal => compare_node(lhs.destination(), rhs.destination()),
              o => o,
            },
            o => o,
          },
          o => o.into(),
        }
      }
      _ => Ordering::Null,
    },
  }
}
