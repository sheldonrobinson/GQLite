use std::collections::HashMap;

use crate::{
  interpreter::instructions::{Block, VariablesSizes},
  prelude::*,
  store::TransactionBoxable,
  value_table::{MutableRowInterface, Row, RowInterface},
};
use interpreter::instructions;

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Value
{
  GraphValue(value::Value),
  NodeQuery(store::SelectNodeQuery),
  EdgeQuery(store::SelectEdgeQuery),
}

impl<T> From<T> for Value
where
  T: Into<value::Value>,
{
  fn from(value: T) -> Self
  {
    Self::GraphValue(value.into())
  }
}

macro_rules! try_into_gv_impl {
  ($vn:ty) => {
    impl TryInto<$vn> for Value
    {
      type Error = ErrorType;
      fn try_into(self) -> crate::Result<$vn>
      {
        let a: value::Value = self.try_into()?;
        Ok(a.try_into()?)
      }
    }
    impl TryPopInto<$vn> for Stack
    {
      fn try_pop_into(&mut self) -> crate::Result<$vn>
      {
        self.try_pop()?.try_into()
          .map_err(|e: ErrorType| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))
      }
      fn try_drain_into(&mut self, n: usize) -> Result<Vec<$vn>>
      {
        self.try_drain(n)?.map(|x| x.try_into()).collect::<Result<_>>()
          .map_err(|e| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))
      }
    }
  };
}

try_into_gv_impl! {bool}
try_into_gv_impl! {i64}
try_into_gv_impl! {f64}
try_into_gv_impl! {String}
try_into_gv_impl! {graph::Node}
try_into_gv_impl! {graph::Edge}

macro_rules! try_into_impl {
  ($typename:tt, $type:ty, $errorname:tt) => {
    impl TryInto<$type> for Value
    {
      type Error = ErrorType;
      fn try_into(self) -> std::result::Result<$type, Self::Error>
      {
        match self
        {
          Self::$typename(gv) => Ok(gv),
          _ => Err(
            InternalError::$errorname {
              context: "try_into",
            }
            .into(),
          ),
        }
      }
    }
    impl TryPopInto<$type> for Stack
    {
      fn try_pop_into(&mut self) -> Result<$type>
      {
        self.try_pop()?.try_into()
        .map_err(|e: ErrorType| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))

      }
      fn try_drain_into(&mut self, n: usize) -> Result<Vec<$type>>
      {
        self.try_drain(n)?.map(|x| x.try_into()).collect::<Result<_>>()
          .map_err(|e| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))
      }
    }
  };
}

try_into_impl! {GraphValue, value::Value, ExpectedGraphValue}
try_into_impl! {NodeQuery, store::SelectNodeQuery, ExpectedNodeQuery}
try_into_impl! {EdgeQuery, store::SelectEdgeQuery, ExpectedEdgeQuery}

impl From<store::SelectNodeQuery> for Value
{
  fn from(value: store::SelectNodeQuery) -> Self
  {
    Self::NodeQuery(value)
  }
}

impl From<store::SelectEdgeQuery> for Value
{
  fn from(value: store::SelectEdgeQuery) -> Self
  {
    Self::EdgeQuery(value)
  }
}

#[allow(unused_macros)]
macro_rules! check_for_null {
  ($a: expr) => {
    if $a.is_null() {
      return Ok(crate::value::Value::Null)
    }
};
($a: expr, $($b:expr), *) => {
  check_for_null!($a);
  check_for_null!($($b),*);
}
}

macro_rules! ordering_to_value {
  ($expression:expr, $true_pattern:pat, $null_value:expr ) => {
    match $expression
    {
      $true_pattern => true.into(),
      value::Ordering::ComparedNull => value::Value::Null,
      value::Ordering::Null => $null_value,
      _ => false.into(),
    }
  };
}

macro_rules! contain_to_value {
  ($expression:expr, $true_pattern:pat ) => {
    match $expression
    {
      $true_pattern => true.into(),
      value::ContainResult::ComparedNull => value::Value::Null,
      _ => false.into(),
    }
  };
}

#[derive(Default, Debug)]
struct Stack
{
  stack: Vec<Value>,
}

impl Stack
{
  fn push(&mut self, value: impl Into<Value>)
  {
    self.stack.push(value.into());
  }
  fn try_pop(&mut self) -> Result<Value>
  {
    self.stack.pop().ok_or(InternalError::EmptyStack.into())
  }
  fn try_last(&self) -> Result<&Value>
  {
    self.stack.last().ok_or(InternalError::EmptyStack.into())
  }
  fn try_drain(&mut self, len: usize) -> Result<std::vec::Drain<'_, Value>>
  {
    if len > self.stack.len()
    {
      Err(InternalError::EmptyStack.into())
    }
    else
    {
      Ok(self.stack.drain((self.stack.len() - len)..))
    }
  }
  fn into_vec(self) -> Vec<Value>
  {
    self.stack
  }
  fn try_pop_as_boolean(&mut self) -> Result<bool>
  {
    let v = self.try_pop()?;
    match v
    {
      Value::GraphValue(value::Value::Null) | Value::GraphValue(value::Value::Boolean(false)) =>
      {
        Ok(false)
      }
      _ => Ok(true),
    }
  }
}

trait TryPopInto<T>
{
  fn try_pop_into(&mut self) -> Result<T>;
  fn try_drain_into(&mut self, n: usize) -> Result<Vec<T>>;
}

impl<T> TryPopInto<T> for Stack
where
  T: TryFrom<Value, Error = ErrorType>,
{
  fn try_pop_into(&mut self) -> Result<T>
  {
    self.try_pop()?.try_into()
      .map_err(|e: ErrorType| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))
  }
  fn try_drain_into(&mut self, n: usize) -> Result<Vec<T>>
  {
    self.try_drain(n)?.map(|x| x.try_into()).collect::<Result<Vec<_>>>()
      .map_err(|e| error::map_error!(e, Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))
  }
}

