use crate::{compiler::variables_manager::VariablesManager, parser::ast, prelude::*};

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
  pub(super) fn array(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Array | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
  pub(super) fn array_or_map(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Array | ExpressionType::Map | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
  pub(super) fn map_or_edge_or_node_or_null(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Map
      | ExpressionType::Edge
      | ExpressionType::Node
      | ExpressionType::Variant
      | ExpressionType::Null => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }

  pub(super) fn integer_or_null(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Integer | ExpressionType::Null | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
  pub(super) fn string_or_null(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::String | ExpressionType::Null | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
  pub(super) fn integer_or_string_or_null(x: ExpressionInfo) -> Result<ExpressionInfo>
  {
    match x.expression_type
    {
      ExpressionType::Integer
      | ExpressionType::String
      | ExpressionType::Null
      | ExpressionType::Variant => Ok(x),
      _ => Err(error::CompileTimeError::InvalidArgumentType.into()),
    }
  }
}

trait ExpressionAnalyser
{
  fn analyse(
    self,
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
  ) -> Result<Vec<ExpressionInfo>>;
}

impl<'a, T, VT> ExpressionAnalyser for (T, VT)
where
  T: Iterator<Item = &'a ast::Expression>,
  VT: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse(
    self,
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
  ) -> Result<Vec<ExpressionInfo>>
  {
    self
      .0
      .map(|x| {
        self.1(ExpressionInfo::analyse(
          variables_manager,
          functions_manager,
          x,
        )?)
      })
      .collect::<Result<_>>()
  }
}

impl<'a, VT> ExpressionAnalyser for (&'a ast::Expression, VT)
where
  VT: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse(
    self,
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
  ) -> Result<Vec<ExpressionInfo>>
  {
    Ok(vec![self.1(ExpressionInfo::analyse(
      variables_manager,
      functions_manager,
      self.0,
    )?)?])
  }
}

