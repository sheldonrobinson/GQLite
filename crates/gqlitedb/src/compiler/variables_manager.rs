use std::collections::HashMap;

use crate::prelude::*;

use compiler::expression_analyser::{self, ExpressionType};
use parser::ast;

fn unknown_variable_error(name: &ast::VariableIdentifier) -> ErrorType
{
  InternalError::UnknownVariable {
    name: name.name().clone(),
  }
  .into()
}

// __     __         _       _     _
// \ \   / /_ _ _ __(_) __ _| |__ | | ___
//  \ \ / / _` | '__| |/ _` | '_ \| |/ _ \
//   \ V / (_| | |  | | (_| | |_) | |  __/
//    \_/ \__,_|_|  |_|\__,_|_.__/|_|\___|

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum VariableContent
{
  Node(ast::NodePattern),
  Edge(ast::EdgePattern),
  None,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable
{
  content: VariableContent,
  variable_type: ExpressionType,
  col_id: value_table::ColId,
  /// Track if the variable is expected to have been set or has just been declared.
  is_set: bool,
}

impl Variable
{
  fn from_node(value: ast::NodePattern, col_id: value_table::ColId) -> Self
  {
    Self {
      content: VariableContent::Node(value),
      variable_type: ExpressionType::Node,
      col_id,
      is_set: false,
    }
  }
  fn from_edge(value: ast::EdgePattern, col_id: value_table::ColId) -> Self
  {
    Self {
      content: VariableContent::Edge(value),
      variable_type: ExpressionType::Edge,
      col_id,
      is_set: false,
    }
  }
  fn from_expression(value: ExpressionType, col_id: value_table::ColId) -> Self
  {
    match value
    {
      ExpressionType::Edge => Self {
        content: VariableContent::Edge(ast::EdgePattern {
          variable: None,
          labels: ast::LabelExpression::None,
          properties: None,
          source: ast::NodePattern {
            variable: None,
            labels: ast::LabelExpression::None,
            properties: None,
          },
          destination: ast::NodePattern {
            variable: None,
            labels: ast::LabelExpression::None,
            properties: None,
          },
          directivity: crate::graph::EdgeDirectivity::Directed,
        }),
        variable_type: ExpressionType::Edge,
        col_id,
        is_set: false,
      },
      ExpressionType::Node => Self {
        content: VariableContent::Node(ast::NodePattern {
          variable: None,
          labels: ast::LabelExpression::None,
          properties: None,
        }),
        variable_type: ExpressionType::Node,
        col_id,
        is_set: false,
      },
      variable_type => Self {
        content: VariableContent::None,
        variable_type,
        col_id,
        is_set: false,
      },
    }
  }
  pub(crate) fn col_id(&self) -> value_table::ColId
  {
    self.col_id
  }
  pub(crate) fn mark_set(mut self) -> Self
  {
    self.is_set = true;
    self
  }
}

// __     __    _ _     _       _
// \ \   / /_ _| (_) __| | __ _| |_ ___  _ __
//  \ \ / / _` | | |/ _` |/ _` | __/ _ \| '__|
//   \ V / (_| | | | (_| | (_| | || (_) | |
//    \_/ \__,_|_|_|\__,_|\__,_|\__\___/|_|

#[derive(Debug)]
pub(crate) struct VariablesManager
{
  variables: HashMap<ast::VariableIdentifier, Variable>,
  function_manager: functions::Manager,
}

impl VariablesManager
{
  pub(crate) fn new(function_manager: &functions::Manager) -> Self
  {
    Self {
      variables: Default::default(),
      function_manager: function_manager.clone(),
    }
  }
  pub(crate) fn has_variable(&self, identifier: &Option<ast::VariableIdentifier>) -> bool
  {
    identifier
      .as_ref()
      .is_some_and(|identifier| self.variables.contains_key(identifier))
  }
  /// Get the index of the variable in the row of variables
  pub(crate) fn get_variable_index(&self, identifier: &ast::VariableIdentifier) -> Result<usize>
  {
    self
      .variables
      .get(identifier)
      .ok_or_else(|| unknown_variable_error(identifier))
      .map(|x| x.col_id)
  }
  /// Get the index of the variable in the row of variables
  pub(crate) fn get_variable_index_option(
    &self,
    identifier: &Option<ast::VariableIdentifier>,
  ) -> Result<Option<usize>>
  {
    match identifier
    {
      Some(identifier) => Ok(Some(self.get_variable_index(identifier)?)),
      None => Ok(None),
    }
  }
  pub(crate) fn variables_count(&self) -> usize
  {
    self.variables.len()
  }
  pub(crate) fn variables_iter(
    &self,
  ) -> std::collections::hash_map::Iter<'_, ast::VariableIdentifier, Variable>
  {
    self.variables.iter()
  }
  pub(crate) fn is_set_variable(&self, var_id: &Option<ast::VariableIdentifier>) -> Result<bool>
  {
    var_id.as_ref().map_or(Ok(false), |name| {
      self
        .variables
        .get(name)
        .ok_or_else(|| unknown_variable_error(name))
        .map(|x| x.is_set)
    })
  }
  /// Mark a variable as set.
  pub(crate) fn mark_variables_as_set<'a>(
    &mut self,
    var_id: impl Into<Option<&'a ast::VariableIdentifier>>,
  ) -> Result<()>
  {
    if let Some(var_id) = var_id.into()
    {
      self
        .variables
        .get_mut(var_id)
        .ok_or_else(|| unknown_variable_error(var_id))
        .map(|x| x.is_set = true)
    }
    else
    {
      Ok(())
    }
  }
  fn declare_variable(
    &mut self,
    var_id: &ast::VariableIdentifier,
    expression_type: ExpressionType,
  ) -> Result<()>
  {
    if self.variables.contains_key(var_id)
    {
      Err(
        CompileTimeError::VariableAlreadyBound {
          name: var_id.name().clone(),
        }
        .into(),
      )
    }
    else
    {
      self.variables.insert(
        var_id.clone(),
        Variable::from_expression(expression_type, self.variables.len()),
      );
      Ok(())
    }
  }
  // Validate a node variable, and if unknown, declare it
  fn validate_node(&mut self, node: &ast::NodePattern) -> Result<()>
  {
    if let Some(var_id) = &node.variable
    {
      if let Some(var) = self.variables.get(var_id)
      {
        match var.variable_type
        {
          ExpressionType::Node => match &var.content
          {
            VariableContent::Node(var_node) =>
            {
              if (!node.labels.is_none() || node.properties.is_some())
                && (node.labels != var_node.labels || node.properties != var_node.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_id.name().clone(),
                  }
                  .into(),
                )
              }
              else
              {
                Ok(())
              }
            }
            _ => Err(
              InternalError::ExpectedNode {
                context: "validate_node",
              }
              .into(),
            ),
          },
          ExpressionType::Variant => Ok(()), // Cannot be checked at compile time
          _ => Err(
            CompileTimeError::VariableTypeConflict {
              name: var_id.name().to_owned(),
            }
            .into(),
          ),
        }
      }
      else
      {
        if let Some(props) = &node.properties
        {
          expression_analyser::Analyser::new(self, &self.function_manager).analyse(props)?;
        }
        self.variables.insert(
          var_id.clone(),
          Variable::from_node((*node).to_owned(), self.variables.len()),
        );
        Ok(())
      }
    }
    else
    {
      Ok(())
    }
  }
  fn validate_edge(&mut self, edge: &ast::EdgePattern) -> Result<()>
  {
    self.validate_node(&edge.source)?;
    self.validate_node(&edge.destination)?;
    if let Some(var_id) = &edge.variable
    {
      if let Some(var) = self.variables.get(var_id)
      {
        match var.content
        {
          VariableContent::Edge { .. } => Err(
            CompileTimeError::VariableAlreadyBound {
              name: var_id.name().clone(),
            }
            .into(),
          ),
          _ => Err(
            CompileTimeError::VariableTypeConflict {
              name: var_id.name().clone(),
            }
            .into(),
          ),
        }
      }
      else
      {
        if let Some(props) = &edge.properties
        {
          expression_analyser::Analyser::new(self, &self.function_manager).analyse(props)?;
        }
        self.variables.insert(
          var_id.clone(),
          Variable::from_edge((*edge).to_owned(), self.variables.len()),
        );
        Ok(())
      }
    }
    else
    {
      Ok(())
    }
  }
  /// Check if the node variable exists, and that it is a node and that the definition
  /// is compatible.
  fn is_valid_existing_node(&self, node: &ast::NodePattern) -> Result<bool>
  {
    if let Some(var_id) = &node.variable
    {
      if let Some(var) = self.variables.get(var_id)
      {
        match var.variable_type
        {
          ExpressionType::Node => match &var.content
          {
            VariableContent::Node(var_node) =>
            {
              if (!node.labels.is_none() || node.properties.is_some())
                && (node.labels != var_node.labels || node.properties != var_node.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_id.name().clone(),
                  }
                  .into(),
                )
              }
              else
              {
                Ok(true)
              }
            }
            _ => Err(
              InternalError::ExpectedNode {
                context: "is_valid_existing_node",
              }
              .into(),
            ),
          },
          ExpressionType::Variant => Ok(true), // Cannot be checked at compile time
          _ => Err(
            CompileTimeError::VariableTypeConflict {
              name: var_id.name().clone(),
            }
            .into(),
          ),
        }
      }
      else
      {
        Ok(false)
      }
    }
    else
    {
      Ok(false)
    }
  }
  /// Check if the edge variable exists, and that it is a edge and that the definition
  /// is compatible.
  fn is_valid_existing_edge(&self, edge: &ast::EdgePattern) -> Result<bool>
  {
    if let Some(var_id) = &edge.variable
    {
      if let Some(var) = self.variables.get(var_id)
      {
        match var.variable_type
        {
          ExpressionType::Edge => match &var.content
          {
            VariableContent::Edge(var_edge) =>
            {
              if (!edge.labels.is_none() || edge.properties.is_some())
                && (var_edge.labels != edge.labels || var_edge.properties != edge.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_id.name().clone(),
                  }
                  .into(),
                )
              }
              else
              {
                Ok(true)
              }
            }
            _ => Err(
              InternalError::ExpectedEdge {
                context: "is_valid_existing_edge",
              }
              .into(),
            ),
          },
          ExpressionType::Variant => Ok(true), // Cannot be checked at compile time
          _ => Err(
            CompileTimeError::VariableTypeConflict {
              name: var_id.name().clone(),
            }
            .into(),
          ),
        }
      }
      else
      {
        Ok(false)
      }
    }
    else
    {
      Ok(false)
    }
  }

  /// Expression type for the given expression type
  pub(crate) fn expression_type(
    &self,
    identifier: &ast::VariableIdentifier,
  ) -> Result<ExpressionType>
  {
    Ok(
      self
        .variables
        .get(identifier)
        .ok_or_else(|| CompileTimeError::UndefinedVariable {
          name: identifier.name().to_owned(),
        })?
        .variable_type,
    )
  }

  fn analyse_edge_path(
    &mut self,
    path_variable: Option<ast::VariableIdentifier>,
    edge: &crate::parser::ast::EdgePattern,
    is_create: bool,
  ) -> Result<()>
  {
    if let Some(path_variable) = &path_variable
    {
      self.declare_variable(path_variable, expression_analyser::ExpressionType::Path)?;
    }
    if !self.is_valid_existing_node(&edge.source)?
    {
      self.validate_node(&edge.source)?;
    }
    if !self.is_valid_existing_node(&edge.destination)?
    {
      self.validate_node(&edge.destination)?;
    }
    if is_create || !self.is_valid_existing_edge(edge)?
    {
      self.validate_edge(edge)?;
    }
    Ok(())
  }

  fn analyse_pattern(&mut self, pattern: &ast::Pattern, is_create: bool) -> Result<()>
  {
    match pattern
    {
      ast::Pattern::Node(node) =>
      {
        self.validate_node(node)?;
      }
      ast::Pattern::Edge(edge) =>
      {
        self.analyse_edge_path(None, edge, is_create)?;
      }
      ast::Pattern::Path(path) =>
      {
        self.analyse_edge_path(None, &path.edge, is_create)?;
        self.declare_variable(&path.variable, ExpressionType::Path)?;
      }
    }
    Ok(())
  }
  /// Analyse a named expression and return a col id
  pub(crate) fn analyse_named_expression(
    &mut self,
    named_expression: &ast::NamedExpression,
  ) -> Result<usize>
  {
    let expression_info = expression_analyser::Analyser::new(self, &self.function_manager)
      .analyse(&named_expression.expression)?;
    let col_id = self
      .variables
      .get(&named_expression.identifier)
      .map_or(self.variables.len(), |var| var.col_id);
    self.variables.insert(
      named_expression.identifier.clone(),
      Variable::from_expression(expression_info.expression_type, col_id).mark_set(),
    );
    Ok(col_id)
  }
  pub(crate) fn keep_variables<'a>(
    &mut self,
    names: impl IntoIterator<Item = &'a ast::VariableIdentifier>,
  ) -> Result<()>
  {
    let mut new_variables = HashMap::<ast::VariableIdentifier, Variable>::default();
    for (col_id, var_id) in names.into_iter().enumerate()
    {
      let mut var = self
        .variables
        .remove(var_id)
        .ok_or_else(|| unknown_variable_error(var_id))?;
      var.col_id = col_id;
      new_variables.insert(var_id.clone(), var.mark_set());
    }
    self.variables = new_variables;
    Ok(())
  }
  pub(crate) fn analyse(&mut self, statement: &ast::Statement) -> Result<()>
  {
    if self.variables.iter().any(|(_, var)| !var.is_set)
    {
      return Err(
        InternalError::NotAllVariablesAreSet {
          set_variables: self
            .variables
            .iter()
            .filter(|(_, var)| var.is_set)
            .map(|(key, _)| key.name().clone())
            .collect(),
          all_variables: self
            .variables
            .keys()
            .map(|key| key.name().clone())
            .collect(),
        }
        .into(),
      );
    }
    match statement
    {
      ast::Statement::CreateGraph(..) =>
      {}
      ast::Statement::DropGraph(..) =>
      {}
      ast::Statement::UseGraph(..) =>
      {}
      ast::Statement::Create(create) =>
      {
        for pattern in create.patterns.iter()
        {
          if let ast::Pattern::Node(n) = &pattern
          {
            if self.has_variable(&n.variable)
            {
              return Err(
                CompileTimeError::VariableAlreadyBound {
                  name: n.variable.clone().unwrap().name().clone(),
                }
                .into(),
              );
            }
          }
          self.analyse_pattern(pattern, true)?;
        }
      }
      ast::Statement::Match(match_statement) =>
      {
        for pattern in match_statement.patterns.iter()
        {
          self.analyse_pattern(pattern, false)?;
        }
      }
      ast::Statement::Return(..) =>
      {}
      ast::Statement::Call(..) =>
      {}
      ast::Statement::With(..) =>
      {}
      ast::Statement::Unwind(unwind) =>
      {
        self.declare_variable(
          &unwind.identifier.to_owned(),
          expression_analyser::ExpressionType::Variant,
        )?;
        self.mark_variables_as_set(&unwind.identifier)?;
      }
      ast::Statement::Delete(..) =>
      {}
      ast::Statement::Update(..) =>
      {}
    }
    Ok(())
  }
}