fn execute_boolean_operator(
  stack: &mut Stack,
  instruction: &instructions::Instruction,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  let a: value::Value = a.try_into()?;
  let b: value::Value = b.try_into()?;
  match instruction
  {
    instructions::Instruction::AndBinaryOperator =>
    {
      if a.is_null()
      {
        if b.is_null() || <value::Value as TryInto<bool>>::try_into(b)?
        {
          stack.push(value::Value::Null);
        }
        else
        {
          stack.push(false);
        }
      }
      else
      {
        let a: bool = a.try_into()?;
        if a
        {
          if b.is_null()
          {
            stack.push(value::Value::Null);
          }
          else
          {
            stack.push(b);
          }
        }
        else
        {
          stack.push(false);
        }
      }
    }
    instructions::Instruction::OrBinaryOperator =>
    {
      if a.is_null()
      {
        if b.is_null() || !<value::Value as TryInto<bool>>::try_into(b)?
        {
          stack.push(value::Value::Null);
        }
        else
        {
          stack.push(true);
        }
      }
      else
      {
        let a: bool = a.try_into()?;
        if a
        {
          stack.push(true);
        }
        else if b.is_null()
        {
          stack.push(value::Value::Null);
        }
        else
        {
          stack.push(b);
        }
      }
    }
    instructions::Instruction::XorBinaryOperator =>
    {
      if a.is_null() || b.is_null()
      {
        stack.push(value::Value::Null);
      }
      else
      {
        let a: bool = a.try_into()?;
        let b: bool = b.try_into()?;
        stack.push(a ^ b);
      }
    }
    _ => Err(InternalError::Unreachable {
      context: "evaluator/execute_boolean_operator",
    })?,
  }

  Ok(())
}

fn execute_binary_operator<T: Into<crate::value::Value>>(
  stack: &mut Stack,
  operand: impl FnOnce(crate::value::Value, crate::value::Value) -> Result<T, graphcore::Error>,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  stack.push(operand(a.try_into()?, b.try_into()?)?.into());
  Ok(())
}

