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
  Key,
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

//             _ _     _       _
// __   ____ _| (_) __| | __ _| |_ ___  _ __ ___
// \ \ / / _` | | |/ _` |/ _` | __/ _ \| '__/ __|
//  \ V / (_| | | | (_| | (_| | || (_) | |  \__ \
//   \_/ \__,_|_|_|\__,_|\__,_|\__\___/|_|  |___/
//

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

//  _____                              _                _                _
// | ____|_  ___ __  _ __ ___  ___ ___(_) ___  _ __    / \   _ __   __ _| |_   _ ___  ___ _ __
// |  _| \ \/ / '_ \| '__/ _ \/ __/ __| |/ _ \| '_ \  / _ \ | '_ \ / _` | | | | / __|/ _ \ '__|
// | |___ >  <| |_) | | |  __/\__ \__ \ | (_) | | | |/ ___ \| | | | (_| | | |_| \__ \  __/ |
// |_____/_/\_\ .__/|_|  \___||___/___/_|\___/|_| |_/_/   \_\_| |_|\__,_|_|\__, |___/\___|_|
//            |_|                                                          |___/

trait ExpressionAnalyser
{
  fn analyse<'b>(self, analyser: &Analyser<'b>) -> Result<Vec<ExpressionInfo>>;
}

impl<'a, T, VT> ExpressionAnalyser for (T, VT)
where
  T: Iterator<Item = &'a ast::Expression>,
  VT: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse<'b>(self, analyser: &Analyser<'b>) -> Result<Vec<ExpressionInfo>>
  {
    let mut rv: Vec<ExpressionInfo> = Default::default();
    for x in self.0
    {
      rv.push(self.1(analyser.analyse(x)?)?);
    }
    Ok(rv)
  }
}

impl<VT> ExpressionAnalyser for (&ast::Expression, VT)
where
  VT: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse<'b>(self, analyser: &Analyser<'b>) -> Result<Vec<ExpressionInfo>>
  {
    Ok(vec![self.1(analyser.analyse(self.0)?)?])
  }
}

impl<'a, VT0, VT1> ExpressionAnalyser for ((&'a ast::Expression, &'a ast::Expression), (VT0, VT1))
where
  VT0: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
  VT1: Fn(ExpressionInfo) -> Result<ExpressionInfo>,
{
  fn analyse<'b>(self, analyser: &Analyser<'b>) -> Result<Vec<ExpressionInfo>>
  {
    Ok(vec![
      self.1 .0(analyser.analyse(self.0 .0)?)?,
      self.1 .1(analyser.analyse(self.0 .1)?)?,
    ])
  }
}

//  _____                              _             ___        __
// | ____|_  ___ __  _ __ ___  ___ ___(_) ___  _ __ |_ _|_ __  / _| ___
// |  _| \ \/ / '_ \| '__/ _ \/ __/ __| |/ _ \| '_ \ | || '_ \| |_ / _ \
// | |___ >  <| |_) | | |  __/\__ \__ \ | (_) | | | || || | | |  _| (_) |
// |_____/_/\_\ .__/|_|  \___||___/___/_|\___/|_| |_|___|_| |_|_|  \___/
//            |_|

#[derive(Debug)]
pub(crate) struct ExpressionInfo
{
  pub(crate) expression_type: ExpressionType,
  pub(crate) constant: bool,
  pub(crate) aggregation_result: bool,
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
      constant: dependents.iter().all(|x| x.constant),
      aggregation_result: dependents.into_iter().any(|x| x.aggregation_result),
    }
  }
}

//     _                _
//    / \   _ __   __ _| |_   _ ___  ___ _ __
//   / _ \ | '_ \ / _` | | | | / __|/ _ \ '__|
//  / ___ \| | | | (_| | | |_| \__ \  __/ |
// /_/   \_\_| |_|\__,_|_|\__, |___/\___|_|
//                        |___/

pub(crate) struct Analyser<'b>
{
  pub(crate) variables_manager: &'b VariablesManager,
  pub(crate) functions_manager: &'b functions::Manager,
}

