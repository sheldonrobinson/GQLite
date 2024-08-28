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
    node: ast::GraphNode,
  },
  Edge
  {
    edge: ast::GraphEdge,
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

impl From<ast::GraphNode> for Variable
{
  fn from(value: ast::GraphNode) -> Self
  {
    Self::Node { node: value }
  }
}

impl From<ast::GraphEdge> for Variable
{
  fn from(value: ast::GraphEdge) -> Self
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
  pub(crate) fn declare_edge_variable(&mut self, edge: &ast::GraphEdge) -> Result<()>
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
  pub(crate) fn declare_node_variable(&mut self, node: &ast::GraphNode) -> Result<()>
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
  pub(crate) fn check_node_variable(&self, node: &ast::GraphNode) -> Result<()>
  {
    if let Some(var_name) = &node.variable
    {
      if let Some(var) = self.variables.get(var_name)
      {
        return match var
        {
          Variable::Node { node: var_node } =>
          {
            if (!node.labels.is_empty() || !node.properties.is_none())
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
            CompileTimeError::VariableAlreadyBound {
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
    //     let VariableInformation::GraphNode { node } = var.information;
    //   }
    // }
    Ok(())
  }

  pub(crate) fn check_existing_node(&self, node: &ast::GraphNode) -> Result<bool>
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
  // pub(crate) fn existing_variable(
  //   &self,
  //   var_name: &Option<String>,
  //   expected_var_type: Option<VariableType>,
  // ) -> Result<bool>
  // {
  //   if let Some(var_name) = var_name
  //   {
  //     if let Some(var) = self.variables.get(var_name)
  //     {
  //       if let Some(expected_var_type) = expected_var_type
  //       {
  //         if var.as_type() == expected_var_type
  //         {
  //           Ok(true)
  //         }
  //         else
  //         {
  //           Err(Error::Unknown("Unexpected type, expected node."))
  //         }
  //       }
  //       else
  //       {
  //         Ok(true)
  //       }
  //     }
  //     else
  //     {
  //       Ok(false)
  //     }
  //   }
  //   else
  //   {
  //     Ok(false)
  //   }
  // }

  // fn check_variable_type(
  //   &mut self,
  //   var_name: &String,
  //   expected_var_type: VariableType,
  //   allow_existing: bool,
  // ) -> Result<()>
  // {
  //   if let Some(var) = self.variables.get(var_name)
  //   {
  //     if allow_existing
  //     {
  //       if var.as_type() != expected_var_type
  //       {
  //         return Err(Error::Unknown("Unexpected type, expected node."));
  //       }
  //     }
  //     else
  //     {
  //       return Err(
  //         CompileTimeError::VariableAlreadyBound {
  //           name: var_name.to_owned(),
  //         }
  //         .into(),
  //       );
  //     }
  //   }
  //   else
  //   {
  //     self
  //       .variables
  //       .insert(var_name.to_owned(), expected_var_type);
  //   }
  //   Ok(())
  // }

  //   pub(crate) fn check_graph_node(
  //     &mut self,
  //     graph_node: &ast::GraphNode,
  //     allow_existing: bool,
  //   ) -> Result<()>
  //   {
  //     if let Some(var_name) = graph_node.variable.borrow()
  //     {
  //       self.check_variable_type(var_name, VariableType::Node, allow_existing)?;
  //     }
  //     Ok(())
  //   }
  // }
  // fn build_context(
  //   statement: &ast::Statement,
  //   previous_context: Context,
  // ) -> Result<Context> {
  //   let mut context = previous_context;
  //   match statement {
  //     Statement::CreateGraph(_) => {}
  //     Statement::UseGraph(_) => {}
  //     Statement::Create(create) => {
  //       for pat in create.patterns.iter() {
  //         match pat {
  //           ast::Pattern::GraphNode(graph_node) => {
  //             check_graph_node(&mut context, graph_node, false)?;
  //           }
  //           ast::Pattern::GraphEdge(graph_edge) => {
  //             check_graph_node(&mut context, graph_edge.source.borrow(), true)?;
  //             check_graph_node(&mut context, graph_edge.destination.borrow(), true)?;
  //             if let Some(var_name) = graph_edge.variable.borrow() {
  //               check_variable_type(&mut context, var_name, VariableType::Edge, false)?;
  //             }
  //           }
  //         }
  //         // create.
  //       }
  //     }
  //     Statement::Match(match_statement) => {}
  //     Statement::Return(return_statement) => {}
  //   }
  //   Ok(context)
}