fn eval_instructions(
  stack: &mut Stack,
  row: &impl value_table::RowInterface,
  instructions: &instructions::Instructions,
  parameters: &crate::value::ValueMap,
) -> Result<()>
{
  if crate::consts::SHOW_EVALUATOR_STATE
  {
    println!("----------- eval_instructions");
  }
  for instruction in instructions
  {
    if crate::consts::SHOW_EVALUATOR_STATE
    {
      println!(
        "-- instruction {:#?}\n-- stack {:#?}\n-- row {:#?}",
        instruction, stack, row
      );
    }
    match instruction
    {
      instructions::Instruction::CreateEdgeLiteral { labels } =>
      {
        let props: value::Value = stack.try_pop_into()?;
        let dst: graph::Node = stack.try_pop_into()?;
        let src: graph::Node = stack.try_pop_into()?;
        stack.push(crate::graph::Path::new(
          graph::Key::default(),
          src,
          labels.to_owned(),
          props.into_map(),
          dst,
        ));
      }
      instructions::Instruction::CreateNodeLiteral { labels } =>
      {
        let props: value::Value = stack.try_pop_into()?;
        stack.push(crate::graph::Node::new(
          crate::graph::Key::default(),
          labels.clone(),
          props.into_map(),
        ));
      }
      instructions::Instruction::CreateEdgeQuery { labels } =>
      {
        let props: value::Value = stack.try_pop_into()?;
        let dst: store::SelectNodeQuery = stack.try_pop_into()?;
        let src: store::SelectNodeQuery = stack.try_pop_into()?;
        match props
        {
          value::Value::Edge(ed) =>
          {
            stack.push(store::SelectEdgeQuery::select_source_destination_keys(
              src,
              [ed.key()],
              dst,
            ));
          }
          value::Value::Map(ob) =>
          {
            stack.push(
              store::SelectEdgeQuery::select_source_destination_labels_properties(
                src,
                labels.clone(),
                ob,
                dst,
              ),
            );
          }
          value::Value::Null =>
          {
            stack.push(store::SelectEdgeQuery::select_none());
          }
          _ => Err(RunTimeError::InvalidValueCast {
            value: Box::new(props),
            typename: "Edge properties",
          })?,
        }
      }
      instructions::Instruction::CreateNodeQuery { labels } =>
      {
        let props: value::Value = stack.try_pop_into()?;
        match props
        {
          value::Value::Node(no) =>
          {
            stack.push(store::SelectNodeQuery::select_keys([no.key()]));
          }
          value::Value::Map(ob) =>
          {
            stack.push(store::SelectNodeQuery::select_labels_properties(
              labels.clone(),
              ob,
            ));
          }
          value::Value::Null =>
          {
            stack.push(store::SelectNodeQuery::select_none());
          }
          _ => Err(RunTimeError::InvalidValueCast {
            value: Box::new(props),
            typename: "Node properties",
          })?,
        }
      }
      instructions::Instruction::FunctionCall {
        function,
        arguments_count,
      } =>
      {
        let args: Vec<value::Value> = stack.try_drain_into(*arguments_count)?;
        if args.len() != *arguments_count
        {
          Err(InternalError::MissingStackValue {
            context: "eval_instructions/FunctionCall",
          })?;
        }
        stack.push(function.call(args)?);
      }
      instructions::Instruction::Push { value } =>
      {
        stack.push(value.clone());
      }
      instructions::Instruction::GetVariable { col_id } =>
      {
        stack.push(row.get_owned(*col_id)?);
      }
      instructions::Instruction::GetParameter { name } =>
      {
        if let Some(value) = parameters.get(name)
        {
          stack.push(value.to_owned());
        }
        else
        {
          return Err(
            crate::error::RunTimeError::UnknownParameter {
              name: name.to_owned(),
            }
            .into(),
          );
        }
      }
      instructions::Instruction::CreateArray { length } =>
      {
        let mut m = vec![];
        for _ in 0..*length
        {
          m.push(stack.try_pop_into()?);
        }
        m.reverse();
        stack.push(value::Value::Array(m));
      }
      instructions::Instruction::CreateMap { keys } =>
      {
        let mut m = crate::value::ValueMap::new();
        for k in keys.iter().rev()
        {
          m.insert(k.to_owned(), stack.try_pop_into()?);
        }
        stack.push(value::Value::Map(m));
      }
      instructions::Instruction::MemberAccess { path } =>
      {
        let v: value::Value = stack.try_pop_into()?;
        stack.push(v.access(path.iter()));
      }
      instructions::Instruction::IndexAccess =>
      {
        // Implement access to array or map (edge/node properties).
        // Get the index
        let index: value::Value = stack.try_pop_into()?;
        if index.is_null()
        {
          stack.try_drain(1)?;
          stack.push(value::Value::Null);
        }
        else
        {
          // Get the array/map
          let container: value::Value = stack.try_pop_into()?;
          match container
          {
            value::Value::Array(array) =>
            {
              let idx: i64 = index.try_into()
              .map_err(|e: graphcore::Error| error::map_error!(e.into(), Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))?;
              stack.push(
                array
                  .get(idx as usize)
                  .ok_or(RunTimeError::OutOfBound)?
                  .to_owned(),
              );
            }
            value::Value::Map(map) =>
            {
              let idx: String = index.try_into()
              .map_err(|e: graphcore::Error| error::map_error!(e.into(), Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::MapElementAccessByNonString ))?;
              stack.push(map.get(&idx).unwrap_or(&value::Value::Null).to_owned());
            }
            value::Value::Node(node) =>
            {
              let idx: String = index.try_into()
              .map_err(|e: graphcore::Error| error::map_error!(e.into(), Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))?;
              stack.push(
                node
                  .properties()
                  .get(&idx)
                  .unwrap_or(&value::Value::Null)
                  .to_owned(),
              );
            }
            value::Value::Edge(edge) =>
            {
              let idx: String = index.try_into()
              .map_err(|e: graphcore::Error| error::map_error!(e.into(), Error::RunTime(RunTimeError::InvalidValueCast{..}) => RunTimeError::InvalidArgumentType ))?;
              stack.push(
                edge
                  .properties()
                  .get(&idx)
                  .unwrap_or(&value::Value::Null)
                  .to_owned(),
              );
            }
            value::Value::Null => stack.push(value::Value::Null),
            _ => Err(error::RunTimeError::InvalidArgumentType)?,
          }
        }
      }
      instructions::Instruction::RangeAccess { start, end } =>
      {
        let end: Option<value::Value> = if *end
        {
          Some(stack.try_pop_into()?)
        }
        else
        {
          None
        };
        let start: Option<value::Value> = if *start
        {
          Some(stack.try_pop_into()?)
        }
        else
        {
          None
        };
        // Get the array out of the stack
        let v: value::Value = stack.try_pop_into()?;
        // if either end or start are null, return null
        if end.as_ref().is_some_and(|e| e.is_null()) || start.as_ref().is_some_and(|s| s.is_null())
        {
          stack.push(value::Value::Null);
        }
        else
        {
          let mut start: Option<i64> = start.map(|x| x.try_into()).transpose()?;
          let mut end: Option<i64> = end.map(|x| x.try_into()).transpose()?;
          let v: Vec<value::Value> = v.try_into()?;
          // Compute range length
          let length = match (start, end)
          {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
          };
          if length.is_some_and(|l| l >= v.len() as i64)
          {
            stack.push(v);
          }
          else
          {
            // If start is negative, it should be made into a positive number
            while start.is_some_and(|x| x < 0)
            {
              start = start.map(|x| x + v.len() as i64);
              end = end.map(|x| x + v.len() as i64);
            }
            let end = end.map(|x| x.min(v.len() as i64));
            let v = match (start, end)
            {
              (Some(start), Some(end)) =>
              {
                if end < start
                {
                  Vec::<value::Value>::default()
                }
                else
                {
                  v[start as usize..end as usize].to_owned()
                }
              }
              (Some(start), None) => v[start as usize..].to_owned(),
              (None, Some(end)) => v[..end as usize].to_owned(),
              (None, None) => v.to_owned(),
            };
            stack.push(v);
          }
        }
      }
      instructions::Instruction::Duplicate => stack.push(stack.try_last()?.clone()),
      &instructions::Instruction::Rot3 =>
      {
        let a = stack.try_pop()?;
        let b = stack.try_pop()?;
        let c = stack.try_pop()?;
        stack.push(a);
        stack.push(c);
        stack.push(b);
      }
      &instructions::Instruction::InverseRot3 =>
      {
        let a = stack.try_pop()?;
        let b = stack.try_pop()?;
        let c = stack.try_pop()?;
        stack.push(b);
        stack.push(a);
        stack.push(c);
      }
      &instructions::Instruction::Swap =>
      {
        let a = stack.try_pop()?;
        let b = stack.try_pop()?;
        stack.push(a);
        stack.push(b);
      }
      &instructions::Instruction::AndBinaryOperator
      | &instructions::Instruction::OrBinaryOperator
      | &instructions::Instruction::XorBinaryOperator =>
      {
        execute_boolean_operator(stack, instruction)?;
      }
      &instructions::Instruction::NotUnaryOperator =>
      {
        let a: value::Value = stack.try_pop_into()?;
        match a
        {
          value::Value::Null => stack.push(value::Value::Null),
          value::Value::Boolean(b) => stack.push(!b),
          _ => Err(RunTimeError::InvalidArgumentType)?,
        }
      }
      &instructions::Instruction::NegationUnaryOperator =>
      {
        let a: crate::value::Value = stack.try_pop_into()?;
        stack.push((-a)?);
      }
      &instructions::Instruction::IsNullUnaryOperator =>
      {
        let a: crate::value::Value = stack.try_pop_into()?;
        stack.push(a.is_null());
      }
      &instructions::Instruction::EqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Equal,
            false.into()
          ))
        })?;
      }
      &instructions::Instruction::NotEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Less | value::Ordering::Greater | value::Ordering::Different,
            true.into()
          ))
        })?;
      }
      &instructions::Instruction::InferiorBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Less,
            value::Value::Null
          ))
        })?;
      }
      &instructions::Instruction::SuperiorBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Greater,
            value::Value::Null
          ))
        })?;
      }
      &instructions::Instruction::InferiorEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Equal | value::Ordering::Less,
            value::Value::Null
          ))
        })?;
      }
      &instructions::Instruction::SuperiorEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Equal | value::Ordering::Greater,
            value::Value::Null
          ))
        })?;
      }
      &instructions::Instruction::InBinaryOperator =>
      {
        execute_binary_operator::<value::Value>(stack, |a, b| {
          if b.is_null()
          {
            Ok(value::Value::Null)
          }
          else
          {
            let b_arr: Vec<value::Value> = b.try_into()?;
            Ok(contain_to_value!(
              value::contains(&b_arr, &a),
              value::ContainResult::True
            ))
          }
        })?;
      }
      &instructions::Instruction::NotInBinaryOperator =>
      {
        execute_binary_operator::<value::Value>(stack, |a, b| {
          if b.is_null()
          {
            Ok(value::Value::Null)
          }
          else
          {
            let b_arr: Vec<value::Value> = b.try_into()?;
            Ok(contain_to_value!(
              value::contains(&b_arr, &a),
              value::ContainResult::False
            ))
          }
        })?;
      }
      &instructions::Instruction::AdditionBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a + b)?;
      }
      &instructions::Instruction::SubtractionBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a - b)?;
      }
      &instructions::Instruction::MultiplicationBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a * b)?;
      }
      &instructions::Instruction::DivisionBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a / b)?;
      }
      &instructions::Instruction::ModuloBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a % b)?;
      }
      &instructions::Instruction::ExponentBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a.pow(b))?;
      }
    }
  }
  Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_update_property<TStore: store::Store>(
  store: &TStore,
  tx: &mut TStore::TransactionBox,
  graph_name: &String,
  row: &mut value_table::Row,
  target: value_table::ColId,
  path: &[String],
  instructions: &instructions::Instructions,
  parameters: &crate::value::ValueMap,
  set: bool,
) -> Result<()>
{
  let var = row.get(target)?;
  let mut stack = Stack::default();
  eval_instructions(&mut stack, row, instructions, parameters)?;
  let value: value::Value = stack.try_pop_into()?;
  let value = match value
  {
    value::Value::Node(n) => n.take_properties().into(),
    value::Value::Edge(e) => e.take_properties().into(),
    _ => value,
  };
  let mut piter = path.iter();
  match var
  {
    value::Value::Node(n) =>
    {
      let mut n = n.to_owned();
      if set
      {
        n.properties_mut()
          .set_value(piter.next(), piter, value.remove_null())?;
      }
      else
      {
        n.properties_mut()
          .add_values(piter.next(), piter, value.try_into()?)?;
      }
      store.update_node(tx, graph_name, &n)?;
      row.set(target, n.into())?;
    }
    value::Value::Edge(e) =>
    {
      let mut e = e.to_owned();
      if set
      {
        e.properties_mut()
          .set_value(piter.next(), piter, value.remove_null())?;
      }
      else
      {
        e.properties_mut()
          .add_values(piter.next(), piter, value.try_into()?)?;
      }
      store.update_edge(tx, graph_name, &e)?;
      row.set(target, e.into())?;
    }
    value::Value::Null =>
    {}
    _ => Err(InternalError::ExpectedEdge {
      context: "evaluator/eval_program",
    })?,
  }
  Ok(())
}

