use std::collections::HashMap;

use crate::prelude::*;
use interpreter::instructions;

#[derive(Debug, Clone)]
enum Value
{
  GraphValue(graph::Value),
  NodeQuery(store::SelectNodeQuery),
  EdgeQuery(store::SelectEdgeQuery),
}

impl Value
{
  fn is_null(&self) -> bool
  {
    match self
    {
      Value::GraphValue(gv) => gv.is_null(),
      _ => false,
    }
  }
}

impl<T> From<T> for Value
where
  T: Into<graph::Value>,
{
  fn from(value: T) -> Self
  {
    Self::GraphValue(value.into())
  }
}

// impl<T> TryInto<T> for Value
// where
//   T: TryFrom<graph::Value, Error = crate::Error>,
// {
//   type Error = crate::Error;
//   fn try_into(self) -> std::result::Result<T, Self::Error>
//   {
//     match self
//     {
//       Self::GraphValue(gv) => gv.try_into(),
//       _ => Err(
//         InternalError::ExpectedGraphValue {
//           context: "try_into",
//         }
//         .into(),
//       ),
//     }
//   }
// }

macro_rules! try_into_gv_impl {
  ($vn:ty) => {
    impl TryInto<$vn> for Value
    {
      type Error = crate::Error;
      fn try_into(self) -> std::result::Result<$vn, Self::Error>
      {
        let a: graph::Value = self.try_into()?;
        a.try_into()
      }
    }
    impl TryPopInto<$vn> for Stack
    {
      fn try_pop_into(&mut self) -> Result<$vn>
      {
        self.try_pop()?.try_into()
      }
      fn try_drain_into(&mut self, n: usize) -> Result<Vec<$vn>>
      {
        self.try_drain(n)?.map(|x| x.try_into()).collect()
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
      type Error = crate::Error;
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
      }
      fn try_drain_into(&mut self, n: usize) -> Result<Vec<$type>>
      {
        self.try_drain(n)?.map(|x| x.try_into()).collect()
      }
    }
  };
}

try_into_impl! {GraphValue, graph::Value, ExpectedGraphValue}
try_into_impl! {NodeQuery, store::SelectNodeQuery, ExpectedNodeQuery}
try_into_impl! {EdgeQuery, store::SelectEdgeQuery, ExpectedEdgeQuery}

impl From<store::SelectNodeQuery> for Value
{
  fn from(value: store::SelectNodeQuery) -> Self
  {
    Self::NodeQuery(value.into())
  }
}

impl From<store::SelectEdgeQuery> for Value
{
  fn from(value: store::SelectEdgeQuery) -> Self
  {
    Self::EdgeQuery(value.into())
  }
}

