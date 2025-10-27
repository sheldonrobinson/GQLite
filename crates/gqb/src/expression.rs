use crate::prelude::*;

pub mod expression_builder;

/// Binary operator
#[derive(Debug)]
pub enum BinOp
{
  And,
  Or,
  Equal,
  Different,
  Superior,
  SuperiorEqual,
  Inferior,
  InferiorEqual,
  Addition,
  Substraction,
  Multiplication,
  Division,
}

impl BinOp
{
  fn into_oc_query(self) -> String
  {
    match self
    {
      BinOp::And => "AND",
      BinOp::Or => "OR",
      BinOp::Equal => "=",
      BinOp::Different => "!=",
      BinOp::Superior => ">",
      BinOp::SuperiorEqual => ">=",
      BinOp::Inferior => "<",
      BinOp::InferiorEqual => "<=",
      BinOp::Addition => "+",
      BinOp::Substraction => "-",
      BinOp::Multiplication => "*",
      BinOp::Division => "/",
    }
    .to_string()
  }
}

/// Expression used in queries.
#[derive(Debug)]
pub enum Expression
{
  FunctionCall
  {
    name: String,
    arguments: Vec<Expression>,
  },
  Variable(Variable),
  Literal(Box<graphcore::Value>),
  Get {
    expression: Box<Expression>,
    path: Vec<String>,
  },
  BinaryOp
  {
    operator: BinOp,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
  },
}

impl Expression
{
  pub(crate) fn into_oc_query(self, bindings: &mut graphcore::ValueMap) -> String
  {
    match self
    {
      Expression::BinaryOp { operator, lhs, rhs } => format!(
        "({}) {} ({})",
        lhs.into_oc_query(bindings),
        operator.into_oc_query(),
        rhs.into_oc_query(bindings)
      ),
      Expression::FunctionCall { name, arguments } => format!(
        "{}({})",
        name,
        arguments
          .into_iter()
          .map(|a| a.into_oc_query(bindings))
          .collect::<Vec<_>>()
          .join(",")
      ),
      Expression::Literal(literal) =>
      {
        let properties_binding = format!("$l{}", bindings.len());
        bindings.insert(properties_binding.clone(), *literal);
        properties_binding
      }
      Expression::Variable(var) => format!("{}", var),
      Expression::Get { expression, path } => format!("{}.{}", expression.into_oc_query(bindings), path.join("."))
    }
  }
}

impl From<Variable> for Expression
{
  fn from(value: Variable) -> Self
  {
    Self::Variable(value)
  }
}

impl<T> From<T> for Expression
where
  graphcore::Value: From<T>,
{
  fn from(value: T) -> Self
  {
    Self::Literal(Box::new(value.into()))
  }
}
