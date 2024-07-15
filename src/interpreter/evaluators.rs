use std::borrow::{ Borrow, BorrowMut };

use crate::Result;

use super::instructions;

fn eval_instructions(
  stack: &mut Vec<crate::graph::Value>,
  row: &crate::value_table::Row,
  instructions: &instructions::Instructions
) -> Result<()> {
  for instruction in instructions {
    match instruction {
      instructions::Instruction::CreateEdge { label: _ } => {
        // let props = stack.pop();
        // stack.push(crate::graph::Edge {
        //   label: label,
        //   properties: props,
        // })
        return Err(crate::Error::Unimplemented("eval_instructions/CreateEdge"));
      }
      instructions::Instruction::CreateNode { labels } => {
        let props = stack.pop().unwrap();
        stack.push(crate::graph::Node {
          key: crate::graph::Key::default(),
          labels: labels.clone(),
          properties: props.to_object_safe(),
        }.into())
      }
      instructions::Instruction::Push { value } => {
        stack.push(value.clone());
      }
      instructions::Instruction::GetVariable { name } => {
        if let Some(value) = row.get(name) {
          stack.push(value.to_owned());
        } else {
          return Err(crate::Error::UnknownVariable(name.to_owned()));
        }
      }
    }
  }
  Ok(())
}

pub(crate) fn eval_program(
  store: &crate::store::Store,
  program: super::Program
) -> crate::Result<crate::graph::Value> {
  let mut input_table = crate::value_table::ValueTable::new();
  input_table.add_row(crate::value_table::Row::new());
  let mut tx = store.begin()?;
  let mut stack = Vec::<crate::graph::Value>::new();
  for block in program {
    match block {
      instructions::Block::Create { instructions, variables } => {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter() {
          eval_instructions(stack.borrow_mut(), row, instructions.borrow())?;
          let mut new_row = row.clone();
          for (v,var) in stack.drain(stack.len() - variables.len()..).zip(variables.iter()) {
            match v {
              crate::graph::Value::Node(n) => {
                store.add_nodes(tx.borrow_mut(), "default", vec![n.to_owned()].iter())?;
                new_row.insert(
                  var
                    .as_ref()
                    .ok_or(crate::Error::Unknown("executor/eval/variables[0].as_ref()"))?
                    .to_owned(),
                  crate::graph::Value::Node(n)
                );
              }
              _ => {
                return Err(crate::Error::Unimplemented("executor/eval/create"));
              }
            }
          }
          output_table.add_row(new_row);
        }
        input_table = output_table;
      }
      instructions::Block::Match { instructions, variables } => {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter() {
          eval_instructions(stack.borrow_mut(), row, instructions.borrow())?;
          for _ in variables.iter().rev() {
            let _ = stack.pop().unwrap();
          }
          let nodes = store.select_nodes(
            tx.borrow_mut(),
            "default",
            crate::store::SelectQuery::<core::slice::Iter<'_, crate::graph::Key>>::default()
          )?;

          for node in nodes.iter() {
            let mut new_row = row.clone();
            new_row.insert(
              variables[0]
                .as_ref()
                .ok_or(crate::Error::Unknown("executor/eval/variables[0].as_ref()"))?
                .to_owned(),
              crate::graph::Value::Node(node.to_owned())
            );
            output_table.add_row(new_row);
          }
        }
        input_table = output_table;
      }
      instructions::Block::Return { variables } => {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter() {
          let mut out_row = crate::value_table::Row::new();
          for (name, instructions) in variables.iter() {
            let mut stack = Vec::<crate::graph::Value>::new();
            eval_instructions(&mut stack, row, instructions)?;
            let value = stack.first().ok_or(crate::Error::Unknown("eval_program/return"))?;
            out_row.insert(name.to_owned(), value.to_owned());
          }
          output_table.add_row(out_row);
        }
        let mut r = Vec::<crate::graph::Value>::new();
        r.push(
          crate::graph::Value::Array(
            variables
              .iter()
              .map(|(name, _)| crate::graph::Value::String(name.to_owned()))
              .collect()
          )
        );
        for row in output_table.iter() {
          r.push(
            crate::graph::Value::Array(
              variables
                .iter()
                .map(|(name, _)| {
                  match row.get(name) {
                    Some(v) => v.to_owned(),
                    None => crate::graph::Value::Invalid,
                  }
                })
                .collect()
            )
          );
        }
        tx.commit()?;
        return Ok(crate::graph::Value::Array(r));
      }
    }
  }
  tx.commit()?;
  Ok(crate::graph::Value::Invalid)
}
