use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;

// use crate::graph::ToValue;
use crate::interpreter::context;
use crate::interpreter::instructions::{Block, Instruction, Instructions};
use crate::parser::ast;
use crate::Error;
use crate::Result;

fn compile_expression(expression: &crate::parser::ast::Expression, instructions: &mut Instructions)
{
  let expr = match expression
  {
    ast::Expression::Value(value) => Instruction::Push {
      value: value.value.clone(),
    },
    ast::Expression::Variable(variable) => Instruction::GetVariable {
      name: variable.identifier.clone(),
    },
    ast::Expression::Map(map) =>
    {
      let mut keys = Vec::new();
      for (k, v) in map.map.iter()
      {
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
)
{
  if let Some(expr) = properties
  {
    compile_expression(expr, instructions);
  }
  else
  {
    instructions.push(Instruction::Push {
      value: crate::graph::Value::Invalid,
    });
  }
}

fn has_variable(variables: &Vec<Option<String>>, var_name: &Option<String>) -> bool
{
  if let Some(var_name) = var_name
  {
    if let Some(_) = variables
      .iter()
      .find(|item| item.as_ref().is_some_and(|v| *v == *var_name))
    {
      true
    }
    else
    {
      false
    }
  }
  else
  {
    false
  }
}

fn compile_create_node(
  context: &mut context::Context,
  node: &crate::parser::ast::GraphNode,
  instructions: &mut Instructions,
  variables: &mut Vec<Option<String>>,
  allow_existing: bool,
)
{
  context.check_graph_node(node, allow_existing);
  variables.push(node.variable.to_owned());
  compile_optional_expression(node.properties.borrow(), instructions);
  instructions.push(Instruction::CreateNodeLiteral {
    labels: node.labels.to_owned(),
  });
}

fn compile_create_patterns(
  context: &mut context::Context,
  patterns: &Vec<crate::parser::ast::Pattern>,
) -> Result<Block>
{
  let mut instructions = Instructions::new();
  let mut variables = Vec::<Option<String>>::new();

  for c in patterns.iter()
  {
    match c
    {
      crate::parser::ast::Pattern::GraphNode(node) =>
      {
        compile_create_node(context, node, &mut instructions, &mut variables, false);
      }
      crate::parser::ast::Pattern::GraphEdge(edge) =>
      {
        println!("{:?}", context);
        let mut second_should_swap = false;
        if context.existing_variable(&edge.source.variable, Some(context::VariableType::Node))?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.source.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          second_should_swap = true;
          compile_create_node(
            context,
            &edge.source,
            &mut instructions,
            &mut variables,
            false,
          );
          instructions.push(Instruction::Duplicate);
        }
        if context.existing_variable(
          edge.destination.variable.borrow(),
          Some(context::VariableType::Node),
        )?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.destination.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          compile_create_node(
            context,
            &edge.destination,
            &mut instructions,
            &mut variables,
            false,
          );
          instructions.push(Instruction::Duplicate);
          if second_should_swap
          {
            instructions.push(Instruction::Rot3);
          }
        }
        variables.push(edge.variable.to_owned());
        compile_optional_expression(edge.properties.borrow(), &mut instructions);
        instructions.push(Instruction::CreateEdgeLiteral {
          label: edge.label.as_ref().map(|x| x.to_owned()),
        });
      }
    }
  }
  Ok(Block::Create {
    instructions: instructions,
    variables: variables,
  })
}

fn compile_match_node(node: &crate::parser::ast::GraphNode, instructions: &mut Instructions)
{
  compile_optional_expression(node.properties.borrow(), instructions);
  instructions.push(Instruction::CreateNodeLiteral {
    labels: node.labels.to_owned(),
  });
}

fn compile_match_patterns(
  context: &context::Context,
  patterns: &Vec<crate::parser::ast::Pattern>,
) -> Result<Vec<Block>>
{
  let blocks = patterns
    .iter()
    .map(|c| match c
    {
      crate::parser::ast::Pattern::GraphNode(node) =>
      {
        let mut instructions = Instructions::new();
        compile_match_node(node, &mut instructions);
        Ok(Block::MatchNode {
          instructions: instructions,
          variable: node.variable.to_owned(),
        })
      }
      crate::parser::ast::Pattern::GraphEdge(edge) =>
      {
        let mut instructions = Instructions::new();
        let mut source_variable = None;
        if context.existing_variable(&edge.source.variable, Some(context::VariableType::Node))?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.source.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          source_variable = edge.source.variable.to_owned();
          compile_match_node(&edge.source, &mut instructions);
        }
        let mut destination_variable = None;
        if context.existing_variable(
          &edge.destination.variable,
          Some(context::VariableType::Node),
        )?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.destination.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          destination_variable = edge.destination.variable.to_owned();
          compile_match_node(&edge.destination, &mut instructions);
        }
        compile_optional_expression(edge.properties.borrow(), &mut instructions);
        instructions.push(Instruction::CreateEdgeLiteral {
          label: edge.label.as_ref().map(|x| x.to_owned()),
        });
        Ok(Block::MatchEdge {
          instructions: instructions,
          left_variable: source_variable,
          edge_variable: edge.variable.to_owned(),
          right_variable: destination_variable,
        })
      }
    })
    .collect::<Result<Vec<Block>>>()?;
  Ok(blocks)
}

pub(crate) fn compile(statements: crate::parser::ast::Statements) -> Result<super::Program>
{
  let context_cell = Cell::new(context::Context::default());
  let mut statements_err = Ok(());
  let program = statements
    .iter()
    .map(|stmt| {
      let mut c_context = context_cell.take();
      let inst = match stmt
      {
        ast::Statement::Create(create) =>
        {
          let cp = compile_create_patterns(&mut c_context, &create.patterns);
          match cp
          {
            Ok(cp) => Ok(Vec::from([cp]).into_iter()),
            Err(e) => Err(e),
          }
        }
        ast::Statement::Match(match_statement) =>
        {
          let cm = compile_match_patterns(&mut c_context, &match_statement.patterns);
          match cm
          {
            Ok(cm) => Ok(cm.into_iter()),
            Err(e) => Err(e),
          }
        }
        ast::Statement::Return(return_statement) =>
        {
          let mut variables = std::collections::BTreeMap::<String, Instructions>::new();

          for expr in return_statement.expressions.iter()
          {
            let mut ints = Instructions::new();
            compile_expression(expr.expression.borrow(), ints.borrow_mut());
            variables.insert(expr.name.to_owned(), ints);
          }

          Ok(
            Vec::from([Block::Return {
              variables: variables,
            }])
            .into_iter(),
          )
        }
        _ => Err(crate::Error::Unimplemented("compile")),
      };
      context_cell.set(c_context);
      inst
    })
    .scan(&mut statements_err, |err, gp| {
      gp.map_err(|e| **err = Err(e)).ok()
    })
    .flatten();
  let program = program.collect::<super::Program>();
  statements_err?;
  println!("program = {:?}", program);
  Ok(program)
}
