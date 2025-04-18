use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
  error::{self, CompileTimeError, InternalError},
  functions,
  interpreter::{
    expression_analyser,
    instructions::{
      self, Block, BlockMatch, CreateAction, Instruction, Instructions, RWAggregation, RWExpression,
    },
    validator,
  },
  parser::ast,
  Result,
};

static FAKE_VARIABLE_COUNTER: AtomicU64 = AtomicU64::new(0);

macro_rules! compile_binary_op {
  ( $x:tt, $function_manager:tt, $instructions:tt, $aggregations:tt ) => {
    compile_expression($function_manager, &$x.right, $instructions, $aggregations)?;
    compile_expression($function_manager, &$x.left, $instructions, $aggregations)?;
  };
}

fn compile_expression(
  function_manager: &functions::Manager,
  expression: &crate::parser::ast::Expression,
  instructions: &mut Instructions,
  aggregations: &mut Option<&mut HashMap<String, RWAggregation>>,
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
      let aggregator = function_manager.get_aggregator::<CompileTimeError>(&function_call.name);
      match aggregator
      {
        Ok(aggregator) =>
        {
          let var_name = format!(
            "__gqlite_aggregator_{}",
            FAKE_VARIABLE_COUNTER.fetch_add(1, Ordering::Relaxed)
          );
          let mut init_instructions = Instructions::new();
          let mut argument_instructions = Instructions::new();

          compile_expression(
            function_manager,
            function_call
              .arguments
              .get(0)
              .ok_or(error::InternalError::MissingAggregationArgument)?,
            &mut argument_instructions,
            aggregations,
          )?;
          if let Some(init_arg) = function_call.arguments.get(1)
          {
            compile_expression(
              function_manager,
              init_arg,
              &mut init_instructions,
              aggregations,
            )?;
          }

          aggregations
            .as_mut()
            .ok_or(error::InternalError::MissingAggregations)?
            .insert(
              var_name.to_owned(),
              RWAggregation {
                init_instructions,
                aggregator,
                argument_instructions,
              },
            );
          Instruction::GetVariable { name: var_name }
        }
        Err(_) =>
        {
          for v in function_call.arguments.iter()
          {
            compile_expression(function_manager, v, instructions, aggregations)?;
          }

          let function = function_manager.get_function::<CompileTimeError>(&function_call.name)?;
          Instruction::FunctionCall {
            function,
            arguments_count: function_call.arguments.len(),
          }
        }
      }
    }
    ast::Expression::Array(array) =>
    {
      for v in array.array.iter()
      {
        compile_expression(function_manager, v, instructions, aggregations)?;
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
        compile_expression(function_manager, v, instructions, aggregations)?;
        keys.push(k.to_owned());
      }
      Instruction::CreateMap { keys: keys }
    }
    ast::Expression::MemberAccess(member_access) =>
    {
      compile_expression(
        function_manager,
        &member_access.left,
        instructions,
        aggregations,
      )?;
      Instruction::MemberAccess {
        path: member_access.path.to_owned(),
      }
    }
    ast::Expression::LogicalAnd(logical_and) =>
    {
      compile_binary_op!(logical_and, function_manager, instructions, aggregations);
      Instruction::AndBinaryOperator
    }
    ast::Expression::LogicalOr(logical_or) =>
    {
      compile_binary_op!(logical_or, function_manager, instructions, aggregations);
      Instruction::OrBinaryOperator
    }
    ast::Expression::LogicalXor(logical_xor) =>
    {
      compile_binary_op!(logical_xor, function_manager, instructions, aggregations);
      Instruction::XorBinaryOperator
    }
    ast::Expression::RelationalEqual(relational_equal) =>
    {
      compile_binary_op!(
        relational_equal,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::EqualBinaryOperator
    }
    ast::Expression::RelationalDifferent(relational_different) =>
    {
      compile_binary_op!(
        relational_different,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::NotEqualBinaryOperator
    }
    ast::Expression::RelationalInferior(relational_inferior) =>
    {
      compile_binary_op!(
        relational_inferior,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::InferiorBinaryOperator
    }
    ast::Expression::RelationalSuperior(relational_superior) =>
    {
      compile_binary_op!(
        relational_superior,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::SuperiorBinaryOperator
    }
    ast::Expression::RelationalInferiorEqual(relational_inferior_equal) =>
    {
      compile_binary_op!(
        relational_inferior_equal,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::InferiorEqualBinaryOperator
    }
    ast::Expression::RelationalSuperiorEqual(relational_superior_equal) =>
    {
      compile_binary_op!(
        relational_superior_equal,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::SuperiorEqualBinaryOperator
    }
    ast::Expression::RelationalIn(relational_in) =>
    {
      compile_binary_op!(relational_in, function_manager, instructions, aggregations);
      Instruction::InBinaryOperator
    }
    ast::Expression::RelationalNotIn(relational_not_in) =>
    {
      compile_binary_op!(
        relational_not_in,
        function_manager,
        instructions,
        aggregations
      );
      Instruction::NotInBinaryOperator
    }

    ast::Expression::Addition(addition) =>
    {
      compile_binary_op!(addition, function_manager, instructions, aggregations);
      Instruction::AdditionBinaryOperator
    }
    ast::Expression::Substraction(substraction) =>
    {
      compile_binary_op!(substraction, function_manager, instructions, aggregations);
      Instruction::SubstractionBinaryOperator
    }
    ast::Expression::Multiplication(multiplication) =>
    {
      compile_binary_op!(multiplication, function_manager, instructions, aggregations);
      Instruction::MultiplicationBinaryOperator
    }
    ast::Expression::Division(division) =>
    {
      compile_binary_op!(division, function_manager, instructions, aggregations);
      Instruction::DivisionBinaryOperator
    }
    ast::Expression::Modulo(modulo) =>
    {
      compile_binary_op!(modulo, function_manager, instructions, aggregations);
      Instruction::ModuloBinaryOperator
    }
    ast::Expression::Negation(logical_negation) =>
    {
      compile_expression(
        function_manager,
        &logical_negation.value,
        instructions,
        aggregations,
      )?;
      Instruction::NegationUnaryOperator
    }
    ast::Expression::LogicalNegation(logical_negation) =>
    {
      compile_expression(
        function_manager,
        &logical_negation.value,
        instructions,
        aggregations,
      )?;
      Instruction::NotUnaryOperator
    }
    ast::Expression::IsNull(is_null) =>
    {
      compile_expression(function_manager, &is_null.value, instructions, aggregations)?;
      Instruction::IsNullUnaryOperator
    }
    ast::Expression::IsNotNull(is_null) =>
    {
      compile_expression(function_manager, &is_null.value, instructions, aggregations)?;
      instructions.push(Instruction::IsNullUnaryOperator);
      Instruction::NotUnaryOperator
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
    compile_expression(function_manager, expr, instructions, &mut None)?;
  }
  else
  {
    instructions.push(Instruction::Push {
      value: crate::graph::Value::Object(Default::default()),
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
  validator.check_unexisting_variable(&node.variable)?;
  validator.validate_node(&node)?;
  variables.push(node.variable.to_owned());
  compile_optional_expression(function_manager, &node.properties, instructions)?;
  let mut labels = Default::default();
  compile_labels_expression(&mut labels, &node.labels)?;
  instructions.push(Instruction::CreateNodeLiteral { labels });
  Ok(())
}

fn compile_labels_expression(
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
        compile_labels_expression(labels, &expr)?;
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

// Assume top of the stack contains an edge or node
fn compile_filter_labels(
  instructions: &mut Instructions,
  label_expressions: &ast::LabelExpression,
  has_label_function: &functions::Function,
) -> Result<()>
{
  match &label_expressions
  {
    &ast::LabelExpression::And(expressions) =>
    {
      instructions.push(Instruction::Push { value: true.into() });
      instructions.push(Instruction::Swap);
      for expr in expressions.iter()
      {
        compile_filter_labels(instructions, expr, has_label_function)?;
        // stack contains (a: bool) (b: labels) (c: bool)
        instructions.push(Instruction::InverseRot3);
        // stack contains (c: bool) (a: bool) (b: labels)
        instructions.push(Instruction::AndBinaryOperator);
        // stack contains (a&c: bool) (b: labels)
        instructions.push(Instruction::Swap);
        // stack contains (b: labels) (a&&c: bool)
      }
      Ok(())
    }
    &ast::LabelExpression::Or(expressions) =>
    {
      instructions.push(Instruction::Push {
        value: false.into(),
      });
      instructions.push(Instruction::Swap);
      for expr in expressions.iter()
      {
        compile_filter_labels(instructions, expr, has_label_function)?;
        // stack contains (a: bool) (b: labels) (c: bool)
        instructions.push(Instruction::InverseRot3);
        // stack contains (c: bool) (a: bool) (b: labels)
        instructions.push(Instruction::OrBinaryOperator);
        // stack contains (a||c: bool) (b: labels)
        instructions.push(Instruction::Swap);
        // stack contains (b: labels) (a||c: bool)
      }
      Ok(())
    }
    &ast::LabelExpression::Not(expr) =>
    {
      compile_filter_labels(instructions, expr, has_label_function)?;
      instructions.push(Instruction::NotUnaryOperator);
      Ok(())
    }
    &ast::LabelExpression::String(label) =>
    {
      instructions.push(Instruction::Duplicate);
      instructions.push(Instruction::Push {
        value: label.to_owned().into(),
      });
      instructions.push(Instruction::FunctionCall {
        function: has_label_function.to_owned(),
        arguments_count: 2,
      });
      Ok(())
    }
    &ast::LabelExpression::None =>
    {
      instructions.push(Instruction::Push { value: true.into() });
      Ok(())
    }
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
        validator.check_unexisting_variable(&edge.variable)?;
        if validator.is_valid_existing_node(&edge.source)?
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
        else if validator.is_valid_existing_node(&edge.destination)?
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
        validator.validate_edge(edge)?;
        variables.push(edge.variable.to_owned());
        compile_optional_expression(function_manager, &edge.properties, &mut instructions)?;
        if !edge.labels.is_string()
        {
          Err(CompileTimeError::NoSingleRelationshipType)?;
        }
        let mut labels = Default::default();
        compile_labels_expression(&mut labels, &edge.labels)?;
        instructions.push(Instruction::CreateEdgeLiteral { labels });
      }
      crate::parser::ast::Pattern::Path(_) =>
      {
        Err(InternalError::PathPatternInCreateExpression {
          context: "compiler/compile_create_patterns",
        })?;
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
  filter: &mut Instructions,
  get_node_function_name: Option<&'static str>,
) -> Result<()>
{
  compile_optional_expression(function_manager, &node.properties, instructions)?;
  let mut labels = Default::default();
  if node.labels.is_all_inclusive()
  {
    compile_labels_expression(&mut labels, &node.labels)?;
  }
  else
  {
    if let Some(get_node_function_name) = get_node_function_name
    {
      filter.push(Instruction::Duplicate);
      filter.push(Instruction::FunctionCall {
        function: function_manager.get_function::<CompileTimeError>(get_node_function_name)?,
        arguments_count: 1,
      });
    }
    let has_label_function = function_manager.get_function::<CompileTimeError>("has_label")?;
    compile_filter_labels(filter, &node.labels, &has_label_function)?;
    filter.push(Instruction::Rot3);
    filter.push(Instruction::AndBinaryOperator);
    filter.push(Instruction::Swap);
  }
  instructions.push(Instruction::CreateNodeQuery { labels });
  Ok(())
}

fn compile_match_edge(
  function_manager: &functions::Manager,
  validator: &mut validator::Validator,
  path_variable: Option<String>,
  edge: &crate::parser::ast::EdgePattern,
  single_match: bool,
  previous_edges: &mut Vec<String>,
) -> Result<BlockMatch>
{
  if let Some(path_variable) = &path_variable
  {
    validator.declare_variable(
      path_variable.to_owned(),
      expression_analyser::ExpressionType::Path,
    )?;
  }

  let mut instructions = Instructions::new();
  let mut source_variable = None;
  let mut filter = Instructions::new();
  if validator.is_valid_existing_node(&edge.source)?
  {
    instructions.push(Instruction::GetVariable {
      name: edge.source.variable.as_ref().unwrap().to_owned(),
    });
    instructions.push(Instruction::CreateNodeQuery { labels: vec![] });
  }
  else
  {
    source_variable = edge.source.variable.to_owned();
    compile_match_node(
      function_manager,
      &edge.source,
      &mut instructions,
      &mut filter,
      Some("get_source"),
    )?;
  }
  let mut destination_variable = None;
  if validator.is_valid_existing_node(&edge.destination)?
  {
    instructions.push(Instruction::GetVariable {
      name: edge.destination.variable.as_ref().unwrap().to_owned(),
    });
    instructions.push(Instruction::CreateNodeQuery { labels: vec![] });
  }
  else
  {
    destination_variable = edge.destination.variable.to_owned();
    compile_match_node(
      function_manager,
      &edge.destination,
      &mut instructions,
      &mut filter,
      Some("get_destination"),
    )?;
  }
  if validator.is_valid_existing_edge(edge)?
  {
    if !validator.is_valid_existing_node(&edge.source)?
    {
      validator.validate_node(&edge.source)?;
    }
    if !validator.is_valid_existing_node(&edge.destination)?
    {
      validator.validate_node(&edge.destination)?;
    }
    instructions.push(Instruction::GetVariable {
      name: edge.variable.as_ref().unwrap().to_owned(),
    });
    instructions.push(Instruction::CreateEdgeQuery { labels: vec![] });
  }
  else
  {
    validator.validate_edge(edge)?;
    compile_optional_expression(function_manager, &edge.properties, &mut instructions)?;
    // Handle labels
    let mut labels = Default::default();
    if edge.labels.is_all_inclusive()
    {
      compile_labels_expression(&mut labels, &edge.labels)?;
    }
    else
    {
      let has_label_function = function_manager.get_function::<CompileTimeError>("has_label")?;
      compile_filter_labels(&mut filter, &edge.labels, &has_label_function)?;
      filter.push(Instruction::Rot3);
      filter.push(Instruction::AndBinaryOperator);
      filter.push(Instruction::Swap);
    }
    instructions.push(Instruction::CreateEdgeQuery { labels });
  }
  // Make sure that this edge isn't equal to an already matched edge
  let edge_variable = if single_match
  {
    edge.variable.to_owned()
  }
  else
  {
    let edge_variable = edge.variable.to_owned().unwrap_or_else(|| {
      format!(
        "__gqlite_edge_{}",
        FAKE_VARIABLE_COUNTER.fetch_add(1, Ordering::Relaxed)
      )
    });
    for other in previous_edges.iter()
    {
      filter.push(Instruction::Duplicate);
      filter.push(Instruction::GetVariable {
        name: other.clone(),
      });
      filter.push(Instruction::NotEqualBinaryOperator);
      filter.push(Instruction::InverseRot3);
      filter.push(Instruction::AndBinaryOperator);
      filter.push(Instruction::Swap);
    }
    previous_edges.push(edge_variable.clone());
    Some(edge_variable)
  };
  // Create block
  Ok(BlockMatch::MatchEdge {
    instructions: instructions,
    left_variable: source_variable,
    edge_variable,
    right_variable: destination_variable,
    path_variable,
    filter,
    directivity: edge.directivity,
  })
}

fn compile_match_patterns(
  function_manager: &functions::Manager,
  validator: &mut validator::Validator,
  patterns: &Vec<crate::parser::ast::Pattern>,
  where_expression: &Option<crate::parser::ast::Expression>,
  optional: bool,
) -> Result<Block>
{
  let is_single_match = patterns.len() == 1;
  let mut edge_variables = vec![];
  let blocks = patterns.iter().map(|c| match c
  {
    crate::parser::ast::Pattern::Node(node) =>
    {
      let mut instructions = Instructions::new();
      validator.validate_node(node)?;
      let mut filter = Instructions::new();
      compile_match_node(function_manager, node, &mut instructions, &mut filter, None)?;
      Ok(BlockMatch::MatchNode {
        instructions: instructions,
        variable: node.variable.to_owned(),
        filter,
      })
    }
    crate::parser::ast::Pattern::Edge(edge) => compile_match_edge(
      function_manager,
      validator,
      None,
      &edge,
      is_single_match,
      &mut edge_variables,
    ),
    crate::parser::ast::Pattern::Path(path) => compile_match_edge(
      function_manager,
      validator,
      Some(path.variable.to_owned()),
      &path.edge,
      is_single_match,
      &mut edge_variables,
    ),
  });
  let blocks = blocks.collect::<Result<_>>()?;
  let mut filter = Instructions::new();
  if let Some(where_expression) = where_expression
  {
    let ei = expression_analyser::ExpressionInfo::analyse(
      validator.variables_ref(),
      function_manager,
      where_expression,
    )?;
    if ei.aggregation_result
    {
      return Err(CompileTimeError::InvalidAggregation.into());
    }
    compile_expression(function_manager, where_expression, &mut filter, &mut None)?;
  }
  Ok(Block::BlockMatch {
    blocks,
    filter,
    optional,
  })
}

pub(crate) fn compile(
  function_manager: &functions::Manager,
  statements: crate::parser::ast::Statements,
) -> Result<super::Program>
{
  let mut validator = validator::Validator::new(function_manager.clone());
  let mut statements_err = Ok(());
  let program = statements
    .iter()
    .map(|stmt| {
      let inst = match stmt
      {
        ast::Statement::Create(create) =>
        {
          compile_create_patterns(function_manager, &mut validator, &create.patterns)
        }
        ast::Statement::Match(match_statement) => compile_match_patterns(
          function_manager,
          &mut validator,
          &match_statement.patterns,
          &match_statement.where_expression,
          match_statement.optional,
        ),
        ast::Statement::Return(return_statement) =>
        {
          let mut variables = Vec::<instructions::RWExpression>::new();

          if return_statement.all
          {
            for (name, _) in validator.variables_ref()
            {
              variables.push(instructions::RWExpression {
                name: name.to_owned(),
                instructions: vec![Instruction::GetVariable {
                  name: name.to_owned(),
                }],
                aggregations: Default::default(),
              });
            }
          }

          for expr in return_statement.expressions.iter()
          {
            let mut instructions = Instructions::new();
            let mut aggregations = HashMap::<String, RWAggregation>::new();
            compile_expression(
              function_manager,
              &expr.expression,
              &mut instructions,
              &mut Some(&mut aggregations),
            )?;
            variables.push(instructions::RWExpression {
              name: expr.name.clone(),
              instructions,
              aggregations,
            });
          }

          Ok(Block::Return { variables })
        }
        ast::Statement::Call(call) =>
        {
          let mut instructions = Instructions::new();
          for e in call.arguments.iter().rev()
          {
            compile_expression(function_manager, e, &mut instructions, &mut None)?;
          }
          Ok(Block::Call {
            arguments: instructions,
            name: call.name.to_owned(),
          })
        }
        ast::Statement::With(with) =>
        {
          let mut val_variables = Default::default();
          if with.all
          {
            val_variables = validator.to_variables();
          }
          let mut variables = Vec::<RWExpression>::new();
          let mut variable_names = Vec::<String>::new();
          for e in with.expressions.iter()
          {
            let mut instructions = Instructions::new();
            let mut aggregations = HashMap::<String, RWAggregation>::new();
            compile_expression(
              function_manager,
              &e.expression,
              &mut instructions,
              &mut Some(&mut aggregations),
            )?;
            if variable_names.contains(&e.name)
            {
              return Err(
                CompileTimeError::ColumnNameConflict {
                  name: e.name.to_owned(),
                }
                .into(),
              );
            }
            variable_names.push(e.name.to_owned());
            variables.push(RWExpression {
              name: e.name.to_owned(),
              instructions,
              aggregations,
            });
            val_variables.insert(
              e.name.to_owned(),
              expression_analyser::ExpressionInfo::analyse(
                validator.variables_ref(),
                &function_manager,
                &e.expression,
              )?
              .expression_type
              .into(),
            );
          }
          validator.set_variables(val_variables);
          Ok(Block::With {
            all: with.all,
            variables,
          })
        }
        ast::Statement::Unwind(unwind) =>
        {
          let mut instructions = Instructions::new();
          compile_expression(
            function_manager,
            &unwind.expression,
            &mut instructions,
            &mut None,
          )?;
          validator.declare_variable(
            unwind.name.to_owned(),
            expression_analyser::ExpressionType::Variant,
          )?;
          Ok(Block::Unwind {
            name: unwind.name.to_owned(),
            instructions,
          })
        }
        ast::Statement::Delete(delete_statement) => Ok(Block::Delete {
          detach: delete_statement.detach,
          instructions: delete_statement
            .expressions
            .iter()
            .map(|expr| {
              let mut instructions = Instructions::new();
              let ei = expression_analyser::ExpressionInfo::analyse(
                validator.variables_ref(),
                function_manager,
                expr,
              )?;
              match ei.expression_type
              {
                expression_analyser::ExpressionType::Node
                | expression_analyser::ExpressionType::Edge
                | expression_analyser::ExpressionType::Variant =>
                {
                  compile_expression(function_manager, &expr, &mut instructions, &mut None)?
                }
                _ => Err(CompileTimeError::InvalidDelete)?,
              }
              Ok(instructions)
            })
            .collect::<Result<_>>()?,
        }),
        ast::Statement::Update(update_statement) => Ok(Block::Update {
          updates: update_statement
            .updates
            .iter()
            .map(|x| match x
            {
              ast::OneUpdate::SetProperty(update_property)
              | ast::OneUpdate::AddProperty(update_property) =>
              {
                let mut instructions = Instructions::new();
                compile_expression(
                  function_manager,
                  &update_property.expression,
                  &mut instructions,
                  &mut None,
                )?;

                match x
                {
                  ast::OneUpdate::SetProperty(_) => Ok(instructions::UpdateOne::SetProperty {
                    target: update_property.target.to_owned(),
                    path: update_property.path.to_owned(),
                    instructions,
                  }),
                  ast::OneUpdate::AddProperty(_) => Ok(instructions::UpdateOne::AddProperty {
                    target: update_property.target.to_owned(),
                    path: update_property.path.to_owned(),
                    instructions,
                  }),
                  _ => Err(
                    InternalError::Unreachable {
                      context: "compile/Update/SetAddProperty",
                    }
                    .into(),
                  ),
                }
              }
              ast::OneUpdate::RemoveProperty(remove_property) =>
              {
                Ok(instructions::UpdateOne::RemoveProperty {
                  target: remove_property.target.to_owned(),
                  path: remove_property.path.to_owned(),
                })
              }
              ast::OneUpdate::AddLabels(add_labels) => Ok(instructions::UpdateOne::AddLabels {
                target: add_labels.target.to_owned(),
                labels: add_labels.labels.to_owned(),
              }),
              ast::OneUpdate::RemoveLabels(rm_labels) =>
              {
                Ok(instructions::UpdateOne::RemoveLabels {
                  target: rm_labels.target.to_owned(),
                  labels: rm_labels.labels.to_owned(),
                })
              }
            })
            .collect::<Result<_>>()?,
        }),
      };
      inst
    })
    .scan(&mut statements_err, |err, gp| {
      gp.map_err(|e| **err = Err(e)).ok()
    });
  let program = program.collect::<super::Program>();
  statements_err?;
  if crate::consts::SHOW_PROGRAM
  {
    println!("program = {:#?}", program);
  }
  Ok(program)
}
