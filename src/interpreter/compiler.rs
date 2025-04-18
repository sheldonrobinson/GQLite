use std::borrow::{Borrow, BorrowMut};

// use crate::graph::ToValue;
use crate::interpreter::instructions::{Block, Instruction, Instructions};
use crate::parser::ast;

use super::instructions;

fn compile_expression(
  expression: &crate::parser::ast::Expression,
  instructions: &mut Instructions,
) {
  let expr = match expression {
    ast::Expression::Value(value) => Instruction::Push {
      value: value.value.clone(),
    },
    ast::Expression::Variable(variable) => Instruction::GetVariable {
      name: variable.identifier.clone(),
    },
    ast::Expression::Map(map) => {
      let mut keys = Vec::new();
      for (k, v) in map.map.iter() {
        compile_expression(v, instructions);
        keys.push(k.to_owned());
      }
      Instruction::CreateMap { keys: keys }
    }
  };
  instructions.push(expr);
}

fn compile_optional_expression(
  properties: &Option<ast::Expression>,
  instructions: &mut Instructions,
) {
  if let Some(expr) = properties {
    compile_expression(expr, instructions);
  } else {
    instructions.push(Instruction::Push {
      value: crate::graph::Value::Invalid,
    });
  }
}

fn compile_patterns(
  patterns: &Vec<crate::parser::ast::Pattern>,
  instructions: &mut Instructions,
  variables: &mut Vec<Option<String>>,
) {
  for c in patterns.iter() {
    match c {
      crate::parser::ast::Pattern::GraphNode(node) => {
        variables.push(node.variable.to_owned());
        compile_optional_expression(node.properties.borrow(), instructions);
        instructions.push(Instruction::CreateNode {
          labels: node.labels.to_owned(),
        });
      }
      crate::parser::ast::Pattern::GraphEdge(edge) => {
        variables.push(edge.variable.to_owned());
        compile_optional_expression(edge.properties.borrow(), instructions);
        instructions.push(Instruction::CreateEdge {
          label: edge.label.to_owned(),
        });
      }
    }
  }
}

pub(crate) fn compile(statements: crate::parser::ast::Statements) -> crate::Result<super::Program> {
  let program = statements.iter().map(|stmt| match stmt {
    ast::Statement::Create(create) => {
      let mut instructions = Instructions::new();
      let mut variables = Vec::<Option<String>>::new();
      compile_patterns(
        create.patterns.borrow(),
        instructions.borrow_mut(),
        variables.borrow_mut(),
      );
      Ok(Block::Create {
        instructions: instructions,
        variables: variables,
      })
    }
    ast::Statement::Match(match_statement) => {
      let mut instructions = Instructions::new();
      let mut variables = Vec::<Option<String>>::new();
      compile_patterns(
        match_statement.patterns.borrow(),
        instructions.borrow_mut(),
        variables.borrow_mut(),
      );
      Ok(Block::Match {
        instructions: instructions,
        variables: variables,
      })
    }
    ast::Statement::Return(return_statement) => {
      let mut variables = std::collections::HashMap::<String, Instructions>::new();

      for expr in return_statement.expressions.iter() {
        let mut ints = Instructions::new();
        compile_expression(expr.expression.borrow(), ints.borrow_mut());
        variables.insert(expr.name.to_owned(), ints);
      }

      Ok(Block::Return {
        variables: variables,
      })
    }
    _ => Err(crate::Error::Unimplemented("compile")),
  });
  program.collect::<crate::Result<super::Program>>()
}
