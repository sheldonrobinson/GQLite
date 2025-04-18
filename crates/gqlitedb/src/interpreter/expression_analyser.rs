use crate::{functions, graph, parser::ast, Result};

// __     __         _       _     _     _____
// \ \   / /_ _ _ __(_) __ _| |__ | | __|_   _|   _ _ __   ___
//  \ \ / / _` | '__| |/ _` | '_ \| |/ _ \| || | | | '_ \ / _ \
//   \ V / (_| | |  | | (_| | |_) | |  __/| || |_| | |_) |  __/
//    \_/ \__,_|_|  |_|\__,_|_.__/|_|\___||_| \__, | .__/ \___|
//                                            |___/|_|

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ExpressionType
{
  Array,
  Map,
  Node,
  Edge,
  Boolean,
  Null,
  Integer,
  Float,
  Path,
  String,
  Variant,
}

#[derive(Debug)]
pub(crate) struct ExpressionInfo
{
  pub(crate) expression_type: ExpressionType,
  pub(crate) constant: bool,
  pub(crate) aggregation_result: bool,
}

pub(crate) trait Variables
{
  fn expression_type(&self, name: impl Into<String>) -> Result<ExpressionType>;
}

mod validators
{
  use crate::{error, Result};

  use super::{ExpressionInfo, ExpressionType};

  pub(super) fn any(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    Ok(x)
  }
  pub(super) fn boolean_or_null(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Boolean | ExpressionType::Null | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
}

impl ExpressionInfo
{
  fn new(expression_type: ExpressionType, constant: bool, aggregation_result: bool) -> Self
  {
    Self {
      expression_type,
      constant,
      aggregation_result,
    }
  }
  fn new_type(expression_type: ExpressionType, dependents: impl Into<Vec<ExpressionInfo>>) -> Self
  {
    let dependents = dependents.into();
    Self {
      expression_type,
      constant: dependents.iter().all(|x| x.constant == true),
      aggregation_result: dependents.into_iter().any(|x| x.aggregation_result == true),
    }
  }
  fn analyses<'a>(
    variables: &impl Variables,
    function_manager: &functions::Manager,
    expressions: impl Iterator<Item = &'a ast::Expression>,
    validator: impl Fn(ExpressionInfo) -> Result<ExpressionInfo>,
  ) -> Result<Vec<ExpressionInfo>>
  {
    expressions
      .map(|x| validator(Self::analyse(variables, function_manager, x)?))
      .collect::<Result<_>>()
  }

  pub(crate) fn analyse(
    variables: &impl Variables,
    function_manager: &functions::Manager,
    expression: &ast::Expression,
  ) -> Result<ExpressionInfo>
  {
    match expression
    {
      ast::Expression::Array(arr) => Ok(Self::new_type(
        ExpressionType::Array,
        Self::analyses(
          variables,
          function_manager,
          arr.array.iter(),
          validators::any,
        )?,
      )),
      ast::Expression::FunctionCall(call) =>
      {
        let arguments = Self::analyses(
          variables,
          function_manager,
          call.arguments.iter(),
          validators::any,
        )?;
        Ok(Self::new(
          function_manager.validate_arguments(
            &call.name,
            arguments.iter().map(|x| x.expression_type).collect(),
          )?,
          function_manager.is_deterministic(&call.name)?
            && arguments.iter().all(|x| x.constant == true),
          function_manager.is_aggregate(&call.name),
        ))
      }
      ast::Expression::IsNull(isn) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&isn.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::IsNotNull(isn) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&isn.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Negation(ln) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ln.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::LogicalNegation(ln) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&ln.value].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalAnd(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalOr(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalXor(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::RelationalEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalDifferent(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalInferior(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalSuperior(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalInferiorEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalSuperiorEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalIn(ri) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalNotIn(ri) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Addition(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Multiplication(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Substraction(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Division(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Modulo(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables,
          function_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Map(map) => Ok(Self::new_type(
        ExpressionType::Map,
        Self::analyses(
          variables,
          function_manager,
          map.map.values(),
          validators::any,
        )?,
      )),
      ast::Expression::MemberAccess(ma) => Ok(Self::new_type(
        ExpressionType::Variant,
        [Self::analyse(variables, function_manager, &ma.left)?],
      )),
      ast::Expression::Parameter(_) => Ok(Self::new(ExpressionType::Variant, true, false)),
      ast::Expression::Value(val) => Ok(Self::new(
        match val.value
        {
          graph::Value::Array(_) => ExpressionType::Array,
          graph::Value::Boolean(_) => ExpressionType::Boolean,
          graph::Value::Edge(_) => ExpressionType::Edge,
          graph::Value::Node(_) => ExpressionType::Node,
          graph::Value::Float(_) => ExpressionType::Float,
          graph::Value::Integer(_) => ExpressionType::Integer,
          graph::Value::Invalid => ExpressionType::Null,
          graph::Value::Object(_) => ExpressionType::Map,
          graph::Value::Path(_) => ExpressionType::Path,
          graph::Value::String(_) => ExpressionType::String,
        },
        true,
        false,
      )),
      ast::Expression::Variable(var) => Ok(ExpressionInfo::new(
        variables.expression_type(&var.identifier)?,
        false,
        false,
      )),
    }
  }
}