impl<'b> Analyser<'b>
{
  pub(crate) fn new(
    variables_manager: &'b VariablesManager,
    functions_manager: &'b functions::Manager,
  ) -> Self
  {
    Self {
      variables_manager,
      functions_manager,
    }
  }
  fn analyses_in<'a>(
    &self,
    left_expr: &'a ast::Expression,
    right_expr: &'a ast::Expression,
  ) -> Result<Vec<ExpressionInfo>>
  {
    let mut left_expr = self.analyse(left_expr)?;
    let right_expr = self.analyse(right_expr)?;
    let right_expr = validators::array_or_map(right_expr)?;

    if right_expr.expression_type == ExpressionType::Map
    {
      left_expr = validators::string_or_null(left_expr)?;
    }

    Ok(vec![left_expr, right_expr])
  }
  pub(crate) fn analyse(&self, expression: &ast::Expression) -> Result<ExpressionInfo>
  {
    match expression
    {
      ast::Expression::Array(arr) => Ok(ExpressionInfo::new_type(
        ExpressionType::Array,
        (arr.array.iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::FunctionCall(call) =>
      {
        let arguments = (call.arguments.iter(), validators::any).analyse(self)?;
        Ok(ExpressionInfo::new(
          self.functions_manager.validate_arguments(
            &call.name,
            arguments.iter().map(|x| x.expression_type).collect(),
          )?,
          self.functions_manager.is_deterministic(&call.name)?
            && arguments.iter().all(|x| x.constant),
          self.functions_manager.is_aggregate(&call.name)?,
        ))
      }
      ast::Expression::IsNull(isn) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&isn.value].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::IsNotNull(isn) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&isn.value].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Negation(ln) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ln.value].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::LogicalNegation(ln) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&ln.value].into_iter(), validators::boolean_or_null).analyse(self)?,
      )),
      ast::Expression::LogicalAnd(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        (
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )
          .analyse(self)?,
      )),
      ast::Expression::LogicalOr(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        (
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )
          .analyse(self)?,
      )),
      ast::Expression::LogicalXor(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        (
          [&rd.left, &rd.right].into_iter(),
          validators::boolean_or_null,
        )
          .analyse(self)?,
      )),
      ast::Expression::RelationalEqual(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalDifferent(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalInferior(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalSuperior(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalInferiorEqual(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalSuperiorEqual(rd) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        ([&rd.left, &rd.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::RelationalIn(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        self.analyses_in(&ri.left, &ri.right)?,
      )),
      ast::Expression::RelationalNotIn(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Boolean,
        self.analyses_in(&ri.left, &ri.right)?,
      )),
      ast::Expression::Addition(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Multiplication(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Subtraction(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Division(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Modulo(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Exponent(ri) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        ([&ri.left, &ri.right].into_iter(), validators::any).analyse(self)?,
      )),
      ast::Expression::Map(map) => Ok(ExpressionInfo::new_type(
        ExpressionType::Map,
        (map.map.iter().map(|(_, v)| v), validators::any).analyse(self)?,
      )),
      ast::Expression::MemberAccess(ma) => Ok(ExpressionInfo::new_type(
        ExpressionType::Variant,
        [validators::map_or_edge_or_node_or_null(
          self.analyse(&ma.left)?,
        )?],
      )),
      ast::Expression::IndexAccess(ia) => Ok({
        let left = self.analyse(&ia.left)?;
        let index = self.analyse(&ia.index)?;

        match left.expression_type
        {
          ExpressionType::Array => ExpressionInfo::new_type(
            ExpressionType::Variant,
            [left, validators::integer_or_null(index)?],
          ),
          ExpressionType::Map | ExpressionType::Edge | ExpressionType::Node =>
          {
            ExpressionInfo::new_type(
              ExpressionType::Variant,
              [left, validators::string_or_null(index)?],
            )
          }
          ExpressionType::Null => ExpressionInfo::new_type(ExpressionType::Null, [left, index]),
          ExpressionType::Variant => ExpressionInfo::new_type(
            ExpressionType::Variant,
            [left, validators::integer_or_string_or_null(index)?],
          ),
          _ => Err(crate::error::CompileTimeError::InvalidArgumentType)?,
        }
      }),
      ast::Expression::RangeAccess(ia) => Ok({
        let mut dependents = vec![validators::array(self.analyse(&ia.left)?)?];
        if let Some(start) = &ia.start
        {
          dependents.push(validators::integer_or_null(self.analyse(start)?)?);
        }
        if let Some(end) = &ia.end
        {
          dependents.push(validators::integer_or_null(self.analyse(end)?)?);
        }
        ExpressionInfo::new_type(ExpressionType::Variant, dependents)
      }),
      ast::Expression::Parameter(_) =>
      {
        Ok(ExpressionInfo::new(ExpressionType::Variant, true, false))
      }
      ast::Expression::Value(val) => Ok(ExpressionInfo::new(
        match val.value
        {
          value::Value::Array(_) => ExpressionType::Array,
          value::Value::Key(_) => ExpressionType::Key,
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
        self.variables_manager.expression_type(&var.identifier)?,
        false,
        false,
      )),
    }
  }
}