fn handle_asc(o: std::cmp::Ordering, asc: bool) -> std::cmp::Ordering
{
  if asc
  {
    o
  }
  else
  {
    match o
    {
      std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
      std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
      std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
    }
  }
}

#[derive(Debug, Default, Clone, Hash, PartialEq)]
struct RowKey(value_table::Row);

impl Eq for RowKey {}

fn create_aggregations_states(
  variables: &Vec<&instructions::RWExpression>,
  parameters: &crate::value::ValueMap,
) -> Result<Vec<HashMap<usize, Box<dyn aggregators::AggregatorState>>>>
{
  variables
    .iter()
    .map(|rw_expr| {
      rw_expr
        .aggregations
        .iter()
        .map(|(name, agg)| {
          let mut stack = Stack::default();

          eval_instructions(
            &mut stack,
            &value_table::Row::default(),
            &agg.init_instructions,
            parameters,
          )?;
          let state = agg.aggregator.create(
            stack
              .into_vec()
              .into_iter()
              .map(|v| v.try_into())
              .collect::<Result<_>>()?,
          )?;

          Ok((name.to_owned(), state))
        })
        .collect::<Result<HashMap<_, _>>>()
    })
    .collect::<Result<Vec<_>>>()
}

fn compute_return_with_table(
  variables: Vec<&instructions::RWExpression>,
  filter: &instructions::Instructions,
  modifiers: &instructions::Modifiers,
  input_table: value_table::ValueTable<usize>,
  parameters: &crate::value::ValueMap,
  variables_sizes: &VariablesSizes,
) -> Result<value_table::ValueTable<usize>>
{
  let mut output_table = value_table::ValueTable::new(variables_sizes.total_size());
  // Compute table
  if variables.iter().any(|v| !v.aggregations.is_empty())
  {
    // 1) For each row, compute non-aggregated columns, based on those columns, select a vector of aggregator states. and update them

    let mut aggregation_table =
      HashMap::<RowKey, Vec<HashMap<usize, Box<dyn aggregators::AggregatorState>>>>::default();

    for row in input_table.into_row_iter()
    {
      // a) compute non-aggregated columns
      let mut row = row.extended(variables_sizes.total_size())?;
      let out_row = variables
        .iter()
        .map(|rw_expr| {
          if rw_expr.aggregations.is_empty()
          {
            assert_eq!(rw_expr.aggregations.len(), 0);
            let mut stack = Stack::default();
            eval_instructions(&mut stack, &row, &rw_expr.instructions, parameters)?;
            let value: value::Value = stack.try_pop_into()?;
            row.set(rw_expr.col_id, value.to_owned())?;
            Ok(value)
          }
          else
          {
            Ok(value::Value::Null)
          }
        })
        .collect::<Result<Row>>()?;
      // b) initialise aggregations
      use std::collections::hash_map::Entry;
      let aggregations_states = match aggregation_table.entry(RowKey(out_row.clone()))
      {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) =>
        {
          let aggregations_states = create_aggregations_states(&variables, parameters)?;
          entry.insert(aggregations_states)
        }
      };
      // c) update aggregations states
      for (rw_expr, aggregation_states) in variables.iter().zip(aggregations_states.iter_mut())
      {
        for (name, agg) in rw_expr.aggregations.iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &row, &agg.argument_instructions, parameters)?;
          let value: value::Value = stack.try_pop_into()?;
          aggregation_states
            .get_mut(name)
            .ok_or(InternalError::MissingAggregationState)?
            .next(value)?;
        }
      }
    }

    // Aggregation always return at least once, unless there is a non-aggregated value
    if aggregation_table.is_empty() && variables.iter().all(|v| !v.aggregations.is_empty())
    {
      let row = Row::new(Default::default(), variables_sizes.total_size());
      let aggregations_states = create_aggregations_states(&variables, parameters)?;
      aggregation_table.insert(RowKey(row), aggregations_states);
    }

    // 2) For each vector of aggregator states, compute the final result

    for (row, aggregations_states) in aggregation_table
    {
      let mut out_row = value_table::Row::new(Default::default(), variables_sizes.total_size());
      for (idx, (rw_expr, aggregation_states)) in variables
        .iter()
        .zip(aggregations_states.into_iter())
        .enumerate()
      {
        if rw_expr.aggregations.is_empty()
        {
          out_row.set(rw_expr.col_id, row.0.get(idx)?.to_owned())?;
        }
        else
        {
          for (name, s) in aggregation_states.into_iter()
          {
            let value = s.finalise()?;
            out_row.set(name, value)?;
          }
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &out_row, &rw_expr.instructions, parameters)?;
          let value: value::Value = stack.try_pop_into()?;
          out_row.set(rw_expr.col_id, value.to_owned())?;
        }
      }
      output_table.add_truncated_row(out_row)?;
    }
  }
  else
  {
    output_table = input_table
      .into_row_iter()
      .map(|row| {
        let mut out_row = row.extended(variables_sizes.total_size())?;
        for rw_expr in variables.iter()
        {
          assert_eq!(rw_expr.aggregations.len(), 0);
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &out_row, &rw_expr.instructions, parameters)?;
          let value: value::Value = stack.try_pop_into()?;
          out_row.set(rw_expr.col_id, value.to_owned())?;
        }
        Ok(out_row)
      })
      .collect::<Result<Result<_>>>()??;
  }
  // Apply filter
  if !filter.is_empty()
  {
    output_table = filter_rows(output_table.into_row_iter(), filter, parameters)?.try_into()?;
  }
  // Apply modifiers
  // Sort the table according to order_by
  if !modifiers.order_by.is_empty()
  {
    let mut table_key = output_table
      .into_row_iter()
      .map(|x| {
        let mut v = Vec::<(value::Value, bool)>::new();
        for info in modifiers.order_by.iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &x, &info.instructions, parameters)?;
          v.push((stack.try_pop_into()?, info.asc));
        }
        Ok((x, v))
      })
      .collect::<Result<Vec<_>>>()?;
    table_key.sort_by(|(_, a), (_, b)| {
      a.iter()
        .zip(b.iter())
        .map(|((a, asc), (b, _))| handle_asc(a.orderability(b), *asc))
        .find(|x| *x != std::cmp::Ordering::Equal)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
    output_table = table_key
      .into_iter()
      .map(|(x, _)| x)
      .collect::<Result<_>>()?;
  }

  // Skip
  if let Some(skip) = &modifiers.skip
  {
    let mut stack = Stack::default();
    eval_instructions(&mut stack, &value_table::Row::default(), skip, parameters)?;
    let q: i64 = stack
      .try_pop_into()
      .map_err(|_| RunTimeError::InvalidArgumentType)?;
    if q >= 0
    {
      output_table.remove_first_rows(q as usize);
    }
    else
    {
      Err(RunTimeError::NegativeIntegerArgument)?
    }
  }

  // Limit
  if let Some(limit) = &modifiers.limit
  {
    let mut stack = Stack::default();
    eval_instructions(&mut stack, &value_table::Row::default(), limit, parameters)?;
    let q: i64 = stack
      .try_pop_into()
      .map_err(|_| RunTimeError::InvalidArgumentType)?;
    if q >= 0
    {
      output_table.truncate(q as usize);
    }
    else
    {
      Err(RunTimeError::NegativeIntegerArgument)?
    }
  }

  output_table
    .into_row_iter()
    .map(|mut row| {
      Ok(Row::new(
        variables
          .iter()
          .map(|rw_expr| row.take(rw_expr.col_id))
          .collect::<Result<_>>()?,
        0,
      ))
    })
    .map(value_table::RowResult)
    .collect()
}

