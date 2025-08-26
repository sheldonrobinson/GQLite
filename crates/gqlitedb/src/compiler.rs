pub(crate) mod expression_analyser;
pub(crate) mod variables_manager;

use crate::{
  compiler::variables_manager::VariablesManager, interpreter::instructions::VariablesSizes,
  prelude::*,
};

use interpreter::instructions::{
  self, Block, BlockMatch, CreateAction, Instruction, Instructions, Modifiers, RWAggregation,
  RWExpression,
};
use parser::ast;

macro_rules! compile_binary_op {
  ( $this:tt, $x:tt, $instructions:tt, $aggregations:tt ) => {
    $this.compile_expression(&$x.right, $instructions, $aggregations)?;
    $this.compile_expression(&$x.left, $instructions, $aggregations)?;
  };
}

struct Compiler
{
  function_manager: functions::Manager,
  variables_manager: variables_manager::VariablesManager,
  temporary_variables: usize,
}

impl Compiler
{
  /// Get the variables sizes from the current state of the compiler.
  fn variables_size(&self) -> instructions::VariablesSizes
  {
    instructions::VariablesSizes {
      temporary_variables: self.temporary_variables,
      persistent_variables: self.variables_manager.variables_count(),
    }
  }

  fn create_temporary_variable(&mut self) -> usize
  {
    let col_id = self.temporary_variables;
    self.temporary_variables += 1;
    col_id + self.variables_manager.variables_count()
  }

