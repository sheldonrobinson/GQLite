use std::{collections::HashMap, default, hash::Hash};

use crate::{
  error::{InternalError, RunTimeError},
  graph,
  interpreter::instructions::BlockMatch,
  store::{self, SelectEdgeQuery},
  Error, Result,
};

use super::instructions;

#[derive(Debug, Clone)]
enum Value
{
  GraphValue(graph::Value),
  NodeQuery(store::SelectNodeQuery),
  EdgeQuery(store::SelectEdgeQuery),
}

impl Value {}

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
  operand: impl FnOnce(bool, bool) -> bool,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  let a: graph::Value = a.try_into()?;
  let b: graph::Value = b.try_into()?;
  stack.push(operand(a.try_into()?, b.try_into()?).into());
  Ok(())
}

fn execute_binary_operator(
  stack: &mut Stack,
  operand: impl FnOnce(crate::graph::Value, crate::graph::Value) -> bool,
) -> Result<()>
{
  let a = stack.try_pop()?;
  let b = stack.try_pop()?;
  stack.push(operand(a.try_into()?, b.try_into()?).into());
  Ok(())
}
fn eval_instructions(
  stack: &mut Stack,
  row: &crate::value_table::Row,
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
              SelectEdgeQuery::select_source_destination_keys(src, [ed.key].into(), dst).into(),
            );
          }
          graph::Value::Object(ob) =>
          {
            stack.push(
              SelectEdgeQuery::select_source_destination_labels_properties(
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
            stack.push(SelectEdgeQuery::select_none().into());
          }
          _ => Err(InternalError::InvalidValueCast)?,
        }
      }
      instructions::Instruction::CreateNodeQuery { labels } =>
      {
        let props: graph::Value = stack.try_pop_into()?;
        match props
        {
          graph::Value::Node(no) =>
          {
            stack.push(crate::store::SelectNodeQuery::select_keys([no.key].into()).into());
          }
          graph::Value::Object(ob) =>
          {
            stack.push(
              crate::store::SelectNodeQuery::select_labels_properties(labels.clone(), ob).into(),
            );
          }
          graph::Value::Invalid =>
          {
            stack.push(crate::store::SelectNodeQuery::select_none().into());
          }
          _ => Err(InternalError::InvalidValueCast)?,
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
      &instructions::Instruction::AndBinaryOperator =>
      {
        execute_boolean_operator(stack, |a, b| a && b)?;
      }
      &instructions::Instruction::OrBinaryOperator =>
      {
        execute_boolean_operator(stack, |a, b| a || b)?;
      }
      &instructions::Instruction::NotUnaryOperator =>
      {
        let a: bool = stack.try_pop_into()?;
        stack.push((!a).into());
      }
      &instructions::Instruction::EqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a == b)?;
      }
      &instructions::Instruction::NotEqualBinaryOperator =>
      {
        execute_binary_operator(stack, |a, b| a != b)?;
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

pub(crate) fn eval_program(
  store: &crate::store::Store,
  program: super::Program,
  parameters: crate::graph::ValueObject,
) -> crate::Result<crate::graph::Value>
{
  let mut input_table = crate::value_table::ValueTable::new();
  input_table.add_row(crate::value_table::Row::new());
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
        let mut output_table = crate::value_table::ValueTable::new();
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
                  store.add_nodes(&mut tx, "default", vec![n.to_owned()].iter())?;
                  if let Some(var) = var
                  {
                    let _ = new_row.noreplace_insert(&var, crate::graph::Value::Node(n));
                  }
                }
                crate::graph::Value::Edge(e) =>
                {
                  store.add_edges(&mut tx, "default", vec![e.to_owned()].iter())?;
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
      instructions::Block::BlockMatch { blocks, optional } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
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
                  let query: crate::store::SelectNodeQuery = stack.try_pop_into()?;
                  let nodes = store.select_nodes(&mut tx, "default", query)?;

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

                  let edges = store.select_edges(&mut tx, "default", query, *directivity)?;

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
          if current_rows.is_empty() && optional
          {
            let mut new_row = row;
            for block in blocks.iter()
            {
              match block
              {
                BlockMatch::MatchNode { variable, .. } =>
                {
                  new_row.insert_none(&variable);
                }
                BlockMatch::MatchEdge {
                  left_variable,
                  edge_variable,
                  right_variable,
                  ..
                } =>
                {
                  new_row.insert_none(left_variable);
                  new_row.insert_none(edge_variable);
                  new_row.insert_none(right_variable);
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
      instructions::Block::Return { variables } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          let mut out_row = crate::value_table::Row::new();
          for (name, instructions) in variables.iter()
          {
            let mut stack = Stack::default();
            eval_instructions(&mut stack, row, instructions, &parameters)?;
            let value: graph::Value = stack.try_pop_into()?;
            out_row.insert(name.to_owned(), value.to_owned());
          }
          output_table.add_row(out_row);
        }
        let mut r = Vec::<crate::graph::Value>::new();
        r.push(crate::graph::Value::Array(
          variables
            .iter()
            .map(|(name, _)| crate::graph::Value::String(name.to_owned()))
            .collect(),
        ));
        for row in output_table.iter()
        {
          r.push(crate::graph::Value::Array(
            variables
              .iter()
              .map(|(name, _)| match row.get(name)
              {
                Some(v) => v.to_owned(),
                None => crate::graph::Value::Invalid,
              })
              .collect(),
          ));
        }
        tx.commit()?;
        return Ok(crate::graph::Value::Array(r));
      }
      instructions::Block::With { all, variables } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          let mut out_row = if all
          {
            row.clone()
          }
          else
          {
            crate::value_table::Row::new()
          };
          for (name, instructions) in variables.iter()
          {
            let mut stack = Stack::default();
            eval_instructions(&mut stack, row, instructions, &parameters)?;
            let value: graph::Value = stack.try_pop_into()?;
            out_row.insert(name.to_owned(), value.to_owned());
          }
          output_table.add_row(out_row);
        }
        input_table = output_table;
      }
      instructions::Block::Unwind { name, instructions } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
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
  tx.commit()?;
  Ok(crate::graph::Value::Invalid)
}
