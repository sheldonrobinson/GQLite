use std::borrow::{Borrow, BorrowMut};

use crate::{
  graph::{self, Key},
  Error, Result,
};

use super::instructions;

fn eval_instructions(
  stack: &mut Vec<crate::graph::Value>,
  row: &crate::value_table::Row,
  instructions: &instructions::Instructions,
) -> Result<()>
{
  for instruction in instructions
  {
    match instruction
    {
      instructions::Instruction::CreateEdgeLiteral { label } =>
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
            label: label.to_owned().unwrap_or(String::default()),
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
          return Err(Error::UnknownVariable(name.to_owned()));
        }
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
    }
  }
  Ok(())
}

pub(crate) fn eval_program(
  store: &crate::store::Store,
  program: super::Program,
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
      instructions::Block::Create {
        instructions,
        variables,
      } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          eval_instructions(stack.borrow_mut(), row, instructions.borrow())?;
          let mut new_row = row.clone();
          for (v, var) in stack
            .drain(stack.len() - variables.len()..)
            .zip(variables.iter())
          {
            match v
            {
              crate::graph::Value::Node(n) =>
              {
                store.add_nodes(tx.borrow_mut(), "default", vec![n.to_owned()].iter())?;
                new_row.insert(
                  var
                    .as_ref()
                    .ok_or(Error::Unknown("executor/eval/variables[0].as_ref()"))?
                    .to_owned(),
                  crate::graph::Value::Node(n),
                );
              }
              crate::graph::Value::Edge(e) =>
              {
                store.add_edges(tx.borrow_mut(), "default", vec![e.to_owned()].iter())?;
                new_row.insert(
                  var
                    .as_ref()
                    .ok_or(Error::Unknown("executor/eval/variables[0].as_ref()"))?
                    .to_owned(),
                  crate::graph::Value::Edge(e),
                );
              }
              _ =>
              {
                return Err(Error::Unimplemented("executor/eval/create"));
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
      } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          eval_instructions(stack.borrow_mut(), row, instructions.borrow())?;
          let _template = stack.pop().unwrap();
          let nodes = store.select_nodes(
            tx.borrow_mut(),
            "default",
            crate::store::SelectNodeQuery::select_all(),
          )?;

          for node in nodes.iter()
          {
            let mut new_row = row.clone();
            new_row.insert(
              variable
                .as_ref()
                .ok_or(Error::Unknown("executor/eval/match_node/variable.as_ref()"))?
                .to_owned(),
              crate::graph::Value::Node(node.to_owned()),
            );
            output_table.add_row(new_row);
          }
        }
        input_table = output_table;
      }
      instructions::Block::MatchEdge {
        instructions,
        left_variable,
        edge_variable,
        right_variable,
      } =>
      {
        let mut output_table = crate::value_table::ValueTable::new();
        for row in input_table.iter()
        {
          eval_instructions(stack.borrow_mut(), row, instructions.borrow())?;
          let _template = stack.pop().unwrap();
          let edges = store.select_edges(
            tx.borrow_mut(),
            "default",
            crate::store::SelectEdgeQuery::select_all(),
          )?;

          for edge in edges.iter()
          {
            let mut new_row = row.clone();
            if let Some(left_variable) = left_variable.to_owned()
            {
              new_row.insert(
                left_variable,
                crate::graph::Value::Node(edge.source.to_owned()),
              );
            }
            if let Some(right_variable) = right_variable.to_owned()
            {
              new_row.insert(
                right_variable,
                crate::graph::Value::Node(edge.destination.to_owned()),
              );
            }
            if let Some(edge_variable) = edge_variable.to_owned()
            {
              new_row.insert(edge_variable, crate::graph::Value::Edge(edge.to_owned()));
            }
            output_table.add_row(new_row);
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
            eval_instructions(&mut stack, row, instructions)?;
            let value = stack.first().ok_or(Error::Unknown("eval_program/return"))?;
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
    }
  }
  tx.commit()?;
  Ok(crate::graph::Value::Invalid)
}
