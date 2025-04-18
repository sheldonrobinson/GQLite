// use crate::graph::ToValue;
use crate::interpreter::instructions::{Block, CreateAction, Instruction, Instructions};
use crate::interpreter::validator;
use crate::parser::ast;
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
    ast::Expression::Array(array) =>
    {
      for v in array.array.iter()
      {
        compile_expression(v, instructions);
      }
      Instruction::CreateArray {
        length: array.array.len(),
      }
    }
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
    ast::Expression::MemberAccess(member_access) =>
    {
      compile_expression(&member_access.left, instructions);
      Instruction::MemberAccess {
        path: member_access.path.to_owned(),
      }
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
  validator: &mut validator::Validator,
  node: &crate::parser::ast::GraphNode,
  instructions: &mut Instructions,
  variables: &mut Vec<Option<String>>,
) -> Result<()>
{
  validator.declare_node_variable(node)?;
  variables.push(node.variable.to_owned());
  compile_optional_expression(&node.properties, instructions);
  instructions.push(Instruction::CreateNodeLiteral {
    labels: node.labels.to_owned(),
  });
  Ok(())
}

fn compile_create_patterns(
  validator: &mut validator::Validator,
  patterns: &Vec<crate::parser::ast::Pattern>,
) -> Result<Block>
{
  let actions = patterns.iter().map(|c| {
    let mut instructions = Instructions::new();
    let mut variables = Vec::<Option<String>>::new();
    match c
    {
      crate::parser::ast::Pattern::GraphNode(node) =>
      {
        compile_create_node(validator, node, &mut instructions, &mut variables)?;
      }
      crate::parser::ast::Pattern::GraphEdge(edge) =>
      {
        println!("{:?}", validator);
        if validator.check_existing_node(&edge.source)?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.source.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          compile_create_node(validator, &edge.source, &mut instructions, &mut variables)?;
          instructions.push(Instruction::Duplicate);
        }
        if edge.source.variable.is_some()
          && edge.destination.variable.is_some()
          && edge.source.variable == edge.destination.variable
        {
          instructions.push(Instruction::Duplicate);
        }
        else if validator.check_existing_node(&edge.destination)?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.destination.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          compile_create_node(
            validator,
            &edge.destination,
            &mut instructions,
            &mut variables,
          )?;
          instructions.push(Instruction::Duplicate);
          instructions.push(Instruction::Rot3);
        }
        validator.declare_edge_variable(edge)?;
        variables.push(edge.variable.to_owned());
        compile_optional_expression(&edge.properties, &mut instructions);
        instructions.push(Instruction::CreateEdgeLiteral {
          label: edge.label.as_ref().map(|x| x.to_owned()),
        });
      }
    }
    Ok(CreateAction {
      instructions,
      variables,
    })
  });
  Ok(Block::Create {
    actions: actions.collect::<Result<Vec<CreateAction>>>()?,
  })
}

fn compile_match_node(node: &crate::parser::ast::GraphNode, instructions: &mut Instructions)
{
  compile_optional_expression(&node.properties, instructions);
  instructions.push(Instruction::CreateNodeLiteral {
    labels: node.labels.to_owned(),
  });
}

fn compile_match_patterns(
  validator: &mut validator::Validator,
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
        validator.declare_node_variable(node)?;
        compile_match_node(node, &mut instructions);
        Ok(Block::MatchNode {
          instructions: instructions,
          variable: node.variable.to_owned(),
        })
      }
      crate::parser::ast::Pattern::GraphEdge(edge) =>
      {
        let mut instructions = Instructions::new();
        validator.declare_edge_variable(edge)?;
        let mut source_variable = None;
        if validator.check_existing_node(&edge.source)?
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
        if validator.check_existing_node(&edge.destination)?
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
        compile_optional_expression(&edge.properties, &mut instructions);
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
  let mut validator = validator::Validator::default();
  let mut statements_err = Ok(());
  let program = statements
    .iter()
    .map(|stmt| {
      let inst = match stmt
      {
        ast::Statement::Create(create) =>
        {
          let cp = compile_create_patterns(&mut validator, &create.patterns);
          match cp
          {
            Ok(cp) => Ok(Vec::from([cp]).into_iter()),
            Err(e) => Err(e),
          }
        }
        ast::Statement::Match(match_statement) =>
        {
          let cm = compile_match_patterns(&mut validator, &match_statement.patterns);
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
            let mut instructions = Instructions::new();
            compile_expression(&expr.expression, &mut instructions);
            variables.insert(expr.name.to_owned(), instructions);
          }

          Ok(
            Vec::from([Block::Return {
              variables: variables,
            }])
            .into_iter(),
          )
        }
        ast::Statement::Call(call) =>
        {
          let mut instructions = Instructions::new();
          for e in call.arguments.iter().rev()
          {
            compile_expression(e, &mut instructions);
          }
          Ok(
            Vec::from([Block::Call {
              arguments: instructions,
              name: call.name.to_owned(),
            }])
            .into_iter(),
          )
        }
        ast::Statement::With(with) =>
        {
          let mut val_variables = Default::default();
          if with.all
          {
            val_variables = validator.to_variables();
          }
          let mut variables = std::collections::BTreeMap::<String, Instructions>::new();
          for e in with.expressions.iter()
          {
            let mut instructions = Instructions::new();
            compile_expression(&e.expression, &mut instructions);
            variables.insert(e.name.to_owned(), instructions);
            val_variables.insert(e.name.to_owned(), validator.evaluate(&e.expression)?);
          }
          validator.set_variables(val_variables);
          Ok(
            Vec::from([Block::With {
              all: with.all,
              variables,
            }])
            .into_iter(),
          )
        }
        ast::Statement::Unwind(unwind) =>
        {
          let mut instructions = Instructions::new();
          compile_expression(&unwind.expression, &mut instructions);
          Ok(
            Vec::from([Block::Unwind {
              name: unwind.name.to_owned(),
              instructions,
            }])
            .into_iter(),
          )
        }
        _ => Err(crate::Error::Unimplemented("compile")),
      };
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