  fn compile_expression(
    &mut self,
    expression: &crate::parser::ast::Expression,
    instructions: &mut Instructions,
    aggregations: &mut Option<&mut Vec<(value_table::ColId, RWAggregation)>>,
  ) -> Result<()>
  {
    expression_analyser::Analyser::new(&self.variables_manager, &self.function_manager)
      .analyse(expression)?;

    let expr = match expression
    {
      ast::Expression::Value(value) => Instruction::Push {
        value: value.value.clone(),
      },
      ast::Expression::Variable(variable) => Instruction::GetVariable {
        col_id: self
          .variables_manager
          .get_variable_index(&variable.identifier)?,
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
            let var_col_id = self.create_temporary_variable();
            let mut init_instructions = Instructions::new();
            let mut argument_instructions = Instructions::new();

            self.compile_expression(
              function_call
                .arguments
                .first()
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
              .push((
                var_col_id,
                RWAggregation {
                  init_instructions,
                  aggregator,
                  argument_instructions,
                },
              ));
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
        Instruction::CreateMap { keys }
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
      ast::Expression::Subtraction(subtraction) =>
      {
        compile_binary_op!(self, subtraction, instructions, aggregations);
        Instruction::SubtractionBinaryOperator
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
      ast::Expression::Exponent(exponent) =>
      {
        compile_binary_op!(self, exponent, instructions, aggregations);
        Instruction::ExponentBinaryOperator
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
      self.compile_expression(expr, instructions, &mut None)?;
    }
    else
    {
      instructions.push(Instruction::Push {
        value: crate::value::Value::Map(Default::default()),
      });
    }
    Ok(())
  }

  fn compile_create_node(
    &mut self,
    node: &crate::parser::ast::NodePattern,
    instructions: &mut Instructions,
    variables: &mut Vec<Option<value_table::ColId>>,
  ) -> Result<()>
  {
    variables.push(
      self
        .variables_manager
        .get_variable_index_option(&node.variable)?,
    );
    self
      .variables_manager
      .mark_variables_as_set(&node.variable)?;
    self.compile_optional_expression(&node.properties, instructions)?;
    let mut labels = Default::default();
    Self::compile_labels_expression(&mut labels, &node.labels)?;
    instructions.push(Instruction::CreateNodeLiteral { labels });
    Ok(())
  }

  fn compile_labels_expression(
    labels: &mut Vec<String>,
    label_expressions: &ast::LabelExpression,
  ) -> Result<()>
  {
    match label_expressions
    {
      ast::LabelExpression::And(expressions) =>
      {
        for expr in expressions.iter()
        {
          Self::compile_labels_expression(labels, expr)?;
        }
        Ok(())
      }
      ast::LabelExpression::String(label) =>
      {
        labels.push(label.to_owned());
        Ok(())
      }
      ast::LabelExpression::None => Ok(()),
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
    match label_expressions
    {
      ast::LabelExpression::And(expressions) =>
      {
        instructions.push(Instruction::Push { value: true.into() });
        instructions.push(Instruction::Swap);
        for expr in expressions.iter()
        {
          Self::compile_filter_labels(instructions, expr, has_label_function)?;
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
      ast::LabelExpression::Or(expressions) =>
      {
        instructions.push(Instruction::Push {
          value: false.into(),
        });
        instructions.push(Instruction::Swap);
        for expr in expressions.iter()
        {
          Self::compile_filter_labels(instructions, expr, has_label_function)?;
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
      ast::LabelExpression::Not(expr) =>
      {
        Self::compile_filter_labels(instructions, expr, has_label_function)?;
        instructions.push(Instruction::NotUnaryOperator);
        Ok(())
      }
      ast::LabelExpression::String(label) =>
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
      ast::LabelExpression::None =>
      {
        instructions.push(Instruction::Push { value: true.into() });
        Ok(())
      }
    }
  }

  fn compile_create_patterns(&mut self, patterns: &[crate::parser::ast::Pattern]) -> Result<Block>
  {
    let actions = patterns.iter().map(|c| {
      let mut instructions = Instructions::new();
      let mut variables = Vec::<Option<value_table::ColId>>::new();
      match c
      {
        crate::parser::ast::Pattern::Node(node) =>
        {
          self.compile_create_node(node, &mut instructions, &mut variables)?;
        }
        crate::parser::ast::Pattern::Edge(edge) =>
        {
          // Generate source
          if self
            .variables_manager
            .is_set_variable(&edge.source.variable)?
          {
            instructions.push(Instruction::GetVariable {
              col_id: self
                .variables_manager
                .get_variable_index(edge.source.variable.as_ref().unwrap())?,
            });
          }
          else
          {
            self.compile_create_node(&edge.source, &mut instructions, &mut variables)?;
            instructions.push(Instruction::Duplicate);
          }
          // Generate destination
          if edge.source.variable.is_some()
            && edge.destination.variable.is_some()
            && edge.source.variable == edge.destination.variable
          {
            instructions.push(Instruction::Duplicate);
          }
          else if self
            .variables_manager
            .is_set_variable(&edge.destination.variable)?
          {
            instructions.push(Instruction::GetVariable {
              col_id: self
                .variables_manager
                .get_variable_index(edge.destination.variable.as_ref().unwrap())?,
            });
          }
          else
          {
            self.compile_create_node(&edge.destination, &mut instructions, &mut variables)?;
            instructions.push(Instruction::Duplicate);
            instructions.push(Instruction::Rot3);
          }
          variables.push(
            self
              .variables_manager
              .get_variable_index_option(&edge.variable)?,
          );
          self
            .variables_manager
            .mark_variables_as_set(&edge.variable)?;
          self.compile_optional_expression(&edge.properties, &mut instructions)?;
          if !edge.labels.is_string()
          {
            Err(CompileTimeError::NoSingleRelationshipType)?;
          }
          let mut labels = Default::default();
          Self::compile_labels_expression(&mut labels, &edge.labels)?;
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
      actions: actions.collect::<Result<_>>()?,
      variables_size: self.variables_size(),
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
    self.compile_optional_expression(&node.properties, instructions)?;
    let mut labels = Default::default();
    if node.labels.is_all_inclusive()
    {
      Self::compile_labels_expression(&mut labels, &node.labels)?;
    }
    else
    {
      if let Some(get_node_function_name) = get_node_function_name
      {
        filter.push(Instruction::Duplicate);
        filter.push(Instruction::FunctionCall {
          function: self
            .function_manager
            .get_function::<CompileTimeError>(get_node_function_name)?,
          arguments_count: 1,
        });
      }
      let has_label_function = self
        .function_manager
        .get_function::<CompileTimeError>("has_label")?;
      Self::compile_filter_labels(filter, &node.labels, &has_label_function)?;
      filter.push(Instruction::Rot3);
      filter.push(Instruction::AndBinaryOperator);
      filter.push(Instruction::Swap);
    }
    instructions.push(Instruction::CreateNodeQuery { labels });
    Ok(())
  }

  fn compile_match_edge(
    &mut self,
    path_variable: Option<ast::VariableIdentifier>,
    edge: &crate::parser::ast::EdgePattern,
    single_match: bool,
    previous_edges: &mut Vec<usize>,
  ) -> Result<BlockMatch>
  {
    let mut instructions = Instructions::new();
    let mut source_variable = None;
    let mut filter = Instructions::new();
    if self
      .variables_manager
      .is_set_variable(&edge.source.variable)?
    {
      instructions.push(Instruction::GetVariable {
        col_id: self
          .variables_manager
          .get_variable_index(edge.source.variable.as_ref().unwrap())?,
      });
      instructions.push(Instruction::CreateNodeQuery { labels: vec![] });
    }
    else
    {
      source_variable = edge.source.variable.to_owned();
      self.compile_match_node(
        &edge.source,
        &mut instructions,
        &mut filter,
        Some("get_source"),
      )?;
    }
    let mut destination_variable = None;
    if self
      .variables_manager
      .is_set_variable(&edge.destination.variable)?
    {
      instructions.push(Instruction::GetVariable {
        col_id: self
          .variables_manager
          .get_variable_index(edge.destination.variable.as_ref().unwrap())?,
      });
      instructions.push(Instruction::CreateNodeQuery { labels: vec![] });
    }
    else
    {
      destination_variable = edge.destination.variable.to_owned();
      self.compile_match_node(
        &edge.destination,
        &mut instructions,
        &mut filter,
        Some("get_destination"),
      )?;
    }
    if self.variables_manager.is_set_variable(&edge.variable)?
    {
      instructions.push(Instruction::GetVariable {
        col_id: self
          .variables_manager
          .get_variable_index(edge.variable.as_ref().unwrap())?,
      });
      instructions.push(Instruction::CreateEdgeQuery { labels: vec![] });
    }
    else
    {
      self.compile_optional_expression(&edge.properties, &mut instructions)?;
      // Handle labels
      let mut labels = Default::default();
      if edge.labels.is_all_inclusive()
      {
        Self::compile_labels_expression(&mut labels, &edge.labels)?;
      }
      else
      {
        let has_label_function = self
          .function_manager
          .get_function::<CompileTimeError>("has_label")?;
        Self::compile_filter_labels(&mut filter, &edge.labels, &has_label_function)?;
        filter.push(Instruction::Rot3);
        filter.push(Instruction::AndBinaryOperator);
        filter.push(Instruction::Swap);
      }
      instructions.push(Instruction::CreateEdgeQuery { labels });
    }

    // Mark variables as set once they are compiled
    self
      .variables_manager
      .mark_variables_as_set(&edge.source.variable)?;
    self
      .variables_manager
      .mark_variables_as_set(&edge.destination.variable)?;
    self
      .variables_manager
      .mark_variables_as_set(&edge.variable)?;

    // Make sure that this edge isn't equal to an already matched edge
    let edge_variable = if single_match
    {
      self
        .variables_manager
        .get_variable_index_option(&edge.variable)?
    }
    else
    {
      let edge_variable = self
        .variables_manager
        .get_variable_index_option(&edge.variable)?
        .unwrap_or_else(|| self.create_temporary_variable());
      for other in previous_edges.iter()
      {
        filter.push(Instruction::Duplicate);
        filter.push(Instruction::GetVariable { col_id: *other });
        filter.push(Instruction::NotEqualBinaryOperator);
        filter.push(Instruction::InverseRot3);
        filter.push(Instruction::AndBinaryOperator);
        filter.push(Instruction::Swap);
      }
      previous_edges.push(edge_variable);
      Some(edge_variable)
    };
    self
      .variables_manager
      .mark_variables_as_set(&path_variable)?;
    // Create block
    Ok(BlockMatch::MatchEdge {
      instructions,
      left_variable: self
        .variables_manager
        .get_variable_index_option(&source_variable)?,
      edge_variable,
      right_variable: self
        .variables_manager
        .get_variable_index_option(&destination_variable)?,
      path_variable: self
        .variables_manager
        .get_variable_index_option(&path_variable)?,
      filter,
      directivity: edge.directivity,
    })
  }

  #[allow(clippy::type_complexity)]
  fn compile_return_with(
    &mut self,
    all: bool,
    expressions: &[ast::NamedExpression],
    where_expression: &Option<ast::Expression>,
    modifiers: &ast::Modifiers,
  ) -> Result<(
    Vec<(String, RWExpression)>,
    Instructions,
    Modifiers,
    VariablesSizes,
  )>
  {
    let mut variables = Vec::<(ast::VariableIdentifier, RWExpression)>::new();
    let mut filter = Default::default();
    if all
    {
      for (var_id, var) in self.variables_manager.variables_iter()
      {
        variables.push((
          var_id.clone(),
          instructions::RWExpression {
            col_id: var.col_id(),
            instructions: vec![Instruction::GetVariable {
              col_id: var.col_id(),
            }],
            aggregations: Default::default(),
          },
        ));
      }
    }
    for e in expressions.iter()
    {
      let mut instructions = Instructions::new();
      let mut aggregations = Vec::<(usize, RWAggregation)>::new();
      self.compile_expression(
        &e.expression,
        &mut instructions,
        &mut Some(&mut aggregations),
      )?;
      if variables.iter().any(|(name, _)| *name == e.identifier)
      {
        return Err(
          CompileTimeError::ColumnNameConflict {
            name: e.identifier.name().to_owned(),
          }
          .into(),
        );
      }
      let col_id = self.variables_manager.analyse_named_expression(e)?;
      variables.push((
        e.identifier.clone(),
        RWExpression {
          col_id,
          instructions,
          aggregations,
        },
      ));
    }

    // Compile where expression
    if let Some(where_expression) = where_expression
    {
      let ei = expression_analyser::Analyser::new(&self.variables_manager, &self.function_manager)
        .analyse(where_expression)?;
      if ei.aggregation_result
      {
        return Err(CompileTimeError::InvalidAggregation.into());
      }
      self.compile_expression(where_expression, &mut filter, &mut None)?;
    }

    let modifiers = self.compile_modifiers(modifiers)?;
    let variables_size = self.variables_size();
    self
      .variables_manager
      .keep_variables(variables.iter().map(|(n, _)| n))?;

    let variables = variables
      .into_iter()
      .map(|(var_id, e)| (var_id.take_name(), e))
      .collect();
    Ok((variables, filter, modifiers, variables_size))
  }

  fn compile_match_patterns(
    &mut self,
    patterns: &[crate::parser::ast::Pattern],
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
        let mut filter = Instructions::new();
        self.compile_match_node(node, &mut instructions, &mut filter, None)?;
        self
          .variables_manager
          .mark_variables_as_set(&node.variable)?;
        Ok(BlockMatch::MatchNode {
          instructions,
          variable: self
            .variables_manager
            .get_variable_index_option(&node.variable)?,
          filter,
        })
      }
      crate::parser::ast::Pattern::Edge(edge) =>
      {
        self.compile_match_edge(None, edge, is_single_match, &mut edge_variables)
      }
      crate::parser::ast::Pattern::Path(path) => self.compile_match_edge(
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
      let ei = expression_analyser::Analyser::new(&self.variables_manager, &self.function_manager)
        .analyse(where_expression)?;
      if ei.aggregation_result
      {
        return Err(CompileTimeError::InvalidAggregation.into());
      }
      self.compile_expression(where_expression, &mut filter, &mut None)?;
    }
    Ok(Block::Match {
      blocks,
      filter,
      optional,
      variables_size: self.variables_size(),
    })
  }

  fn check_for_constant_integer_expression(&mut self, x: &ast::Expression) -> Result<()>
  {
    let ei = expression_analyser::Analyser::new(&self.variables_manager, &self.function_manager)
      .analyse(x)?;
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
        self.check_for_constant_integer_expression(x)?;
        let mut instructions = Instructions::new();
        self.compile_expression(x, &mut instructions, &mut None)?;
        Ok::<_, ErrorType>(instructions)
      })
      .transpose()?;
    let skip = modifiers
      .skip
      .as_ref()
      .map(|x| {
        self.check_for_constant_integer_expression(x)?;
        let mut instructions = Instructions::new();
        self.compile_expression(x, &mut instructions, &mut None)?;
        Ok::<_, ErrorType>(instructions)
      })
      .transpose()?;
    let order_by = modifiers.order_by.as_ref().map_or_else(
      || Ok(Default::default()),
      |x| {
        x.expressions
          .iter()
          .map(|x| {
            let mut instructions = Instructions::new();

            self.compile_expression(&x.expression, &mut instructions, &mut None)?;

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
) -> Result<interpreter::Program>
{
  let mut compiler = Compiler {
    variables_manager: VariablesManager::new(function_manager),
    function_manager: function_manager.clone(),
    temporary_variables: 0,
  };
  let mut statements_err = Ok(());
  let program = statements
    .iter()
    .map(|stmt| {
      compiler.temporary_variables = 0;
      compiler.variables_manager.analyse(stmt)?;
      let inst = match stmt
      {
        ast::Statement::CreateGraph(create_graph) => Ok(Block::CreateGraph {
          name: create_graph.name.to_owned(),
        }),
        ast::Statement::DropGraph(drop_graph) => Ok(Block::DropGraph {
          name: drop_graph.name.to_owned(),
          if_exists: drop_graph.if_exists,
        }),
        ast::Statement::UseGraph(use_graph) => Ok(Block::UseGraph {
          name: use_graph.name.to_owned(),
        }),
        ast::Statement::Create(create) => compiler.compile_create_patterns(&create.patterns),
        ast::Statement::Match(match_statement) => compiler.compile_match_patterns(
          &match_statement.patterns,
          &match_statement.where_expression,
          match_statement.optional,
        ),
        ast::Statement::Return(return_statement) =>
        {
          let (variables, filter, modifiers, variables_size) = compiler.compile_return_with(
            return_statement.all,
            &return_statement.expressions,
            &return_statement.where_expression,
            &return_statement.modifiers,
          )?;
          Ok(Block::Return {
            variables,
            filter,
            modifiers,
            variables_size,
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
          })
        }
        ast::Statement::With(with) =>
        {
          let (variables, filter, modifiers, variables_size) = compiler.compile_return_with(
            with.all,
            &with.expressions,
            &with.where_expression,
            &with.modifiers,
          )?;
          Ok(Block::With {
            variables: variables.into_iter().map(|(_, v)| v).collect(),
            filter,
            modifiers,
            variables_size,
          })
        }
        ast::Statement::Unwind(unwind) =>
        {
          let mut instructions = Instructions::new();
          compiler.compile_expression(&unwind.expression, &mut instructions, &mut None)?;
          Ok(Block::Unwind {
            col_id: compiler
              .variables_manager
              .get_variable_index(&unwind.identifier)?,
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
              let ei =
                expression_analyser::Analyser::new(&compiler.variables_manager, function_manager)
                  .analyse(expr)?;
              match ei.expression_type
              {
                expression_analyser::ExpressionType::Node
                | expression_analyser::ExpressionType::Edge
                | expression_analyser::ExpressionType::Variant =>
                {
                  compiler.compile_expression(expr, &mut instructions, &mut None)?
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
                compiler.compile_expression(
                  &update_property.expression,
                  &mut instructions,
                  &mut None,
                )?;

                match x
                {
                  ast::OneUpdate::SetProperty(_) => Ok(instructions::UpdateOne::SetProperty {
                    target: compiler
                      .variables_manager
                      .get_variable_index(&update_property.target)?,
                    path: update_property.path.to_owned(),
                    instructions,
                  }),
                  ast::OneUpdate::AddProperty(_) => Ok(instructions::UpdateOne::AddProperty {
                    target: compiler
                      .variables_manager
                      .get_variable_index(&update_property.target)?,
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
                  target: compiler
                    .variables_manager
                    .get_variable_index(&remove_property.target)?,
                  path: remove_property.path.to_owned(),
                })
              }
              ast::OneUpdate::AddLabels(add_labels) => Ok(instructions::UpdateOne::AddLabels {
                target: compiler
                  .variables_manager
                  .get_variable_index(&add_labels.target)?,
                labels: add_labels.labels.to_owned(),
              }),
              ast::OneUpdate::RemoveLabels(rm_labels) =>
              {
                Ok(instructions::UpdateOne::RemoveLabels {
                  target: compiler
                    .variables_manager
                    .get_variable_index(&rm_labels.target)?,
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
  let program = program.collect::<interpreter::Program>();
  statements_err?;
  if crate::consts::SHOW_PROGRAM
  {
    println!("program = {:#?}", program);
  }
  Ok(program)
}
