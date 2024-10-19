use crate::error::{CompileTimeError, InternalError};
// use crate::graph::ToValue;
use crate::interpreter::instructions::{Block, CreateAction, Instruction, Instructions};
use crate::interpreter::validator;
use crate::parser::ast;
use crate::{functions, Result};

fn compile_expression(
  function_manager: &functions::Manager,
  expression: &crate::parser::ast::Expression,
  instructions: &mut Instructions,
) -> Result<()>
{
  let expr = match expression
  {
    ast::Expression::Value(value) => Instruction::Push {
      value: value.value.clone(),
    },
    ast::Expression::Variable(variable) => Instruction::GetVariable {
      name: variable.identifier.clone(),
    },
    ast::Expression::Parameter(parameter) => Instruction::GetParameter {
      name: parameter.name.clone(),
    },
    ast::Expression::FunctionCall(function_call) =>
    {
      for v in function_call.arguments.iter()
      {
        compile_expression(function_manager, v, instructions)?;
      }
      Instruction::FunctionCall {
        function: function_manager.get::<CompileTimeError>(&function_call.name)?,
        arguments_count: function_call.arguments.len(),
      }
    }
    ast::Expression::Array(array) =>
    {
      for v in array.array.iter()
      {
        compile_expression(function_manager, v, instructions)?;
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
        compile_expression(function_manager, v, instructions)?;
        keys.push(k.to_owned());
      }
      Instruction::CreateMap { keys: keys }
    }
    ast::Expression::MemberAccess(member_access) =>
    {
      compile_expression(function_manager, &member_access.left, instructions)?;
      Instruction::MemberAccess {
        path: member_access.path.to_owned(),
      }
    }
  };
  instructions.push(expr);
  Ok(())
}

fn compile_optional_expression(
  function_manager: &functions::Manager,
  properties: &Option<ast::Expression>,
  instructions: &mut Instructions,
) -> Result<()>
{
  if let Some(expr) = properties
  {
    compile_expression(function_manager, expr, instructions)?;
  }
  else
  {
    instructions.push(Instruction::Push {
      value: crate::graph::Value::Invalid,
    });
  }
  Ok(())
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
  function_manager: &functions::Manager,
  validator: &mut validator::Validator,
  node: &crate::parser::ast::NodePattern,
  instructions: &mut Instructions,
  variables: &mut Vec<Option<String>>,
) -> Result<()>
{
  validator.declare_node_variable(&node)?;
  variables.push(node.variable.to_owned());
  compile_optional_expression(function_manager, &node.properties, instructions);
  let mut labels = Default::default();
  compile_create_labels(&mut labels, &node.labels)?;
  instructions.push(Instruction::CreateNodeLiteral { labels });
  Ok(())
}

fn compile_create_labels(
  labels: &mut Vec<String>,
  label_expressions: &ast::LabelExpression,
) -> Result<()>
{
  match &label_expressions
  {
    &ast::LabelExpression::And(expressions) =>
    {
      for expr in expressions.iter()
      {
        compile_create_labels(labels, &expr)?;
      }
      Ok(())
    }
    &ast::LabelExpression::String(label) =>
    {
      labels.push(label.to_owned());
      Ok(())
    }
    &ast::LabelExpression::None => Ok(()),
    _ => Err(
      InternalError::InvalidCreateLabels {
        context: "compile_create_labels",
      }
      .into(),
    ),
  }
}