impl<'a, VT0, VT1> ExpressionAnalyser for ((&'a ast::Expression, &'a ast::Expression), (VT0, VT1))
where
  VT0: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
  VT1: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse(
    self,
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
  ) -> Result<Vec<ExpressionInfo>>
  {
    Ok(vec![
      self.1 .0(ExpressionInfo::analyse(
        variables_manager,
        functions_manager,
        self.0 .0,
      )?)?,
      self.1 .1(ExpressionInfo::analyse(
        variables_manager,
        functions_manager,
        self.0 .1,
      )?)?,
    ])
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
  fn analyses<'a, TExpressions, TValidatorType>(
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
    expressions: TExpressions,
    validator: TValidatorType,
  ) -> Result<Vec<ExpressionInfo>>
  where
    (TExpressions, TValidatorType): ExpressionAnalyser,
  {
    (expressions, validator).analyse(variables_manager, functions_manager)
  }
  fn analyses_in<'a>(
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
    left_expr: &'a ast::Expression,
    right_expr: &'a ast::Expression,
  ) -> Result<Vec<ExpressionInfo>>
  {
    let mut left_expr = ExpressionInfo::analyse(variables_manager, functions_manager, left_expr)?;
    let right_expr = ExpressionInfo::analyse(variables_manager, functions_manager, right_expr)?;
    let right_expr = validators::array_or_map(right_expr)?;

    if right_expr.expression_type == ExpressionType::Map
    {
      left_expr = validators::string_or_null(left_expr)?;
    }

    Ok(vec![left_expr, right_expr])
  }
  pub(crate) fn analyse(
    variables_manager: &VariablesManager,
    functions_manager: &functions::Manager,
    expression: &ast::Expression,
  ) -> Result<ExpressionInfo>
  {
    match expression
    {
      ast::Expression::Array(arr) => Ok(Self::new_type(
        ExpressionType::Array,
        Self::analyses(
          variables_manager,
          functions_manager,
          arr.array.iter(),
          validators::any,
        )?,
      )),
      ast::Expression::FunctionCall(call) =>
      {
        let arguments = Self::analyses(
          variables_manager,
          functions_manager,
          call.arguments.iter(),
          validators::any,
        )?;
        Ok(Self::new(
          functions_manager.validate_arguments(
            &call.name,
            arguments.iter().map(|x| x.expression_type).collect(),
          )?,
          functions_manager.is_deterministic(&call.name)?
            && arguments.iter().all(|x| x.constant == true),
          functions_manager.is_aggregate(&call.name),
        ))
      }
      ast::Expression::IsNull(isn) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&isn.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::IsNotNull(isn) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&isn.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Negation(ln) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ln.value].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::LogicalNegation(ln) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ln.value].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalAnd(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalOr(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::LogicalXor(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )?,
      )),
      ast::Expression::RelationalEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalDifferent(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalInferior(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalSuperior(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalInferiorEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalSuperiorEqual(rd) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&rd.left, &rd.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::RelationalIn(ri) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses_in(variables_manager, functions_manager, &ri.left, &ri.right)?,
      )),
      ast::Expression::RelationalNotIn(ri) => Ok(Self::new_type(
        ExpressionType::Boolean,
        Self::analyses_in(variables_manager, functions_manager, &ri.left, &ri.right)?,
      )),
      ast::Expression::Addition(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Multiplication(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Subtraction(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Division(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Modulo(ri) => Ok(Self::new_type(
        ExpressionType::Variant,
        Self::analyses(
          variables_manager,
          functions_manager,
          [&ri.left, &ri.right].into_iter(),
          validators::any,
        )?,
      )),
      ast::Expression::Map(map) => Ok(Self::new_type(
        ExpressionType::Map,
        Self::analyses(
          variables_manager,
          functions_manager,
          map.map.iter().map(|(_, v)| v),
          validators::any,
        )?,
      )),
      ast::Expression::MemberAccess(ma) => Ok(Self::new_type(
        ExpressionType::Variant,
        [validators::map_or_edge_or_node_or_null(Self::analyse(
          variables_manager,
          functions_manager,
          &ma.left,
        )?)?],
      )),
      ast::Expression::IndexAccess(ia) => Ok({
        let left = Self::analyse(variables_manager, functions_manager, &ia.left)?;
        let index = Self::analyse(variables_manager, functions_manager, &ia.index)?;

        match left.expression_type
        {
          ExpressionType::Array => Self::new_type(
            ExpressionType::Variant,
            [left, validators::integer_or_null(index)?],
          ),
          ExpressionType::Map | ExpressionType::Edge | ExpressionType::Node => Self::new_type(
            ExpressionType::Variant,
            [left, validators::string_or_null(index)?],
          ),
          ExpressionType::Null => Self::new_type(ExpressionType::Null, [left, index]),
          ExpressionType::Variant => Self::new_type(
            ExpressionType::Variant,
            [left, validators::integer_or_string_or_null(index)?],
          ),
          _ => Err(crate::error::CompileTimeError::InvalidArgumentType)?,
        }
      }),
      ast::Expression::RangeAccess(ia) => Ok({
        let mut dependents = vec![validators::array(Self::analyse(
          variables_manager,
          functions_manager,
          &ia.left,
        )?)?];
        if let Some(start) = &ia.start
        {
          dependents.push(validators::integer_or_null(Self::analyse(
            variables_manager,
            functions_manager,
            start,
          )?)?);
        }
        if let Some(end) = &ia.end
        {
          dependents.push(validators::integer_or_null(Self::analyse(
            variables_manager,
            functions_manager,
            end,
          )?)?);
        }
        Self::new_type(ExpressionType::Variant, dependents)
      }),
      ast::Expression::Parameter(_) => Ok(Self::new(ExpressionType::Variant, true, false)),
      ast::Expression::Value(val) => Ok(Self::new(
        match val.value
        {
          value::Value::Array(_) => ExpressionType::Array,
          value::Value::Boolean(_) => ExpressionType::Boolean,
          value::Value::Edge(_) => ExpressionType::Edge,
          value::Value::Node(_) => ExpressionType::Node,
          value::Value::Float(_) => ExpressionType::Float,
          value::Value::Integer(_) => ExpressionType::Integer,
          value::Value::Null => ExpressionType::Null,
          value::Value::Map(_) => ExpressionType::Map,
          value::Value::Path(_) => ExpressionType::Path,
          value::Value::String(_) => ExpressionType::String,
        },
        true,
        false,
      )),
      ast::Expression::Variable(var) => Ok(ExpressionInfo::new(
        variables_manager.expression_type(&var.identifier)?,
        false,
        false,
      )),
    }
  }
}