fn filter_rows(
  current_rows: impl Iterator<Item = value_table::Row>,
  filter: &instructions::Instructions,
  parameters: &crate::value::ValueMap,
) -> Result<Vec<value_table::Row>>
{
  current_rows
    .filter_map(|row| {
      let res: Result<bool> = (|| {
        let mut stack = Stack::default();
        eval_instructions(&mut stack, &row, filter, parameters)?;
        stack.try_pop_as_boolean()
      })();
      match res
      {
        Err(x) => Some(Err(x)),
        Ok(v) =>
        {
          if v
          {
            Some(Ok(row))
          }
          else
          {
            None
          }
        }
      }
    })
    .collect()
}

fn is_write_program(program: &super::Program) -> bool
{
  program.iter().any(|b| match b
  {
    Block::UseGraph { .. }
    | Block::Match { .. }
    | Block::Return { .. }
    | Block::Unwind { .. }
    | Block::Call { .. }
    | Block::With { .. } => false,
    Block::CreateGraph { .. }
    | Block::DropGraph { .. }
    | Block::Create { .. }
    | Block::Update { .. }
    | Block::Delete { .. } => true,
  })
}

pub(crate) fn eval_program<TStore: store::Store>(
  store: &TStore,
  program: &super::Program,
  parameters: &crate::value::ValueMap,
) -> crate::Result<query_result::QueryResult>
{
  let mut graph_name: String = "default".into();
  let mut input_table = value_table::ValueTable::new(0);
  input_table.add_full_row(value_table::Row::default())?;
  let mut tx = if is_write_program(program)
  {
    store.begin_write()?
  }
  else
  {
    store.begin_read()?
  };
  let mut stack = Default::default();
  for block in program
  {
    if crate::consts::SHOW_EVALUATOR_STATE
    {
      println!("--- block {:#?}", block);
      println!("input_table: {:#?}", input_table);
    }
    match block
    {
      instructions::Block::CreateGraph { name } =>
      {
        store
          .create_graph(&mut tx, name, false)
          .map_err(|e|
            error::map_error!(e, Error::StoreError(StoreError::DuplicatedGraph { graph_name }) => RunTimeError::DuplicatedGraph {
              graph_name: graph_name.clone(),
            } ))?;
        graph_name = name.to_owned();
      }
      instructions::Block::DropGraph { name, if_exists } =>
      {
        store
          .drop_graph(&mut tx, name, *if_exists)
          .map_err(|e|
            error::map_error!(e, Error::StoreError(StoreError::UnknownGraph { graph_name }) => RunTimeError::UnknownGraph {
              graph_name: graph_name.clone(),
            } ))?;
        graph_name = name.to_owned();
      }
      instructions::Block::UseGraph { name } =>
      {
        graph_name = name.to_owned();
        if !store.graphs_list(&mut tx)?.contains(&graph_name)
        {
          Err(RunTimeError::UnknownGraph {
            graph_name: graph_name.to_owned(),
          })?;
        }
      }
      instructions::Block::Create {
        actions,
        variables_size,
      } =>
      {
        let mut output_table = value_table::ValueTable::new(variables_size.total_size());
        for row in input_table.into_row_iter()
        {
          let mut new_row = row.extended(variables_size.total_size())?;
          for action in actions.iter()
          {
            eval_instructions(&mut stack, &new_row, &action.instructions, parameters)?;
            for (v, var) in stack
              .try_drain_into(action.variables.len())?
              .into_iter()
              .zip(action.variables.iter())
            {
              match v
              {
                crate::value::Value::Node(n) =>
                {
                  store.create_nodes(&mut tx, &graph_name, vec![&n])?;
                  if let Some(var) = var
                  {
                    new_row.set_if_unset(*var, crate::value::Value::Node(n))?;
                  }
                }
                crate::value::Value::Path(p) =>
                {
                  store.create_edges(&mut tx, &graph_name, vec![&p])?;
                  if let Some(var) = var
                  {
                    new_row.set(*var, crate::value::Value::Edge(p.into()))?;
                  }
                }
                _ =>
                {
                  return Err(InternalError::Unimplemented("executor/eval/create").into());
                }
              }
            }
          }
          output_table.add_truncated_row(new_row)?;
        }
        input_table = output_table;
      }
      instructions::Block::Match {
        blocks,
        filter,
        optional,
        variables_size,
      } =>
      {
        let mut output_table = value_table::ValueTable::new(variables_size.persistent_variables);
        for row in input_table.into_row_iter()
        {
          let mut current_rows: Vec<_> = vec![row.clone().extended(variables_size.total_size())?];
          for block in blocks.iter()
          {
            let mut new_rows = Vec::<value_table::Row>::default();
            for row in current_rows
            {
              match block
              {
                instructions::BlockMatch::MatchNode {
                  instructions,
                  variable,
                  filter,
                } =>
                {
                  eval_instructions(&mut stack, &row, instructions, parameters)?;
                  let query: store::SelectNodeQuery = stack.try_pop_into()?;
                  let nodes = store.select_nodes(&mut tx, &graph_name, query)?;

                  for node in nodes.into_iter()
                  {
                    let mut new_row = row.clone();
                    if let Some(variable) = variable
                    {
                      new_row.set_if_unset(*variable, node.to_owned().into())?;
                    }
                    let should_add_row = if filter.is_empty()
                    {
                      true
                    }
                    else
                    {
                      let mut stack = Stack::default();
                      stack.push(true);
                      stack.push(node);
                      eval_instructions(&mut stack, &new_row, filter, parameters)?;
                      stack.try_pop()?; // Get rid of the edge
                      stack.try_pop_into()?
                    };
                    if should_add_row
                    {
                      new_rows.push(new_row);
                    }
                  }
                }
                instructions::BlockMatch::MatchEdge {
                  instructions,
                  left_variable,
                  edge_variable,
                  right_variable,
                  path_variable,
                  filter,
                  directivity,
                } =>
                {
                  eval_instructions(&mut stack, &row, instructions, parameters)?;
                  let query = stack.try_pop_into()?;

                  let edges = store.select_edges(&mut tx, &graph_name, query, *directivity)?;

                  for edge in edges.into_iter()
                  {
                    let mut new_row = row.clone();
                    let (left, right) = if edge.reversed
                    {
                      (edge.path.destination(), edge.path.source())
                    }
                    else
                    {
                      (edge.path.source(), edge.path.destination())
                    };
                    if let Some(left_variable) = left_variable.to_owned()
                    {
                      new_row.set(left_variable, left.to_owned().into())?;
                    }
                    if let Some(right_variable) = right_variable.to_owned()
                    {
                      new_row.set(right_variable, right.to_owned().into())?;
                    }
                    if let Some(edge_variable) = edge_variable.to_owned()
                    {
                      new_row.set(edge_variable, edge.path.to_edge().into())?;
                    }
                    if let Some(path_variable) = path_variable.to_owned()
                    {
                      new_row.set(path_variable, edge.path.to_owned().into())?;
                    }
                    let should_add_row = if filter.is_empty()
                    {
                      true
                    }
                    else
                    {
                      let mut stack = Stack::default();
                      stack.push(true);
                      stack.push(edge.path.into_edge());
                      eval_instructions(&mut stack, &new_row, filter, parameters)?;
                      stack.try_pop()?; // Get rid of the edge
                      stack.try_pop_into()?
                    };
                    if should_add_row
                    {
                      new_rows.push(new_row);
                    }
                  }
                }
              }
            }
            current_rows = new_rows;
          }
          if !filter.is_empty()
          {
            current_rows = filter_rows(current_rows.into_iter(), filter, parameters)?;
          }
          if current_rows.is_empty() && *optional
          {
            output_table.add_truncated_row(row.extended(variables_size.persistent_variables)?)?;
          }
          else
          {
            output_table.add_truncated_rows(current_rows)?;
          }
        }
        input_table = output_table;
      }
      instructions::Block::Return {
        variables,
        filter,
        modifiers,
        variables_size,
      } =>
      {
        let (names, variables): (Vec<_>, Vec<_>) = variables.iter().map(|(s, e)| (s, e)).unzip();
        let output_table = compute_return_with_table(
          variables,
          filter,
          modifiers,
          input_table,
          parameters,
          variables_size,
        )?;
        let headers = names.into_iter().map(|name| name.to_owned()).collect();
        let mut data = Vec::<crate::value::Value>::new();
        for row in output_table.into_row_iter()
        {
          data.extend(row.into_iter());
        }
        tx.close()?;
        return Ok(graphcore::Table::new(headers, data)?.into());
      }
      instructions::Block::With {
        variables,
        filter,
        modifiers,
        variables_size,
      } =>
      {
        input_table = compute_return_with_table(
          variables.iter().collect(),
          filter,
          modifiers,
          input_table,
          parameters,
          variables_size,
        )?;
      }
      instructions::Block::Unwind {
        col_id,
        instructions,
        variables_size,
      } =>
      {
        let mut output_table = value_table::ValueTable::new(variables_size.persistent_variables);
        for row in input_table.into_row_iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &row, instructions, parameters)?;
          let value = stack.try_pop_into()?;
          match value
          {
            value::Value::Array(arr) =>
            {
              for v in arr.into_iter()
              {
                let mut out_row = row.clone().extended(variables_size.total_size())?;
                out_row.set(*col_id, v)?;
                output_table.add_truncated_row(out_row)?;
              }
            }
            value::Value::Null =>
            {}
            _ =>
            {
              let mut out_row = row.extended(variables_size.total_size())?;
              out_row.set(*col_id, value)?;
              output_table.add_truncated_row(out_row)?;
            }
          }
        }
        input_table = output_table;
      }
      instructions::Block::Delete {
        detach,
        instructions,
      } =>
      {
        let mut nodes_keys = Vec::<graph::Key>::new();
        let mut edges_keys = Vec::<graph::Key>::new();
        for row in input_table.row_iter()
        {
          for instructions in instructions.iter()
          {
            let mut stack = Stack::default();
            eval_instructions(&mut stack, &row, instructions, parameters)?;
            let value: value::Value = stack.try_pop_into()?;
            match value
            {
              value::Value::Node(node) => nodes_keys.push(node.unpack().0),
              value::Value::Edge(edge) => edges_keys.push(edge.unpack().0),
              value::Value::Null =>
              {}
              _ => return Err(RunTimeError::InvalidDelete.into()),
            }
          }
        }

        store.delete_edges(
          &mut tx,
          &graph_name,
          store::SelectEdgeQuery::select_keys(edges_keys),
          graph::EdgeDirectivity::Directed,
        )?;
        store.delete_nodes(
          &mut tx,
          &graph_name,
          store::SelectNodeQuery::select_keys(nodes_keys),
          *detach,
        )?;
      }
      instructions::Block::Update {
        updates,
        variables_size,
      } =>
      {
        let mut output_table = value_table::ValueTable::new(variables_size.persistent_variables);
        for row in input_table.into_row_iter()
        {
          let mut out_row = row.extended(variables_size.total_size())?;
          for update in updates.iter()
          {
            match update
            {
              instructions::UpdateOne::SetProperty {
                target,
                path,
                instructions,
              } =>
              {
                eval_update_property(
                  store,
                  &mut tx,
                  &graph_name,
                  &mut out_row,
                  *target,
                  path,
                  instructions,
                  parameters,
                  true,
                )?;
              }
              instructions::UpdateOne::AddProperty {
                target,
                path,
                instructions,
              } =>
              {
                eval_update_property(
                  store,
                  &mut tx,
                  &graph_name,
                  &mut out_row,
                  *target,
                  path,
                  instructions,
                  parameters,
                  false,
                )?;
              }
              instructions::UpdateOne::RemoveProperty { target, path } =>
              {
                let var = out_row.get(*target)?;
                let mut piter = path.iter();
                match var
                {
                  value::Value::Node(n) =>
                  {
                    let mut n = n.to_owned();
                    n.properties_mut().remove_value(piter.next(), piter)?;
                    store.update_node(&mut tx, &graph_name, &n)?;
                    out_row.set(*target, n.into())?;
                  }
                  value::Value::Edge(e) =>
                  {
                    let mut e = e.to_owned();
                    e.properties_mut().remove_value(piter.next(), piter)?;
                    store.update_edge(&mut tx, &graph_name, &e)?;
                    out_row.set(*target, e.into())?;
                  }
                  value::Value::Null =>
                  {}
                  _ => Err(InternalError::ExpectedEdge {
                    context: "evaluator/eval_program",
                  })?,
                }
              }
              instructions::UpdateOne::AddLabels { target, labels }
              | instructions::UpdateOne::RemoveLabels { target, labels } =>
              {
                let add_labels = match update
                {
                  instructions::UpdateOne::AddLabels { .. } => true,
                  instructions::UpdateOne::RemoveLabels { .. } => false,
                  _ => Err(InternalError::Unreachable {
                    context: "evaluator/eval_program/add_remove_labels",
                  })?,
                };

                let var = out_row.get(*target)?;
                match var
                {
                  value::Value::Node(n) =>
                  {
                    let mut n = n.to_owned();
                    if add_labels
                    {
                      n.labels_mut().append(&mut labels.clone());
                    }
                    else
                    {
                      n.labels_edit(|l| l.into_iter().filter(|x| !labels.contains(x)).collect());
                    }
                    store.update_node(&mut tx, &graph_name, &n)?;
                    out_row.set(*target, n.into())?;
                  }
                  value::Value::Edge(e) =>
                  {
                    let mut e = e.to_owned();
                    if add_labels
                    {
                      e.labels_mut().append(&mut labels.clone());
                    }
                    else
                    {
                      e.labels_edit(|l| l.into_iter().filter(|x| !labels.contains(x)).collect());
                    }
                    store.update_edge(&mut tx, &graph_name, &e)?;
                    out_row.set(*target, e.into())?;
                  }
                  value::Value::Null =>
                  {}
                  _ => Err(InternalError::ExpectedEdge {
                    context: "evaluator/eval_program",
                  })?,
                }
              }
            }
          }
          output_table.add_truncated_row(out_row)?;
        }
        input_table = output_table;
      }
      instructions::Block::Call { arguments: _, name } =>
      {
        if name == "gqlite.internal.stats"
        {
          let stats = store.compute_statistics(&mut tx)?;
          let mut res = value::ValueMap::new();
          res.insert("nodes_count".into(), (stats.nodes_count as i64).into());
          res.insert("edges_count".into(), (stats.edges_count as i64).into());
          res.insert(
            "labels_nodes_count".into(),
            (stats.labels_nodes_count as i64).into(),
          );
          res.insert(
            "properties_count".into(),
            (stats.properties_count as i64).into(),
          );
          return Ok(crate::value::Value::Map(res).into());
        }
        else
        {
          return Err(InternalError::Unimplemented("call for any other function").into());
        }
      }
    }
  }
  tx.close()?;
  Ok(crate::QueryResult::Empty)
}

#[cfg(test)]
mod tests
{
  use crate::{
    interpreter::instructions::Instruction::{AndBinaryOperator, OrBinaryOperator},
    prelude::*,
  };

  use super::{execute_boolean_operator, TryPopInto};

  fn test_execute_boolean_operator_(
    instruction: &interpreter::instructions::Instruction,
    a: impl Into<super::Value>,
    b: impl Into<super::Value>,
    g: impl Into<value::Value>,
  )
  {
    let mut stack = super::Stack::default();
    stack.push(b.into());
    stack.push(a.into());
    execute_boolean_operator(&mut stack, instruction).unwrap();
    let r: value::Value = stack.try_pop_into().unwrap();
    assert_eq!(r, g.into());
  }

  #[test]
  fn test_execute_boolean_operator()
  {
    test_execute_boolean_operator_(&AndBinaryOperator, true, true, true);
    test_execute_boolean_operator_(&AndBinaryOperator, true, false, false);
    test_execute_boolean_operator_(&AndBinaryOperator, false, true, false);
    test_execute_boolean_operator_(&AndBinaryOperator, false, value::Value::Null, false);
    test_execute_boolean_operator_(
      &OrBinaryOperator,
      value::Value::Null,
      false,
      value::Value::Null,
    );
  }
}
