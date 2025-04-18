use std::borrow::{ Borrow, BorrowMut };

use crate::graph::ToValue;
use crate::parser::ast;
use crate::interpreter::instructions::{ Instruction, Instructions, Block };

fn compile_patterns(
  patterns: &Vec<crate::parser::ast::GraphNodeOrEdge>,
  instructions: &mut Instructions,
  variables: &mut Vec<Option<String>>
) {
  for c in patterns.iter() {
    match c {
      crate::parser::ast::GraphNodeOrEdge::GraphNode(node) => {
        variables.push(node.variable.to_owned());
        instructions.push(Instruction::Push { value: crate::graph::Node::default().to_value() });
      }
      crate::parser::ast::GraphNodeOrEdge::GraphEdge(edge) => {
        variables.push(edge.variable.to_owned());
        instructions.push(Instruction::Push { value: crate::graph::Edge::default().to_value() });
      }
    }
  }
}

fn compile_expression(
  expression: &crate::parser::ast::Expression,
  instructions: &mut Instructions
) {
  instructions.push(
    match expression {
      ast::Expression::Value(value) => {
        Instruction::Push { value: value.value.clone() }
      }
      ast::Expression::Variable(variable) => {
        Instruction::GetVariable { name: variable.identifier.clone()  }
      }
    }
  );
}

pub(crate) fn compile(statements: crate::parser::ast::Statements) -> crate::Result<super::Program> {
  let program = statements.iter().map(|stmt| {
    match stmt {
      ast::Statement::Create(create) => {
        let mut instructions = Instructions::new();
        let mut variables = Vec::<Option<String>>::new();
        compile_patterns(
          create.patterns.borrow(),
          instructions.borrow_mut(),
          variables.borrow_mut()
        );
        Ok(Block::Create { instructions: instructions, variables: variables })
      }
      ast::Statement::Match(match_statement) => {
        let mut instructions = Instructions::new();
        let mut variables = Vec::<Option<String>>::new();
        compile_patterns(
          match_statement.patterns.borrow(),
          instructions.borrow_mut(),
          variables.borrow_mut()
        );
        Ok(Block::Match { instructions: instructions, variables: variables })
      }
      ast::Statement::Return(return_statement) => {
        let mut variables = std::collections::HashMap::<String, Instructions>::new();

        for expr in return_statement.expressions.iter() {
          let mut ints = Instructions::new();
          compile_expression(expr.expression.borrow(), ints.borrow_mut());
          variables.insert(expr.name.to_owned(), ints);
        }

        Ok(Block::Return { variables: variables })
      }
      _ => { Err(crate::Error::Unimplemented("compile")) }
    }
  });
  program.collect::<crate::Result<super::Program>>()
}
