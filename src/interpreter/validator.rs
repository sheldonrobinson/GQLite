use std::collections::HashMap;

use crate::error::{CompileTimeError, InternalError};
use crate::parser::ast;
use crate::Result;

// __     __         _       _     _     _____
// \ \   / /_ _ _ __(_) __ _| |__ | | __|_   _|   _ _ __   ___
//  \ \ / / _` | '__| |/ _` | '_ \| |/ _ \| || | | | '_ \ / _ \
//   \ V / (_| | |  | | (_| | |_) | |  __/| || |_| | |_) |  __/
//    \_/ \__,_|_|  |_|\__,_|_.__/|_|\___||_| \__, | .__/ \___|
//                                            |___/|_|

#[derive(Debug, PartialEq)]
#[allow(unused)]
pub(crate) enum VariableType
{
  Node,
  Edge,
  Number,
  String,
  Variant,
}

// __     __         _       _     _
// \ \   / /_ _ _ __(_) __ _| |__ | | ___
//  \ \ / / _` | '__| |/ _` | '_ \| |/ _ \
//   \ V / (_| | |  | | (_| | |_) | |  __/
//    \_/ \__,_|_|  |_|\__,_|_.__/|_|\___|

#[derive(Debug, Clone)]
#[allow(unused)]
pub(crate) enum Variable
{
  Node
  {
    node: ast::NodePattern,
  },
  Edge
  {
    edge: ast::EdgePattern,
  },
  Number,
  String,
  Variant,
}

impl Variable
{
  fn as_type(&self) -> VariableType
  {
    match self
    {
      Variable::Node { node: _ } => VariableType::Node,
      Variable::Edge { edge: _ } => VariableType::Edge,
      Variable::Number => VariableType::Number,
      Variable::String => VariableType::String,
      Variable::Variant => VariableType::Variant,
    }
  }
}

impl From<ast::NodePattern> for Variable
{
  fn from(value: ast::NodePattern) -> Self
  {
    Self::Node { node: value }
  }
}

impl From<ast::EdgePattern> for Variable
{
  fn from(value: ast::EdgePattern) -> Self
  {
    Self::Edge { edge: value }
  }
}

// __     __    _ _     _       _
// \ \   / /_ _| (_) __| | __ _| |_ ___  _ __
//  \ \ / / _` | | |/ _` |/ _` | __/ _ \| '__|
//   \ V / (_| | | | (_| | (_| | || (_) | |
//    \_/ \__,_|_|_|\__,_|\__,_|\__\___/|_|

#[derive(Debug, Default)]
pub(crate) struct Validator
{
  variables: HashMap<String, Variable>,
}

impl Validator
{
  pub(crate) fn to_variables(&self) -> HashMap<String, Variable>
  {
    self.variables.to_owned()
  }
  pub(crate) fn set_variables(&mut self, variables: HashMap<String, Variable>)
  {
    self.variables = variables;
  }
  pub(crate) fn evaluate(&self, expression: &ast::Expression) -> Result<Variable>
  {
    match expression
    {
      ast::Expression::Array(_) => Ok(Variable::Variant),
      ast::Expression::FunctionCall(_) =>
      {
        todo!()
      } // Ok(Variable::Variant),
      ast::Expression::Map(_) => Ok(Variable::Variant),
      ast::Expression::MemberAccess(_) => Ok(Variable::Variant),
      ast::Expression::Parameter(_) => Ok(Variable::Variant),
      ast::Expression::Value(_) => Ok(Variable::Variant),
      ast::Expression::Variable(var) =>
      {
        let v =
          self
            .variables
            .get(&var.identifier)
            .ok_or_else(|| InternalError::UnknownVariable {
              context: "Validator/evaluate",
              variable: var.identifier.to_owned(),
            })?;
        Ok((*v).to_owned())
      }
    }
  }
  pub(crate) fn declare_edge_variable(&mut self, edge: &ast::EdgePattern) -> Result<()>
  {
    if let Some(var_name) = &edge.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var
        {
          Variable::Edge { .. } => Err(
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
  pub(crate) fn declare_node_variable(&mut self, node: &ast::NodePattern) -> Result<()>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        match var
        {
          Variable::Node { .. } => Err(
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
          .insert(var_name.to_owned(), (*node).to_owned().into());
        Ok(())
      }
    }
    else
    {
      Ok(())
    }
  }
  pub(crate) fn check_node_variable(&self, node: &ast::NodePattern) -> Result<()>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        return match var
        {
          Variable::Node { node: var_node } =>
          {
            if (!node.labels.is_none() || !node.properties.is_none())
              && (var_node.labels != node.labels || var_node.properties != node.properties)
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
            CompileTimeError::VariableTypeConflict {
              name: var_name.to_owned(),
            }
            .into(),
          ),
        };
      }
    }
    // if let Some(var_name) = &node.variable
    // {
    //   if self.existing_variable(&node.variable, Some(VariableType::Node))?
    //   {
    //     let var = self.variables.get(var_name).unwrap();
    //     let VariableInformation::NodePattern { node } = var.information;
    //   }
    // }
    Ok(())
  }

  pub(crate) fn check_existing_node(&self, node: &ast::NodePattern) -> Result<bool>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(_) = self.variables.get(var_name)
      {
        self.check_node_variable(node)?;
        Ok(true)
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
}