#[allow(unused_macros)]
macro_rules! check_for_null {
  ($a: expr) => {
    if $a.is_null() {
      return Ok(crate::graph::Value::Invalid)
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
      value::Ordering::ComparedNull => graph::Value::Invalid,
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
      value::ContainResult::ComparedNull => graph::Value::Invalid,
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
  fn push(&mut self, value: Value)
  {
    self.stack.push(value);
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
  fn to_vec(self) -> Vec<Value>
  {
    self.stack
  }
  fn try_pop_as_boolean(&mut self) -> Result<bool>
  {
    let v = self.try_pop()?;
    match v
    {
      Value::GraphValue(graph::Value::Invalid)
      | Value::GraphValue(graph::Value::Boolean(false)) => Ok(false),
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
  T: TryFrom<Value, Error = Error>,
{
  fn try_pop_into(&mut self) -> Result<T>
  {
    self.try_pop()?.try_into()
  }
  fn try_drain_into(&mut self, n: usize) -> Result<Vec<T>>
  {
    self.try_drain(n)?.map(|x| x.try_into()).collect()
  }
}

fn execute_boolean_operator(
  stack: &mut Stack,
  instruction: &instructions::Instruction,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  let a: graph::Value = a.try_into()?;
  let b: graph::Value = b.try_into()?;
  match instruction
  {
    &instructions::Instruction::AndBinaryOperator =>
    {
      if a.is_null()
      {
        if b.is_null() || <graph::Value as TryInto<bool>>::try_into(b)? == true
        {
          stack.push(graph::Value::Invalid.into());
        }
        else
        {
          stack.push(false.into());
        }
      }
      else
      {
        let a: bool = a.try_into()?;
        if a
        {
          if b.is_null()
          {
            stack.push(graph::Value::Invalid.into());
          }
          else
          {
            stack.push(b.into());
          }
        }
        else
        {
          stack.push(false.into());
        }
      }
    }
    &instructions::Instruction::OrBinaryOperator =>
    {
      if a.is_null()
      {
        if b.is_null() || <graph::Value as TryInto<bool>>::try_into(b)? == false
        {
          stack.push(graph::Value::Invalid.into());
        }
        else
        {
          stack.push(true.into());
        }
      }
      else
      {
        let a: bool = a.try_into()?;
        if a
        {
          stack.push(true.into());
        }
        else
        {
          if b.is_null()
          {
            stack.push(graph::Value::Invalid.into());
          }
          else
          {
            stack.push(b.into());
          }
        }
      }
    }
    &instructions::Instruction::XorBinaryOperator =>
    {
      if a.is_null() || b.is_null()
      {
        stack.push(graph::Value::Invalid.into());
      }
      else
      {
        let a: bool = a.try_into()?;
        let b: bool = b.try_into()?;
        stack.push((a ^ b).into());
      }
    }
    _ => Err(InternalError::Unreachable {
      context: "evaluator/execute_boolean_operator",
    })?,
  }

  Ok(())
}

fn execute_binary_operator<T: Into<crate::graph::Value>>(
  stack: &mut Stack,
  operand: impl FnOnce(crate::graph::Value, crate::graph::Value) -> Result<T>,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  stack.push(operand(a.try_into()?, b.try_into()?)?.into().into());
  Ok(())
}

fn eval_instructions(
  stack: &mut Stack,
  row: &value_table::Row,
  instructions: &instructions::Instructions,
  parameters: &crate::graph::ValueObject,
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
        let props: graph::Value = stack.try_pop_into()?;
        let dst: graph::Node = stack.try_pop_into()?;
        let src: graph::Node = stack.try_pop_into()?;
        stack.push(
          crate::graph::Edge {
            key: graph::Key::default(),
            source: src,
            destination: dst,
            labels: labels.to_owned(),
            properties: props.to_object_safe(),
          }
          .into(),
        );
      }
      instructions::Instruction::CreateNodeLiteral { labels } =>
      {
        let props: graph::Value = stack.try_pop_into()?;
        stack.push(
          crate::graph::Node {
            key: crate::graph::Key::default(),
            labels: labels.clone(),
            properties: props.to_object_safe(),
          }
          .into(),
        );
      }
      instructions::Instruction::CreateEdgeQuery { labels } =>
      {
        let props: graph::Value = stack.try_pop_into()?;
        let dst: store::SelectNodeQuery = stack.try_pop_into()?;
        let src: store::SelectNodeQuery = stack.try_pop_into()?;
        match props
        {
          graph::Value::Edge(ed) =>
          {
            stack.push(
              store::SelectEdgeQuery::select_source_destination_keys(src, [ed.key], dst).into(),
            );
          }
          graph::Value::Object(ob) =>
          {
            stack.push(
              store::SelectEdgeQuery::select_source_destination_labels_properties(
                src,
                labels.clone(),
                ob,
                dst,
              )
              .into(),
            );
          }
          graph::Value::Invalid =>
          {
            stack.push(store::SelectEdgeQuery::select_none().into());
          }
          _ => Err(InternalError::InvalidValueCast {
            value: props,
            typename: "Edge properties",
          })?,
        }
      }
      instructions::Instruction::CreateNodeQuery { labels } =>
      {
        let props: graph::Value = stack.try_pop_into()?;
        match props
        {
          graph::Value::Node(no) =>
          {
            stack.push(store::SelectNodeQuery::select_keys([no.key]).into());
          }
          graph::Value::Object(ob) =>
          {
            stack.push(store::SelectNodeQuery::select_labels_properties(labels.clone(), ob).into());
          }
          graph::Value::Invalid =>
          {
            stack.push(store::SelectNodeQuery::select_none().into());
          }
          _ => Err(InternalError::InvalidValueCast {
            value: props,
            typename: "Node properties",
          })?,
        }
      }
      instructions::Instruction::FunctionCall {
        function,
        arguments_count,
      } =>
      {
        let args: Vec<graph::Value> = stack.try_drain_into(*arguments_count)?;
        if args.len() != *arguments_count
        {
          Err(InternalError::MissingStackValue {
            context: "eval_instructions/FunctionCall",
          })?;
        }
        stack.push(function.call(args)?.into());
      }
      instructions::Instruction::Push { value } =>
      {
        stack.push(value.clone().into());
      }
      instructions::Instruction::GetVariable { name } =>
      {
        if let Some(value) = row.get(name)
        {
          stack.push(value.to_owned().into());
        }
        else
        {
          return Err(
            crate::error::RunTimeError::UndefinedVariable {
              name: name.to_owned(),
            }
            .into(),
          );
        }
      }
      instructions::Instruction::GetParameter { name } =>
      {
        if let Some(value) = parameters.get(name)
        {
          stack.push(value.to_owned().into());
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
        stack.push(graph::Value::Array(m).into());
      }
      instructions::Instruction::CreateMap { keys } =>
      {
        let mut m = crate::graph::ValueObject::new();
        for k in keys.iter().rev()
        {
          m.insert(k.to_owned(), stack.try_pop_into()?);
        }
        stack.push(graph::Value::Object(m).into());
      }
      instructions::Instruction::MemberAccess { path } =>
      {
        let v: graph::Value = stack.try_pop_into()?;
        stack.push(v.access(path.iter()).into());
      }
      instructions::Instruction::IndexAccess =>
      {
        let idx: i64 = stack.try_pop_into()?;
        let v: graph::Value = stack.try_pop_into()?;
        let v: Vec<graph::Value> = v.try_into()?;
        stack.push(
          v.get(idx as usize)
            .ok_or(RunTimeError::OutOfBound)?
            .to_owned()
            .into(),
        );
      }
      instructions::Instruction::RangeAccess { start, end } =>
      {
        let end: Option<graph::Value> = if *end
        {
          Some(stack.try_pop_into()?)
        }
        else
        {
          None
        };
        let start: Option<graph::Value> = if *start
        {
          Some(stack.try_pop_into()?)
        }
        else
        {
          None
        };
        // Get the array out of the stack
        let v: graph::Value = stack.try_pop_into()?;
        // if either end or start are null, return null
        if end.as_ref().map_or(false, |e| e.is_null())
          || start.as_ref().map_or(false, |s| s.is_null())
        {
          stack.push(graph::Value::Invalid.into());
        }
        else
        {
          let mut start: Option<i64> = start.map(|x| x.try_into()).transpose()?;
          let mut end: Option<i64> = end.map(|x| x.try_into()).transpose()?;
          let v: Vec<graph::Value> = v.try_into()?;
          // Compute range length
          let length = match (start, end)
          {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
          };
          if length.map_or(false, |l| l >= v.len() as i64)
          {
            stack.push(v.into());
          }
          else
          {
            // If start is negative, it should be made into a positive number
            while start.map_or(false, |x| x < 0)
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
                  Vec::<graph::Value>::default().into()
                }
                else
                {
                  v[start as usize..end as usize].to_owned().into()
                }
              }
              (Some(start), None) => v[start as usize..].to_owned().into(),
              (None, Some(end)) => v[..end as usize].to_owned().into(),
              (None, None) => v.to_owned().into(),
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
      &instructions::Instruction::Drop =>
      {
        stack.try_pop()?;
      }
      &instructions::Instruction::AndBinaryOperator
      | &instructions::Instruction::OrBinaryOperator
      | &instructions::Instruction::XorBinaryOperator =>
      {
        execute_boolean_operator(stack, instruction)?;
      }
      &instructions::Instruction::NotUnaryOperator =>
      {
        let a: graph::Value = stack.try_pop_into()?;
        match a
        {
          graph::Value::Invalid => stack.push(graph::Value::Invalid.into()),
          graph::Value::Boolean(b) => stack.push((!b).into()),
          _ => Err(RunTimeError::InvalidArgumentType)?,
        }
      }
      &instructions::Instruction::NegationUnaryOperator =>
      {
        let a: crate::graph::Value = stack.try_pop_into()?;
        stack.push((-a)?.into());
      }
      &instructions::Instruction::IsNullUnaryOperator =>
      {
        let a: crate::graph::Value = stack.try_pop_into()?;
        stack.push((a.is_null()).into());
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
            graph::Value::Invalid
          ))
        })?;
      }
      &instructions::Instruction::SuperiorBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Greater,
            graph::Value::Invalid
          ))
        })?;
      }
      &instructions::Instruction::InferiorEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Equal | value::Ordering::Less,
            graph::Value::Invalid
          ))
        })?;
      }
      &instructions::Instruction::SuperiorEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| {
          Ok(ordering_to_value!(
            a.compare(&b),
            value::Ordering::Equal | value::Ordering::Greater,
            graph::Value::Invalid
          ))
        })?;
      }
      &instructions::Instruction::InBinaryOperator =>
      {
        execute_binary_operator::<graph::Value>(stack, |a, b| {
          if b.is_null()
          {
            Ok(graph::Value::Invalid.into())
          }
          else
          {
            let b_arr: Vec<graph::Value> = b.try_into()?;
            Ok(contain_to_value!(
              value::contains(&b_arr, &a),
              value::ContainResult::True
            ))
          }
        })?;
      }
      &instructions::Instruction::NotInBinaryOperator =>
      {
        execute_binary_operator::<graph::Value>(stack, |a, b| {
          if b.is_null()
          {
            Ok(graph::Value::Invalid.into())
          }
          else
          {
            let b_arr: Vec<graph::Value> = b.try_into()?;
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
      &instructions::Instruction::SubstractionBinaryOperator =>
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
    }
  }
  Ok(())
}

trait HashMapExt
{
  fn noreplace_insert(&mut self, k: &String, v: graph::Value);
  fn insert_none(&mut self, k: &Option<String>);
}

impl HashMapExt for HashMap<String, graph::Value>
{
  fn noreplace_insert(&mut self, k: &String, v: graph::Value)
  {
    if !self.contains_key(k)
    {
      self.insert(k.to_owned(), v);
    }
  }
  fn insert_none(&mut self, k: &Option<String>)
  {
    if let Some(k) = k
    {
      let _ = self.noreplace_insert(k, graph::Value::Invalid.into());
    }
  }
}

pub(crate) fn eval_update_property<TStore: store::Store>(
  store: &TStore,
  mut tx: &mut TStore::Transaction,
  graph_name: &String,
  row: &mut value_table::Row,
  target: &String,
  path: &Vec<String>,
  instructions: &instructions::Instructions,
  parameters: &crate::graph::ValueObject,
  set: bool,
) -> Result<()>
{
  let var = row
    .get(target)
    .ok_or_else(|| crate::error::RunTimeError::UndefinedVariable {
      name: target.to_owned(),
    })?;
  let mut stack = Stack::default();
  eval_instructions(&mut stack, row, &instructions, &parameters)?;
  let value: graph::Value = stack.try_pop_into()?;
  let value = match value
  {
    graph::Value::Node(n) => n.properties.into(),
    graph::Value::Edge(e) => e.properties.into(),
    _ => value,
  };
  use crate::graph::ValueObjectExtension;
  let mut piter = path.iter();
  match var
  {
    graph::Value::Node(n) =>
    {
      let mut n = n.to_owned();
      if set
      {
        n.properties
          .set_value(piter.next(), piter, value.remove_null())?;
      }
      else
      {
        n.properties
          .add_values(piter.next(), piter, value.try_into()?)?;
      }
      store.update_node(&mut tx, &graph_name, &n)?;
      row.insert(target.to_owned(), n.into());
    }
    graph::Value::Edge(e) =>
    {
      let mut e = e.to_owned();
      if set
      {
        e.properties
          .set_value(piter.next(), piter, value.remove_null())?;
      }
      else
      {
        e.properties
          .add_values(piter.next(), piter, value.try_into()?)?;
      }
      store.update_edge(&mut tx, &graph_name, &e)?;
      row.insert(target.to_owned(), e.into());
    }
    graph::Value::Invalid =>
    {}
    _ => Err(InternalError::ExpectedEdge {
      context: "evaluator/eval_program",
    })?,
  }
  Ok(())
}

struct OrderByKey(Vec<(graph::Value, bool)>);

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

fn compute_order_by(
  a: &Vec<(graph::Value, bool)>,
  b: &Vec<(graph::Value, bool)>,
) -> std::cmp::Ordering
{
  a.iter()
    .zip(b.iter())
    .map(|(a, b)| handle_asc(a.0.orderability(&b.0), a.1))
    .find(|p| *p != std::cmp::Ordering::Equal)
    .unwrap_or(std::cmp::Ordering::Equal)
}

fn compute_return_with_table(
  variables: &Vec<instructions::RWExpression>,
  filter: &instructions::Instructions,
  modifiers: &instructions::Modifiers,
  input_table: value_table::ValueTable,
  parameters: &crate::graph::ValueObject,
) -> Result<value_table::ValueTable>
{
  let mut output_table = value_table::ValueTable::new();
  // Compute table
  if variables.iter().any(|v| v.aggregations.len() > 0)
  {
    // Initialise aggregation states
    let mut aggregations_states: Vec<HashMap<String, Box<dyn aggregators::AggregatorState>>> =
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
                &Default::default(),
                &agg.init_instructions,
                parameters,
              )?;
              let state = agg.aggregator.create(
                stack
                  .to_vec()
                  .into_iter()
                  .map(|v| v.try_into())
                  .collect::<Result<_>>()?,
              )?;

              Ok((name.to_owned(), state))
            })
            .collect::<Result<HashMap<_, _>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    // Compute aggregations
    for row in input_table.iter()
    {
      for (rw_expr, aggregation_states) in variables.iter().zip(aggregations_states.iter_mut())
      {
        for (name, agg) in rw_expr.aggregations.iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, row, &agg.argument_instructions, parameters)?;
          let value: graph::Value = stack.try_pop_into()?;
          aggregation_states
            .get_mut(name)
            .ok_or(InternalError::MissingAggregationState)?
            .next(value)?;
        }
      }
    }
    // Export the end result
    let mut out_row = value_table::Row::new();
    for (rw_expr, aggregation_states) in variables.iter().zip(aggregations_states.into_iter())
    {
      let mut in_row = input_table
        .first_row()
        .map_or_else(|| Default::default(), |m| m.to_owned());
      for (name, s) in aggregation_states.into_iter()
      {
        in_row.insert(name, s.finalise()?);
      }
      let mut stack = Stack::default();
      eval_instructions(&mut stack, &in_row, &rw_expr.instructions, &parameters)?;
      let value: graph::Value = stack.try_pop_into()?;
      out_row.insert(rw_expr.name.to_owned(), value.to_owned());
    }
    output_table.add_row(out_row);
  }
  else
  {
    output_table = input_table
      .into_iter()
      .map(|mut row| {
        for rw_expr in variables.iter()
        {
          assert_eq!(rw_expr.aggregations.len(), 0);
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &row, &rw_expr.instructions, &parameters)?;
          let value: graph::Value = stack.try_pop_into()?;
          row.insert(rw_expr.name.to_owned(), value.to_owned());
        }
        Ok(row)
      })
      .collect::<Result<_>>()?;
  }
  // Apply filter
  if !filter.is_empty()
  {
    output_table = filter_rows(output_table.into_iter(), &filter, &parameters)?.into();
  }
  // Apply modifiers
  // Sort the table according to order_by
  if !modifiers.order_by.is_empty()
  {
    let mut table_key = output_table
      .into_iter()
      .map(|x| {
        let mut v = Vec::<(graph::Value, bool)>::new();
        for info in modifiers.order_by.iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, &x, &info.instructions, &parameters)?;
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
    output_table = table_key.into_iter().map(|(x, _)| x).collect();
  }

  // Skip
  if let Some(skip) = &modifiers.skip
  {
    let mut stack = Stack::default();
    eval_instructions(&mut stack, &Default::default(), &skip, &parameters)?;
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
    eval_instructions(&mut stack, &Default::default(), &limit, &parameters)?;
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

  // Filter output_table
  let variables_names = variables.iter().map(|x| &x.name).collect::<Vec<_>>();
  Ok(
    output_table
      .into_iter()
      .map(|row| {
        row
          .into_iter()
          .filter(|(x, _)| variables_names.contains(&x))
          .collect()
      })
      .collect(),
  )
}

fn filter_rows(
  current_rows: impl IntoIterator<Item = HashMap<String, graph::Value>>,
  filter: &instructions::Instructions,
  parameters: &crate::graph::ValueObject,
) -> Result<Vec<HashMap<String, graph::Value>>>
{
  current_rows
    .into_iter()
    .filter_map(|row| {
      let res: Result<bool> = (|| {
        let mut stack = Stack::default();
        eval_instructions(&mut stack, &row, &filter, &parameters)?;
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

///
pub(crate) fn eval_program<TStore: store::Store>(
  store: &TStore,
  program: super::Program,
  parameters: crate::graph::ValueObject,
) -> crate::Result<crate::graph::Value>
{
  let graph_name: String = "default".into();
  let mut input_table = value_table::ValueTable::new();
  input_table.add_row(value_table::Row::new());
  let mut tx = store.begin()?;
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
      instructions::Block::Create { actions } =>
      {
        let mut output_table = value_table::ValueTable::new();
        for row in input_table.iter()
        {
          let mut new_row = row.clone();
          for action in actions.iter()
          {
            eval_instructions(&mut stack, &new_row, &action.instructions, &parameters)?;
            for (v, var) in stack
              .try_drain_into(action.variables.len())?
              .into_iter()
              .zip(action.variables.iter())
            {
              match v
              {
                crate::graph::Value::Node(n) =>
                {
                  store.create_nodes(&mut tx, &graph_name, vec![n.to_owned()].iter())?;
                  if let Some(var) = var
                  {
                    let _ = new_row.noreplace_insert(&var, crate::graph::Value::Node(n));
                  }
                }
                crate::graph::Value::Edge(e) =>
                {
                  store.create_edges(&mut tx, &graph_name, vec![e.to_owned()].iter())?;
                  if let Some(var) = var
                  {
                    new_row.insert(var.to_owned(), crate::graph::Value::Edge(e));
                  }
                }
                _ =>
                {
                  return Err(Error::Unimplemented("executor/eval/create"));
                }
              }
            }
          }
          output_table.add_row(new_row);
        }
        input_table = output_table;
      }
      instructions::Block::BlockMatch {
        blocks,
        filter,
        optional,
      } =>
      {
        let mut output_table = value_table::ValueTable::new();
        for row in input_table.into_iter()
        {
          let mut current_rows: Vec<HashMap<String, graph::Value>> = vec![row.clone()];
          for block in blocks.iter()
          {
            let mut new_rows = Vec::<HashMap<String, graph::Value>>::default();
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
                  eval_instructions(&mut stack, &row, &instructions, &parameters)?;
                  let query: store::SelectNodeQuery = stack.try_pop_into()?;
                  let nodes = store.select_nodes(&mut tx, &graph_name, query)?;

                  for node in nodes.iter()
                  {
                    let mut new_row = row.clone();
                    match &variable
                    {
                      Some(variable) =>
                      {
                        new_row.noreplace_insert(&variable, node.to_owned().into());
                      }
                      None =>
                      {}
                    }
                    let should_add_row = if filter.is_empty()
                    {
                      true
                    }
                    else
                    {
                      let mut stack = Stack::default();
                      stack.push(true.into());
                      stack.push(node.to_owned().into());
                      eval_instructions(&mut stack, &new_row, &filter, &parameters)?;
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
                  eval_instructions(&mut stack, &row, &instructions, &parameters)?;
                  let query = stack.try_pop_into()?;

                  let edges = store.select_edges(&mut tx, &graph_name, query, *directivity)?;

                  for edge in edges.iter()
                  {
                    let mut new_row = row.clone();
                    if let Some(left_variable) = left_variable.to_owned()
                    {
                      new_row.insert(
                        left_variable,
                        if edge.reversed
                        {
                          edge.edge.destination.to_owned()
                        }
                        else
                        {
                          edge.edge.source.to_owned()
                        }
                        .into(),
                      );
                    }
                    if let Some(right_variable) = right_variable.to_owned()
                    {
                      new_row.insert(
                        right_variable,
                        if edge.reversed
                        {
                          edge.edge.source.to_owned()
                        }
                        else
                        {
                          edge.edge.destination.to_owned()
                        }
                        .into(),
                      );
                    }
                    if let Some(edge_variable) = edge_variable.to_owned()
                    {
                      new_row.insert(edge_variable, edge.edge.to_owned().into());
                    }
                    if let Some(path_variable) = path_variable.to_owned()
                    {
                      new_row.insert(
                        path_variable,
                        crate::graph::Value::Path(edge.edge.to_owned().into()),
                      );
                    }
                    let should_add_row = if filter.is_empty()
                    {
                      true
                    }
                    else
                    {
                      let mut stack = Stack::default();
                      stack.push(true.into());
                      stack.push(edge.edge.to_owned().into());
                      eval_instructions(&mut stack, &new_row, &filter, &parameters)?;
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
            current_rows = filter_rows(current_rows, &filter, &parameters)?;
          }
          if current_rows.is_empty() && optional
          {
            let mut new_row = row;
            for block in blocks.iter()
            {
              match block
              {
                instructions::BlockMatch::MatchNode { variable, .. } =>
                {
                  new_row.insert_none(&variable);
                }
                instructions::BlockMatch::MatchEdge {
                  left_variable,
                  edge_variable,
                  right_variable,
                  path_variable,
                  ..
                } =>
                {
                  new_row.insert_none(left_variable);
                  new_row.insert_none(edge_variable);
                  new_row.insert_none(right_variable);
                  new_row.insert_none(path_variable);
                }
              }
            }
            output_table.add_row(new_row);
          }
          else
          {
            output_table.add_rows(&mut current_rows);
          }
        }
        input_table = output_table;
      }
      instructions::Block::Return {
        variables,
        filter,
        modifiers,
      } =>
      {
        let output_table =
          compute_return_with_table(&variables, &filter, &modifiers, input_table, &parameters)?;
        let mut r = Vec::<crate::graph::Value>::new();
        r.push(crate::graph::Value::Array(
          variables
            .iter()
            .map(|rw_expr| crate::graph::Value::String(rw_expr.name.to_owned()))
            .collect(),
        ));
        for row in output_table.iter()
        {
          r.push(crate::graph::Value::Array(
            variables
              .iter()
              .map(|rw_expr| match row.get(&rw_expr.name)
              {
                Some(v) => v.to_owned(),
                None => crate::graph::Value::Invalid,
              })
              .collect(),
          ));
        }
        store.commit(tx)?;
        return Ok(crate::graph::Value::Array(r));
      }
      instructions::Block::With {
        variables,
        filter,
        modifiers,
      } =>
      {
        input_table =
          compute_return_with_table(&variables, &filter, &modifiers, input_table, &parameters)?;
      }
      instructions::Block::Unwind { name, instructions } =>
      {
        let mut output_table = value_table::ValueTable::new();
        for row in input_table.iter()
        {
          let mut stack = Stack::default();
          eval_instructions(&mut stack, row, &instructions, &parameters)?;
          let value = stack.try_pop_into()?;
          match value
          {
            graph::Value::Array(arr) =>
            {
              for v in arr.into_iter()
              {
                let mut out_row = row.clone();
                out_row.insert(name.to_owned(), v);
                output_table.add_row(out_row);
              }
            }
            graph::Value::Invalid =>
            {}
            _ =>
            {
              let mut out_row = row.clone();
              out_row.insert(name.to_owned(), value);
              output_table.add_row(out_row);
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
        for row in input_table.iter()
        {
          for instructions in instructions.iter()
          {
            let mut stack = Stack::default();
            eval_instructions(&mut stack, row, &instructions, &parameters)?;
            let value: graph::Value = stack.try_pop_into()?;
            match value
            {
              graph::Value::Node(node) => nodes_keys.push(node.key),
              graph::Value::Edge(edge) => edges_keys.push(edge.key),
              graph::Value::Invalid =>
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
          detach,
        )?;
      }
      instructions::Block::Update { updates } =>
      {
        let mut output_table = value_table::ValueTable::new();
        for row in input_table.iter()
        {
          let mut out_row = row.clone();
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
                  target,
                  path,
                  instructions,
                  &parameters,
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
                  target,
                  path,
                  instructions,
                  &parameters,
                  false,
                )?;
              }
              instructions::UpdateOne::RemoveProperty { target, path } =>
              {
                let var = out_row.get(target).ok_or_else(|| {
                  crate::error::RunTimeError::UndefinedVariable {
                    name: target.to_owned(),
                  }
                })?;
                use crate::graph::ValueObjectExtension;
                let mut piter = path.iter();
                match var
                {
                  graph::Value::Node(n) =>
                  {
                    let mut n = n.to_owned();
                    n.properties.remove_value(piter.next(), piter)?;
                    store.update_node(&mut tx, &graph_name, &n)?;
                    out_row.insert(target.to_owned(), n.into());
                  }
                  graph::Value::Edge(e) =>
                  {
                    let mut e = e.to_owned();
                    e.properties.remove_value(piter.next(), piter)?;
                    store.update_edge(&mut tx, &graph_name, &e)?;
                    out_row.insert(target.to_owned(), e.into());
                  }
                  graph::Value::Invalid =>
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

                let var = out_row.get(target).ok_or_else(|| {
                  crate::error::RunTimeError::UndefinedVariable {
                    name: target.to_owned(),
                  }
                })?;
                match var
                {
                  graph::Value::Node(n) =>
                  {
                    let mut n = n.to_owned();
                    if add_labels
                    {
                      n.labels.append(&mut labels.clone());
                    }
                    else
                    {
                      n.labels = n
                        .labels
                        .into_iter()
                        .filter(|x| !labels.contains(x))
                        .collect();
                    }
                    store.update_node(&mut tx, &graph_name, &n)?;
                    out_row.insert(target.to_owned(), n.into());
                  }
                  graph::Value::Edge(e) =>
                  {
                    let mut e = e.to_owned();
                    if add_labels
                    {
                      e.labels.append(&mut labels.clone());
                    }
                    else
                    {
                      e.labels = e
                        .labels
                        .into_iter()
                        .filter(|x| !labels.contains(x))
                        .collect();
                    }
                    store.update_edge(&mut tx, &graph_name, &e)?;
                    out_row.insert(target.to_owned(), e.into());
                  }
                  graph::Value::Invalid =>
                  {}
                  _ => Err(InternalError::ExpectedEdge {
                    context: "evaluator/eval_program",
                  })?,
                }
              }
            }
          }
          output_table.add_row(out_row);
        }
        input_table = output_table;
      }
      instructions::Block::Call { arguments: _, name } =>
      {
        if name == "gqlite.internal.stats"
        {
          let stats = store.compute_statistics(&mut tx)?;
          let mut res = graph::ValueObject::new();
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
          return Ok(crate::graph::Value::Object(res));
        }
        else
        {
          return Err(Error::Unimplemented("call for any other function"));
        }
      }
    }
  }
  store.commit(tx)?;
  Ok(crate::graph::Value::Invalid)
}

#[cfg(test)]
mod tests
{
  use crate::{
    graph,
    interpreter::{
      self,
      instructions::Instruction::{AndBinaryOperator, OrBinaryOperator},
    },
  };

  use super::{execute_boolean_operator, TryPopInto};

  fn test_execute_boolean_operator_(
    instruction: &interpreter::instructions::Instruction,
    a: impl Into<super::Value>,
    b: impl Into<super::Value>,
    g: impl Into<graph::Value>,
  )
  {
    let mut stack = super::Stack::default();
    stack.push(b.into());
    stack.push(a.into());
    execute_boolean_operator(&mut stack, instruction).unwrap();
    let r: graph::Value = stack.try_pop_into().unwrap();
    assert_eq!(r, g.into());
  }

  #[test]
  fn test_execute_boolean_operator()
  {
    test_execute_boolean_operator_(&AndBinaryOperator, true, true, true);
    test_execute_boolean_operator_(&AndBinaryOperator, true, false, false);
    test_execute_boolean_operator_(&AndBinaryOperator, false, true, false);
    test_execute_boolean_operator_(&AndBinaryOperator, false, graph::Value::Invalid, false);
    test_execute_boolean_operator_(&OrBinaryOperator, graph::Value::Invalid, false, false);
  }
}
