use std::collections::HashMap;

use crate::{
  error::{InternalError, RunTimeError},
  graph, Error, Result, Value,
};

use super::instructions;

fn execute_boolean_operator(
  stack: &mut Vec<crate::graph::Value>,
  operand: impl FnOnce(bool, bool) -> bool,
) -> Result<()>
{
  let a = stack.pop().ok_or(InternalError::EmptyStack {
    context: "in BooleanBinaryOperator a",
  })?;
  let b = stack.pop().ok_or(InternalError::EmptyStack {
    context: "in BooleanBinaryOperator b",
  })?;
  let a = a.to_boolean().ok_or(InternalError::ExpectedBoolean {
    context: "in BooleanBinaryOperator a",
  })?;
  let b = b.to_boolean().ok_or(InternalError::ExpectedBoolean {
    context: "in BooleanBinaryOperator b",
  })?;
  stack.push(operand(a, b).into());
  Ok(())
}

fn execute_binary_operator(
  stack: &mut Vec<crate::graph::Value>,
  operand: impl FnOnce(crate::graph::Value, crate::graph::Value) -> bool,
) -> Result<()>
{
  let a = stack.pop().ok_or(InternalError::EmptyStack {
    context: "in BooleanBinaryOperator a",
  })?;
  let b = stack.pop().ok_or(InternalError::EmptyStack {
    context: "in BooleanBinaryOperator b",
  })?;
  stack.push(operand(a, b).into());
  Ok(())
}
fn eval_instructions(
  stack: &mut Vec<crate::graph::Value>,
  row: &crate::value_table::Row,
  instructions: &instructions::Instructions,
  parameters: &crate::graph::ValueObject,
) -> Result<()>
{
  if crate::consts::SHOW_EVALUATOR_STATE
  {
    println!("-----------");
  }
  for instruction in instructions
  {
    if crate::consts::SHOW_EVALUATOR_STATE
    {
      println!("-- {:#?} {:#?}", instruction, stack);
    }
    match instruction
    {
      instructions::Instruction::CreateEdgeLiteral { labels } =>
      {
        let props = stack.pop().unwrap();
        let dst: graph::Node = stack
          .pop()
          .unwrap()
          .to_node()
          .ok_or(Error::Unknown("Expected node on stack."))?;
        let src: graph::Node = stack
          .pop()
          .unwrap()
          .to_node()
          .ok_or(Error::Unknown("Expected node on stack."))?;
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
        let props = stack.pop().unwrap();
        stack.push(
          crate::graph::Node {
            key: crate::graph::Key::default(),
            labels: labels.clone(),
            properties: props.to_object_safe(),
          }
          .into(),
        );
      }
      instructions::Instruction::FunctionCall {
        function,
        arguments_count,
      } =>
      {
        let args: Vec<graph::Value> = stack.drain((stack.len() - arguments_count)..).collect();
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
      instructions::Instruction::GetVariable { name } =>
      {
        if let Some(value) = row.get(name)
        {
          stack.push(value.to_owned());
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
          m.push(
            stack
              .pop()
              .ok_or_else(|| InternalError::MissingStackValue {
                context: "eval_instructions/CreateArray",
              })?,
          );
        }
        stack.push(graph::Value::Array(m));
      }
      instructions::Instruction::CreateMap { keys } =>
      {
        let mut m = crate::graph::ValueObject::new();
        for k in keys.iter().rev()
        {
          if let Some(value) = stack.pop()
          {
            m.insert(k.to_owned(), value);
          }
          else
          {
            return Err(Error::EmptyStack(format!("Missing value for key {:?}", k)));
          }
        }
        stack.push(graph::Value::Object(m));
      }
      instructions::Instruction::MemberAccess { path } =>
      {
        let v = stack
          .pop()
          .ok_or_else(|| Error::InternalError("Missing stack value for member access."))?;
        stack.push(v.access(path.iter()));
      }
      instructions::Instruction::Duplicate => stack.push(
        stack
          .last()
          .ok_or(Error::EmptyStack("in duplicate".to_string()))?
          .clone(),
      ),
      &instructions::Instruction::Rot3 =>
      {
        let a = stack
          .pop()
          .ok_or(Error::EmptyStack("in Rot3 a".to_string()))?;
        let b = stack
          .pop()
          .ok_or(Error::EmptyStack("in Rot3 b".to_string()))?;
        let c = stack
          .pop()
          .ok_or(Error::EmptyStack("in Rot3 c".to_string()))?;
        stack.push(a);
        stack.push(c);
        stack.push(b);
      }
      &instructions::Instruction::InverseRot3 =>
      {
        let a = stack
          .pop()
          .ok_or(Error::EmptyStack("in InverseRot3 a".to_string()))?;
        let b = stack
          .pop()
          .ok_or(Error::EmptyStack("in InverseRot3 b".to_string()))?;
        let c = stack
          .pop()
          .ok_or(Error::EmptyStack("in InverseRot3 c".to_string()))?;
        stack.push(b);
        stack.push(a);
        stack.push(c);
      }
      &instructions::Instruction::Swap =>
      {
        let a = stack
          .pop()
          .ok_or(Error::EmptyStack("in Swap a".to_string()))?;
        let b = stack
          .pop()
          .ok_or(Error::EmptyStack("in Swap b".to_string()))?;
        stack.push(a);
        stack.push(b);
      }
      &instructions::Instruction::Drop =>
      {
        stack
          .pop()
          .ok_or(Error::EmptyStack("in drop".to_string()))?;
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
        let a = stack.pop().ok_or(InternalError::EmptyStack {
          context: "in NotUnaryOperator a",
        })?;
        let a = a.to_boolean().ok_or(InternalError::ExpectedBoolean {
          context: "in NotUnaryOperator a",
        })?;
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
  fn insert_none(&mut self, k: Option<String>);
}

impl HashMapExt for HashMap<String, Value>
{
  fn insert_none(&mut self, k: Option<String>)
  {
    if let Some(k) = k
    {
      self.insert(k, Value::Invalid);
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
  let mut stack = Vec::<crate::graph::Value>::new();
  for block in program
  {
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
              .drain(stack.len() - action.variables.len()..)
              .zip(action.variables.iter())
            {
              match v
              {
                crate::graph::Value::Node(n) =>
                {
                  store.add_nodes(&mut tx, "default", vec![n.to_owned()].iter())?;
                  if let Some(var) = var
                  {
                    new_row.insert(var.to_owned(), crate::graph::Value::Node(n));
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
      instructions::Block::MatchNode {
        instructions,
        variable,
        filter,
        optional,
      } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          eval_instructions(&mut stack, row, &instructions, &parameters)?;
          let template = stack
            .pop()
            .ok_or_else(|| InternalError::MissingStackValue {
              context: "eval_program/match_node",
            })?
            .to_node()
            .ok_or_else(|| InternalError::ExpectedNode {
              context: "eval_program/MatchNode",
            })?;
          let nodes = store.select_nodes(
            &mut tx,
            "default",
            crate::store::SelectNodeQuery::select_labels_properties(
              template.labels.iter(),
              template.properties.iter(),
            ),
          )?;

          if nodes.len() == 0 && optional
          {
            let mut new_row = row.clone();
            new_row.insert_none(variable.clone());
            output_table.add_row(new_row);
          }
          else
          {
            for node in nodes.iter()
            {
              let mut new_row = row.clone();
              match &variable
              {
                Some(variable) =>
                {
                  new_row.insert(variable.to_owned(), node.to_owned().into());
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
                let mut stack = Vec::<crate::graph::Value>::default();
                stack.push(true.into());
                stack.push(node.to_owned().into());
                eval_instructions(&mut stack, &new_row, &filter, &parameters)?;
                stack.pop(); // Get rid of the edge
                stack
                  .pop()
                  .ok_or(Error::EmptyStack("in filtering edge".to_string()))?
                  .try_into()?
              };
              if should_add_row
              {
                output_table.add_row(new_row);
              }
            }
          }
        }
        println!(" -- {:#?}", output_table);
        input_table = output_table;
      }
      instructions::Block::MatchEdge {
        instructions,
        left_variable,
        edge_variable,
        right_variable,
        path_variable,
        filter,
        directivity,
        optional,
      } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          println!("{:#?} {:#?}", stack, instructions);
          eval_instructions(&mut stack, row, &instructions, &parameters)?;
          println!("{:#?}", stack);
          let template = stack
            .pop()
            .ok_or_else(|| InternalError::MissingStackValue {
              context: "eval_program/MatchEdge",
            })?
            .to_edge()
            .ok_or_else(|| InternalError::ExpectedEdge {
              context: "eval_program/MatchEdge",
            })?;

          let edges = if left_variable.is_some()
            && row.contains_key(&left_variable.clone().unwrap())
          {
            if right_variable.is_some() && row.contains_key(&right_variable.clone().unwrap())
            {
              store.select_edges(
                &mut tx,
                "default",
                crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
                  crate::store::SelectNodeQuery::select_keys(
                    [&row
                      .get(&left_variable.clone().unwrap())
                      .unwrap()
                      .to_node()
                      .ok_or(InternalError::ExpectedNode {
                        context: "MatchEdge",
                      })?
                      .key]
                    .into_iter(),
                  ),
                  template.labels.iter(),
                  template.properties.iter(),
                  crate::store::SelectNodeQuery::select_keys(
                    [&row
                      .get(&right_variable.clone().unwrap())
                      .unwrap()
                      .to_node()
                      .ok_or(InternalError::ExpectedNode {
                        context: "MatchEdge",
                      })?
                      .key]
                    .into_iter(),
                  ),
                ),
                directivity,
              )?
            }
            else
            {
              store.select_edges(
                &mut tx,
                "default",
                crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
                  crate::store::SelectNodeQuery::select_keys(
                    [&row
                      .get(&left_variable.clone().unwrap())
                      .unwrap()
                      .to_node()
                      .ok_or(InternalError::ExpectedNode {
                        context: "MatchEdge",
                      })?
                      .key]
                    .into_iter(),
                  ),
                  template.labels.iter(),
                  template.properties.iter(),
                  crate::store::SelectNodeQuery::select_labels_properties(
                    template.destination.labels.iter(),
                    template.destination.properties.iter(),
                  ),
                ),
                directivity,
              )?
            }
          }
          else if right_variable.is_some() && row.contains_key(&right_variable.clone().unwrap())
          {
            store.select_edges(
              &mut tx,
              "default",
              crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
                crate::store::SelectNodeQuery::select_labels_properties(
                  template.source.labels.iter(),
                  template.source.properties.iter(),
                ),
                template.labels.iter(),
                template.properties.iter(),
                crate::store::SelectNodeQuery::select_keys(
                  [&row
                    .get(&right_variable.clone().unwrap())
                    .unwrap()
                    .to_node()
                    .ok_or(InternalError::ExpectedNode {
                      context: "MatchEdge",
                    })?
                    .key]
                  .into_iter(),
                ),
              ),
              directivity,
            )?
          }
          else
          {
            store.select_edges(
              &mut tx,
              "default",
              crate::store::SelectEdgeQuery::select_source_destination_labels_properties(
                crate::store::SelectNodeQuery::select_labels_properties(
                  template.source.labels.iter(),
                  template.source.properties.iter(),
                ),
                template.labels.iter(),
                template.properties.iter(),
                crate::store::SelectNodeQuery::select_labels_properties(
                  template.destination.labels.iter(),
                  template.destination.properties.iter(),
                ),
              ),
              directivity,
            )?
          };

          if edges.len() == 0 && optional
          {
            let mut new_row = row.clone();
            new_row.insert_none(left_variable.clone());
            new_row.insert_none(right_variable.clone());
            new_row.insert_none(edge_variable.clone());
            new_row.insert_none(path_variable.clone());
            output_table.add_row(new_row);
          }
          else
          {
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
                let mut stack = Vec::<crate::graph::Value>::default();
                stack.push(true.into());
                stack.push(edge.edge.to_owned().into());
                eval_instructions(&mut stack, &new_row, &filter, &parameters)?;
                stack.pop(); // Get rid of the edge
                stack
                  .pop()
                  .ok_or(Error::EmptyStack("in filtering edge".to_string()))?
                  .try_into()?
              };
              if should_add_row
              {
                output_table.add_row(new_row);
              }
            }
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
            let mut stack = Vec::<crate::graph::Value>::new();
            eval_instructions(&mut stack, row, instructions, &parameters)?;
            let value = stack
              .first()
              .ok_or_else(|| InternalError::MissingStackValue {
                context: "eval_program/return",
              })?;
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
            let mut stack = Vec::<crate::graph::Value>::new();
            eval_instructions(&mut stack, row, instructions, &parameters)?;
            let value = stack
              .first()
              .ok_or_else(|| InternalError::MissingStackValue {
                context: "eval_program/with",
              })?;
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
          let mut stack = Vec::<crate::graph::Value>::new();
          eval_instructions(&mut stack, row, &instructions, &parameters)?;
          let value = stack
            .pop()
            .ok_or_else(|| InternalError::MissingStackValue {
              context: "eval_program/unwind",
            })?;
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
