use std::collections::HashMap;

use crate::error::{CompileTimeError, InternalError};
use crate::interpreter::expression_analyser::ExpressionType;
use crate::parser::ast;
use crate::{functions, Result};

impl crate::interpreter::expression_analyser::Variables for HashMap<String, Variable>
{
  fn expression_type(&self, name: impl Into<String>) -> Result<ExpressionType>
  {
    let name = name.into();
    Ok(
      self
        .get(&name)
        .ok_or_else(|| InternalError::UnknownVariable {
          context: "Validator/evaluate",
          variable: name.to_owned(),
        })?
        .variable_type,
    )
  }
}

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
}

impl From<ast::NodePattern> for Variable
{
  fn from(value: ast::NodePattern) -> Self
  {
    Self {
      content: VariableContent::Node(value),
      variable_type: ExpressionType::Node,
    }
  }
}

impl From<ast::EdgePattern> for Variable
{
  fn from(value: ast::EdgePattern) -> Self
  {
    Self {
      content: VariableContent::Edge(value),
      variable_type: ExpressionType::Edge,
    }
  }
}

impl From<ExpressionType> for Variable
{
  fn from(value: ExpressionType) -> Self
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
      },
      ExpressionType::Node => Self {
        content: VariableContent::Node(ast::NodePattern {
          variable: None,
          labels: ast::LabelExpression::None,
          properties: None,
        }),
        variable_type: ExpressionType::Node,
      },
      variable_type => Self {
        content: VariableContent::None,
        variable_type,
      },
    }
  }
}

// __     __    _ _     _       _
// \ \   / /_ _| (_) __| | __ _| |_ ___  _ __
//  \ \ / / _` | | |/ _` |/ _` | __/ _ \| '__|
//   \ V / (_| | | | (_| | (_| | || (_) | |
//    \_/ \__,_|_|_|\__,_|\__,_|\__\___/|_|

#[derive(Debug)]
pub(crate) struct Validator
{
  variables: HashMap<String, Variable>,
  function_manager: functions::Manager,
}

impl Validator
{
  pub(crate) fn new(function_manager: functions::Manager) -> Self
  {
    Self {
      variables: Default::default(),
      function_manager,
    }
  }
  pub(crate) fn to_variables(&self) -> HashMap<String, Variable>
  {
    self.variables.to_owned()
  }
  pub(crate) fn variables_ref(&self) -> &HashMap<String, Variable>
  {
    &self.variables
  }
  pub(crate) fn set_variables(&mut self, variables: HashMap<String, Variable>)
  {
    self.variables = variables;
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
      self.variables.insert(variable, expression_type.into());
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
        self
          .variables
          .insert(var_name.to_owned(), (*node).to_owned().into());
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
        self
          .variables
          .insert(var_name.to_owned(), (*edge).to_owned().into());
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
  pub(crate) fn check_unexisting_variable(&self, var_name: &Option<String>) -> Result<()>
  {
    if let Some(var_name) = &var_name
    {
      if self.variables.contains_key(var_name)
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
    else
    {
      Ok(())
    }
  }
}
