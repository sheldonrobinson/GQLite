use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::interpreter::instructions::Modifiers;
use crate::value_table;
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
  ( $this:tt, $x:tt, $instructions:tt, $aggregations:tt ) => {
    $this.compile_expression(&$x.right, $instructions, $aggregations)?;
    $this.compile_expression(&$x.left, $instructions, $aggregations)?;
  };
}

struct Compiler
{
  function_manager: functions::Manager,
  validator: validator::Validator,
  variables: HashMap<String, value_table::ColId>,
  persistent_variables: HashMap<String, usize>,
  temporary_variables: usize,
}

impl Compiler
{
  /// Get the variables sizes from the current state of the compiler.
  fn variables_size(&self) -> instructions::VariablesSizes
  {
    instructions::VariablesSizes {
      temporary_variables: self.temporary_variables,
      persistent_variables: self.persistent_variables.len(),
    }
  }

  /// Get the index of the variable in the row of variables
  fn get_variable_index(&self, identifier: &String) -> Result<usize>
  {
    self
      .variables
      .get(identifier)
      .ok_or_else(|| {
        InternalError::UnknownVariable {
          name: identifier.clone(),
        }
        .into()
      })
      .map(|x| *x)
  }
  fn compile_expression(
    &mut self,
    expression: &crate::parser::ast::Expression,
    instructions: &mut Instructions,
    aggregations: &mut Option<&mut HashMap<value_table::ColId, RWAggregation>>,
  ) -> Result<()>
  {
    expression_analyser::ExpressionInfo::analyse(
      self.validator.variables_ref(),
      &self.function_manager,
      &expression,
    )?;

    let expr = match expression
    {
      ast::Expression::Value(value) => Instruction::Push {
        value: value.value.clone(),
      },
      ast::Expression::Variable(variable) => Instruction::GetVariable {
        col_id: self.get_variable_index(&variable.identifier)?,
      },
      ast::Expression::Parameter(parameter) => Instruction::GetParameter {
        name: parameter.name.clone(),
      },
      ast::Expression::FunctionCall(function_call) =>
      {
        let aggregator = self
          .function_manager
          .get_aggregator::<CompileTimeError>(&function_call.name);
        match aggregator
        {
          Ok(aggregator) =>
          {
            let var_col_id = self.temporary_variables;
            self.temporary_variables += 1;
            let mut init_instructions = Instructions::new();
            let mut argument_instructions = Instructions::new();

            self.compile_expression(
              function_call
                .arguments
                .get(0)
                .ok_or(error::InternalError::MissingAggregationArgument)?,
              &mut argument_instructions,
              aggregations,
            )?;
            if let Some(init_arg) = function_call.arguments.get(1)
            {
              self.compile_expression(init_arg, &mut init_instructions, aggregations)?;
            }

            aggregations
              .as_mut()
              .ok_or(error::InternalError::MissingAggregations)?
              .insert(
                var_col_id,
                RWAggregation {
                  init_instructions,
                  aggregator,
                  argument_instructions,
                },
              );
            Instruction::GetVariable { col_id: var_col_id }
          }
          Err(_) =>
          {
            for v in function_call.arguments.iter()
            {
              self.compile_expression(v, instructions, aggregations)?;
            }

            let function = self
              .function_manager
              .get_function::<CompileTimeError>(&function_call.name)?;
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
          self.compile_expression(v, instructions, aggregations)?;
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
          self.compile_expression(v, instructions, aggregations)?;
          keys.push(k.to_owned());
        }
        Instruction::CreateMap { keys: keys }
      }
      ast::Expression::MemberAccess(member_access) =>
      {
        self.compile_expression(&member_access.left, instructions, aggregations)?;
        Instruction::MemberAccess {
          path: member_access.path.to_owned(),
        }
      }
      ast::Expression::IndexAccess(index_access) =>
      {
        self.compile_expression(&index_access.left, instructions, aggregations)?;
        self.compile_expression(&index_access.index, instructions, aggregations)?;
        Instruction::IndexAccess
      }
      ast::Expression::RangeAccess(index_access) =>
      {
        self.compile_expression(&index_access.left, instructions, aggregations)?;
        let start = if let Some(start) = &index_access.start
        {
          self.compile_expression(start, instructions, aggregations)?;
          true
        }
        else
        {
          false
        };
        let end = if let Some(end) = &index_access.end
        {
          self.compile_expression(end, instructions, aggregations)?;
          true
        }
        else
        {
          false
        };

        Instruction::RangeAccess { start, end }
      }
      ast::Expression::LogicalAnd(logical_and) =>
      {
        compile_binary_op!(self, logical_and, instructions, aggregations);
        Instruction::AndBinaryOperator
      }
      ast::Expression::LogicalOr(logical_or) =>
      {
        compile_binary_op!(self, logical_or, instructions, aggregations);
        Instruction::OrBinaryOperator
      }
      ast::Expression::LogicalXor(logical_xor) =>
      {
        compile_binary_op!(self, logical_xor, instructions, aggregations);
        Instruction::XorBinaryOperator
      }
      ast::Expression::RelationalEqual(relational_equal) =>
      {
        compile_binary_op!(self, relational_equal, instructions, aggregations);
        Instruction::EqualBinaryOperator
      }
      ast::Expression::RelationalDifferent(relational_different) =>
      {
        compile_binary_op!(self, relational_different, instructions, aggregations);
        Instruction::NotEqualBinaryOperator
      }
      ast::Expression::RelationalInferior(relational_inferior) =>
      {
        compile_binary_op!(self, relational_inferior, instructions, aggregations);
        Instruction::InferiorBinaryOperator
      }
      ast::Expression::RelationalSuperior(relational_superior) =>
      {
        compile_binary_op!(self, relational_superior, instructions, aggregations);
        Instruction::SuperiorBinaryOperator
      }
      ast::Expression::RelationalInferiorEqual(relational_inferior_equal) =>
      {
        compile_binary_op!(self, relational_inferior_equal, instructions, aggregations);
        Instruction::InferiorEqualBinaryOperator
      }
      ast::Expression::RelationalSuperiorEqual(relational_superior_equal) =>
      {
        compile_binary_op!(self, relational_superior_equal, instructions, aggregations);
        Instruction::SuperiorEqualBinaryOperator
      }
      ast::Expression::RelationalIn(relational_in) =>
      {
        compile_binary_op!(self, relational_in, instructions, aggregations);
        Instruction::InBinaryOperator
      }
      ast::Expression::RelationalNotIn(relational_not_in) =>
      {
        compile_binary_op!(self, relational_not_in, instructions, aggregations);
        Instruction::NotInBinaryOperator
      }

      ast::Expression::Addition(addition) =>
      {
        compile_binary_op!(self, addition, instructions, aggregations);
        Instruction::AdditionBinaryOperator
      }
      ast::Expression::Subtraction(substraction) =>
      {
        compile_binary_op!(self, substraction, instructions, aggregations);
        Instruction::SubstractionBinaryOperator
      }
      ast::Expression::Multiplication(multiplication) =>
      {
        compile_binary_op!(self, multiplication, instructions, aggregations);
        Instruction::MultiplicationBinaryOperator
      }
      ast::Expression::Division(division) =>
      {
        compile_binary_op!(self, division, instructions, aggregations);
        Instruction::DivisionBinaryOperator
      }
      ast::Expression::Modulo(modulo) =>
      {
        compile_binary_op!(self, modulo, instructions, aggregations);
        Instruction::ModuloBinaryOperator
      }
      ast::Expression::Negation(logical_negation) =>
      {
        self.compile_expression(&logical_negation.value, instructions, aggregations)?;
        Instruction::NegationUnaryOperator
      }
      ast::Expression::LogicalNegation(logical_negation) =>
      {
        self.compile_expression(&logical_negation.value, instructions, aggregations)?;
        Instruction::NotUnaryOperator
      }
      ast::Expression::IsNull(is_null) =>
      {
        self.compile_expression(&is_null.value, instructions, aggregations)?;
        Instruction::IsNullUnaryOperator
      }
      ast::Expression::IsNotNull(is_null) =>
      {
        self.compile_expression(&is_null.value, instructions, aggregations)?;
        instructions.push(Instruction::IsNullUnaryOperator);
        Instruction::NotUnaryOperator
      }
    };
    instructions.push(expr);
    Ok(())
  }

  fn compile_optional_expression(
    &mut self,
    properties: &Option<ast::Expression>,
    instructions: &mut Instructions,
  ) -> Result<()>
  {
    if let Some(expr) = properties
    {
      self.compile_expression(function_manager, validator, expr, instructions, &mut None)?;
    }
    else
    {
      instructions.push(Instruction::Push {
        value: crate::graph::Value::Object(Default::default()),
      });
    }
    Ok(())
  }

  fn compile_create_node(
    &mut self,
    node: &crate::parser::ast::NodePattern,
    instructions: &mut Instructions,
    variables: &mut Vec<Option<String>>,
  ) -> Result<()>
  {
    validator.check_unexisting_variable(&node.variable)?;
    validator.validate_node(&node)?;
    variables.push(node.variable.to_owned());
    self.compile_optional_expression(
      function_manager,
      validator,
      &node.properties,
      instructions,
    )?;
    let mut labels = Default::default();
    self.compile_labels_expression(&mut labels, &node.labels)?;
    instructions.push(Instruction::CreateNodeLiteral { labels });
    Ok(())
  }

  fn compile_labels_expression(
    &mut self,
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
          self.compile_labels_expression(labels, &expr)?;
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
    &mut self,
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
    &mut self,
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
          compile_optional_expression(
            function_manager,
            validator,
            &edge.properties,
            &mut instructions,
          )?;
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
    &mut self,
    node: &crate::parser::ast::NodePattern,
    instructions: &mut Instructions,
    filter: &mut Instructions,
    get_node_function_name: Option<&'static str>,
  ) -> Result<()>
  {
    self.compile_optional_expression(
      function_manager,
      validator,
      &node.properties,
      instructions,
    )?;
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
    &mut self,
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
        validator,
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
        validator,
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
      compile_optional_expression(
        function_manager,
        validator,
        &edge.properties,
        &mut instructions,
      )?;
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

  fn compile_return_with(
    &mut self,
    all: bool,
    expressions: &Vec<ast::NamedExpression>,
    where_expression: &Option<ast::Expression>,
    modifiers: &ast::Modifiers,
  ) -> Result<(Vec<RWExpression>, Instructions, Modifiers)>
  {
    let mut variables = Vec::<RWExpression>::new();
    let mut val_variables = Default::default();
    let mut filter = Default::default();
    if all
    {
      val_variables = validator.to_variables();
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
    let mut variable_names = Vec::<String>::new();
    for e in expressions.iter()
    {
      let mut instructions = Instructions::new();
      let mut aggregations = HashMap::<String, RWAggregation>::new();
      compile_expression(
        function_manager,
        validator,
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
    // TODO this is ugly, there need to be a better way to have two sets of variables for validation
    let mut variables_tmp = validator.variables_ref().to_owned();
    variables_tmp.extend(val_variables.to_owned().into_iter());
    validator.set_variables(variables_tmp);

    // Compile where expression
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
      compile_expression(
        function_manager,
        validator,
        where_expression,
        &mut filter,
        &mut None,
      )?;
    }

    let modifiers = compile_modifiers(function_manager, validator, &modifiers)?;
    validator.set_variables(val_variables);

    Ok((variables, filter, modifiers))
  }

  fn compile_match_patterns(
    &mut self,
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
        compile_match_node(
          function_manager,
          validator,
          node,
          &mut instructions,
          &mut filter,
          None,
        )?;
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
      compile_expression(
        function_manager,
        validator,
        where_expression,
        &mut filter,
        &mut None,
      )?;
    }
    Ok(Block::BlockMatch {
      blocks,
      filter,
      optional,
    })
  }

  fn check_for_constant_integer_expression(&mut self, x: &ast::Expression) -> Result<()>
  {
    let ei = expression_analyser::ExpressionInfo::analyse(
      validator.variables_ref(),
      function_manager,
      &x,
    )?;
    if !ei.constant
    {
      Err(error::CompileTimeError::NonConstantExpression.into())
    }
    else
    {
      match ei.expression_type
      {
        expression_analyser::ExpressionType::Integer
        | expression_analyser::ExpressionType::Variant => Ok(()),
        _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
      }
    }
  }

  pub(crate) fn compile_modifiers(
    &mut self,
    modifiers: &ast::Modifiers,
  ) -> Result<instructions::Modifiers>
  {
    let limit = modifiers
      .limit
      .as_ref()
      .map(|x| {
        check_for_constant_integer_expression(function_manager, validator, x)?;
        let mut instructions = Instructions::new();
        compile_expression(
          function_manager,
          validator,
          &x,
          &mut instructions,
          &mut None,
        )?;
        Ok::<_, error::Error>(instructions)
      })
      .transpose()?;
    let skip = modifiers
      .skip
      .as_ref()
      .map(|x| {
        check_for_constant_integer_expression(function_manager, validator, x)?;
        let mut instructions = Instructions::new();
        compile_expression(
          function_manager,
          validator,
          &x,
          &mut instructions,
          &mut None,
        )?;
        Ok::<_, error::Error>(instructions)
      })
      .transpose()?;
    let order_by = modifiers.order_by.as_ref().map_or_else(
      || Ok(Default::default()),
      |x| {
        x.expressions
          .iter()
          .map(|x| {
            let mut instructions = Instructions::new();

            compile_expression(
              function_manager,
              validator,
              &x.expression,
              &mut instructions,
              &mut None,
            )?;

            Ok(instructions::OrderBy {
              asc: x.asc,
              instructions,
            })
          })
          .collect::<Result<_>>()
      },
    )?;
    Ok(instructions::Modifiers {
      limit,
      skip,
      order_by,
    })
  }
}

pub(crate) fn compile(
  function_manager: &functions::Manager,
  statements: crate::parser::ast::Statements,
) -> Result<super::Program>
{
  let validator = validator::Validator::new(function_manager.clone());
  let compiler = Compiler {
    function_manager: function_manager.clone(),
    validator,
    variables: Default::default(),
    persistent_variables: 0,
    temporary_variables: 0,
  };
  let mut statements_err = Ok(());
  let program = statements
    .iter()
    .map(|stmt| {
      compiler.temporary_variables = 0;
      let inst = match stmt
      {
        ast::Statement::Create(create) => compiler.compile_create_patterns(&create.patterns),
        ast::Statement::Match(match_statement) => compiler.compile_match_patterns(
          &match_statement.patterns,
          &match_statement.where_expression,
          match_statement.optional,
        ),
        ast::Statement::Return(return_statement) =>
        {
          let (variables, filter, modifiers) = compiler.compile_return_with(
            return_statement.all,
            &return_statement.expressions,
            &return_statement.where_expression,
            &return_statement.modifiers,
          )?;
          Ok(Block::Return {
            variables,
            filter,
            modifiers,
            variables_size: compiler.variables_size(),
          })
        }
        ast::Statement::Call(call) =>
        {
          let mut instructions = Instructions::new();
          for e in call.arguments.iter().rev()
          {
            compiler.compile_expression(e, &mut instructions, &mut None)?;
          }
          Ok(Block::Call {
            arguments: instructions,
            name: call.name.to_owned(),
            variables_size: compiler.variables_size(),
          })
        }
        ast::Statement::With(with) =>
        {
          let (variables, filter, modifiers) = compiler.compile_return_with(
            with.all,
            &with.expressions,
            &with.where_expression,
            &with.modifiers,
          )?;
          Ok(Block::With {
            variables,
            filter,
            modifiers,
            variables_size: compiler.variables_size(),
          })
        }
        ast::Statement::Unwind(unwind) =>
        {
          let mut instructions = Instructions::new();
          compiler.compile_expression(&unwind.expression, &mut instructions, &mut None)?;
          validator.declare_variable(
            unwind.name.to_owned(),
            expression_analyser::ExpressionType::Variant,
          )?;
          Ok(Block::Unwind {
            name: unwind.name.to_owned(),
            instructions,
            variables_size: compiler.variables_size(),
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
                  compiler.compile_expression(&expr, &mut instructions, &mut None)?
                }
                _ => Err(CompileTimeError::InvalidDelete)?,
              }
              Ok(instructions)
            })
            .collect::<Result<_>>()?,
          variables_size: compiler.variables_size(),
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
                  &mut validator,
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
          variables_size: compiler.variables_size(),
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