fn compile_create_patterns(
  function_manager: &functions::Manager,
  validator: &mut validator::Validator,
  patterns: &Vec<crate::parser::ast::Pattern>,
) -> Result<Block>
{
  let actions = patterns.iter().map(|c| {
    let mut instructions = Instructions::new();
    let mut variables = Vec::<Option<String>>::new();
    match c
    {
      crate::parser::ast::Pattern::Node(node) =>
      {
        compile_create_node(
          function_manager,
          validator,
          node,
          &mut instructions,
          &mut variables,
        )?;
      }
      crate::parser::ast::Pattern::Edge(edge) =>
      {
        if validator.check_existing_node(&edge.source)?
        {
          instructions.push(Instruction::GetVariable {
            name: edge.source.variable.as_ref().unwrap().to_owned(),
          });
        }
        else
        {
          compile_create_node(
            function_manager,
            validator,
            &edge.source,
            &mut instructions,
            &mut variables,
          )?;
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
            function_manager,
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
        compile_optional_expression(function_manager, &edge.properties, &mut instructions);
        let mut labels = Default::default();
        compile_create_labels(&mut labels, &edge.labels)?;
        instructions.push(Instruction::CreateEdgeLiteral { labels });
      }
      crate::parser::ast::Pattern::Path(_) =>
      {
        return Err(
          InternalError::PathPatternInCreateExpression {
            context: "compiler/compile_create_patterns",
          }
          .into(),
        );
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

fn compile_match_node(
  function_manager: &functions::Manager,
  node: &crate::parser::ast::NodePattern,
  instructions: &mut Instructions,
) -> Result<()>
{
  compile_optional_expression(function_manager, &node.properties, instructions)?;
  let mut labels = Default::default();
  compile_create_labels(&mut labels, &node.labels)?;
  instructions.push(Instruction::CreateNodeLiteral { labels });
  Ok(())
}

fn compile_match_edge(
  function_manager: &functions::Manager,

  validator: &mut validator::Validator,
  path_variable: Option<String>,
  edge: &crate::parser::ast::EdgePattern,
) -> Result<Block>
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
    compile_match_node(function_manager, &edge.source, &mut instructions)?;
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
    compile_match_node(function_manager, &edge.destination, &mut instructions)?;
  }
  compile_optional_expression(function_manager, &edge.properties, &mut instructions)?;
  let mut labels = Default::default();
  compile_create_labels(&mut labels, &edge.labels)?;
  instructions.push(Instruction::CreateEdgeLiteral { labels });
  Ok(Block::MatchEdge {
    instructions: instructions,
    left_variable: source_variable,
    edge_variable: edge.variable.to_owned(),
    right_variable: destination_variable,
    path_variable: path_variable,
  })
}

fn compile_match_patterns(
  function_manager: &functions::Manager,

  validator: &mut validator::Validator,
  patterns: &Vec<crate::parser::ast::Pattern>,
) -> Result<Vec<Block>>
{
  let blocks = patterns
    .iter()
    .map(|c| match c
    {
      crate::parser::ast::Pattern::Node(node) =>
      {
        let mut instructions = Instructions::new();
        validator.declare_node_variable(node)?;
        compile_match_node(function_manager, node, &mut instructions)?;
        Ok(Block::MatchNode {
          instructions: instructions,
          variable: node.variable.to_owned(),
        })
      }
      crate::parser::ast::Pattern::Edge(edge) =>
      {
        compile_match_edge(function_manager, validator, None, &edge)
      }
      crate::parser::ast::Pattern::Path(path) => compile_match_edge(
        function_manager,
        validator,
        Some(path.variable.to_owned()),
        &path.edge,
      ),
    })
    .collect::<Result<Vec<Block>>>()?;
  Ok(blocks)
}

pub(crate) fn compile(
  function_manager: &functions::Manager,
  statements: crate::parser::ast::Statements,
) -> Result<super::Program>
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
          let cp = compile_create_patterns(function_manager, &mut validator, &create.patterns);
          match cp
          {
            Ok(cp) => Ok(Vec::from([cp]).into_iter()),
            Err(e) => Err(e),
          }
        }
        ast::Statement::Match(match_statement) =>
        {
          let cm =
            compile_match_patterns(function_manager, &mut validator, &match_statement.patterns);
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
            compile_expression(function_manager, &expr.expression, &mut instructions)?;
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
            compile_expression(function_manager, e, &mut instructions)?;
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
            compile_expression(function_manager, &e.expression, &mut instructions)?;
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
          compile_expression(function_manager, &unwind.expression, &mut instructions)?;
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
  if crate::consts::SHOW_PROGRAM
  {
    println!("program = {:#?}", program);
  }
  Ok(program)
}
