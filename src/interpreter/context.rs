use std::borrow::Borrow;
use std::collections::HashMap;

use crate::parser::ast;
use crate::Error;
use crate::Result;

#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
pub(crate) enum VariableType
{
  Node,
  Edge,
  Number,
  String,
  Variant,
}

#[derive(Debug, Default)]
pub(crate) struct Context
{
  variables: HashMap<String, VariableType>,
  new_variables: Vec<String>,
}

impl Context
{
  pub(crate) fn existing_variable(
    &self,
    var_name: &Option<String>,
    expected_var_type: Option<VariableType>,
  ) -> Result<bool>
  {
    if let Some(var_name) = var_name
    {
      if let Some(var_type) = self.variables.get(var_name)
      {
        if let Some(expected_var_type) = expected_var_type
        {
          if *var_type == expected_var_type
          {
            Ok(true)
          }
          else
          {
            Err(Error::Unknown("Unexpected type, expected node."))
          }
        }
        else
        {
          Ok(true)
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

  pub(crate) fn check_variable_type(
    &mut self,
    var_name: &String,
    expected_var_type: VariableType,
    allow_existing: bool,
  ) -> Result<()>
  {
    if let Some(var_type) = self.variables.get(var_name)
    {
      if allow_existing
      {
        if *var_type != expected_var_type
        {
          return Err(Error::Unknown("Unexpected type, expected node."));
        }
      }
      else
      {
        return Err(Error::Unknown("Variable is already defined"));
      }
    }
    else
    {
      self
        .variables
        .insert(var_name.to_owned(), expected_var_type);
      self.new_variables.push(var_name.to_owned());
    }
    Ok(())
  }

  pub(crate) fn check_graph_node(
    &mut self,
    graph_node: &ast::GraphNode,
    allow_existing: bool,
  ) -> Result<()>
  {
    if let Some(var_name) = graph_node.variable.borrow()
    {
      self.check_variable_type(var_name, VariableType::Node, allow_existing)?;
    }
    Ok(())
  }
}
// pub(crate) fn build_context(
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
// }
