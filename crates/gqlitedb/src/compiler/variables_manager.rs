use std::collections::HashMap;

use crate::prelude::*;

use compiler::expression_analyser::{ExpressionInfo, ExpressionType};
use parser::ast;

// __     __         _       _     _
// \ \   / /_ _ _ __(_) __ _| |__ | | ___
//  \ \ / / _` | '__| |/ _` | '_ \| |/ _ \
//   \ V / (_| | |  | | (_| | |_) | |  __/
//    \_/ \__,_|_|  |_|\__,_|_.__/|_|\___|

#[derive(Debug, Clone)]
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
}

impl Variable
{
  fn from_node(value: ast::NodePattern, col_id: value_table::ColId) -> Self
  {
    Self {
      content: VariableContent::Node(value),
      variable_type: ExpressionType::Node,
      col_id,
    }
  }
  fn from_edge(value: ast::EdgePattern, col_id: value_table::ColId) -> Self
  {
    Self {
      content: VariableContent::Edge(value),
      variable_type: ExpressionType::Edge,
      col_id,
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
      },
      ExpressionType::Node => Self {
        content: VariableContent::Node(ast::NodePattern {
          variable: None,
          labels: ast::LabelExpression::None,
          properties: None,
        }),
        variable_type: ExpressionType::Node,
        col_id,
      },
      variable_type => Self {
        content: VariableContent::None,
        variable_type,
        col_id,
      },
    }
  }
  pub(crate) fn col_id(&self) -> value_table::ColId
  {
    self.col_id
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
  variables: HashMap<String, Variable>,
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

  /// Get the index of the variable in the row of variables
  pub(crate) fn get_variable_index(&self, identifier: &String) -> Result<usize>
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
      .map(|x| x.col_id)
  }
  /// Get the index of the variable in the row of variables
  pub(crate) fn get_variable_index_option(
    &self,
    identifier: &Option<String>,
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
  pub(crate) fn variables_iter(&self) -> std::collections::hash_map::Iter<'_, String, Variable>
  {
    self.variables.iter()
  }
  pub(crate) fn declare_variable(
    &mut self,
    variable: impl Into<String>,
    expression_type: ExpressionType,
  ) -> Result<()>
  {
    let variable = variable.into();
    if self.variables.contains_key(&variable)
    {
      Err(
        CompileTimeError::VariableAlreadyBound {
          name: variable.to_owned(),
        }
        .into(),
      )
    }
    else
    {
      self.variables.insert(
        variable,
        Variable::from_expression(expression_type, self.variables.len()),
      );
      Ok(())
    }
  }
  // Validate a node variable, and if unknown, declare it
  pub(crate) fn validate_node(&mut self, node: &ast::NodePattern) -> Result<()>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var.variable_type
        {
          ExpressionType::Node => match &var.content
          {
            VariableContent::Node(var_node) =>
            {
              if (!node.labels.is_none() || !node.properties.is_none())
                && (node.labels != var_node.labels || node.properties != var_node.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_name.to_owned(),
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
              name: var_name.to_owned(),
            }
            .into(),
          ),
        }
      }
      else
      {
        if let Some(props) = &node.properties
        {
          ExpressionInfo::analyse(&self, &self.function_manager, &props)?;
        }
        self.variables.insert(
          var_name.to_owned(),
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
  pub(crate) fn validate_edge(&mut self, edge: &ast::EdgePattern) -> Result<()>
  {
    self.validate_node(&edge.source)?;
    self.validate_node(&edge.destination)?;
    if let Some(var_name) = &edge.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var.content
        {
          VariableContent::Edge { .. } => Err(
            CompileTimeError::VariableAlreadyBound {
              name: var_name.to_owned(),
            }
            .into(),
          ),
          _ => Err(
            CompileTimeError::VariableTypeConflict {
              name: var_name.to_owned(),
            }
            .into(),
          ),
        }
      }
      else
      {
        if let Some(props) = &edge.properties
        {
          ExpressionInfo::analyse(self, &self.function_manager, &props)?;
        }
        self.variables.insert(
          var_name.to_owned(),
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
  pub(crate) fn is_valid_existing_node(&self, node: &ast::NodePattern) -> Result<bool>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var.variable_type
        {
          ExpressionType::Node => match &var.content
          {
            VariableContent::Node(var_node) =>
            {
              if (!node.labels.is_none() || !node.properties.is_none())
                && (node.labels != var_node.labels || node.properties != var_node.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_name.to_owned(),
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
              name: var_name.to_owned(),
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
  pub(crate) fn is_valid_existing_edge(&self, edge: &ast::EdgePattern) -> Result<bool>
  {
    if let Some(var_name) = &edge.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var.variable_type
        {
          ExpressionType::Edge => match &var.content
          {
            VariableContent::Edge(var_edge) =>
            {
              if (!edge.labels.is_none() || !edge.properties.is_none())
                && (var_edge.labels != edge.labels || var_edge.properties != edge.properties)
              {
                Err(
                  CompileTimeError::VariableAlreadyBound {
                    name: var_name.to_owned(),
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
              name: var_name.to_owned(),
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

  pub(crate) fn expression_type(&self, name: impl Into<String>) -> Result<ExpressionType>
  {
    let name = name.into();
    Ok(
      self
        .variables
        .get(&name)
        .ok_or_else(|| CompileTimeError::UndefinedVariable {
          name: name.to_owned(),
        })?
        .variable_type,
    )
  }
}
