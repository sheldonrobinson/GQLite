//! Module for building expressions.

use crate::Expression;

/// Convert arguments for function calls
pub trait IntoFunctionArgs
{
  /// Convert self to arguments, aka, vector of expressions.
  fn into_args(self) -> Vec<Expression>;
}

impl IntoFunctionArgs for ()
{
  fn into_args(self) -> Vec<Expression>
  {
    Default::default()
  }
}

impl<T0> IntoFunctionArgs for (T0,)
where
  Expression: From<T0>,
{
  fn into_args(self) -> Vec<Expression>
  {
    vec![self.0.into()]
  }
}

macro_rules! binop {
  ($name:ident, $op:ident) => {
    /// Create a binary $name operation
    pub fn $name(lhs: Expression, rhs: Expression) -> Expression
    {
      Expression::BinaryOp {
        operator: super::BinOp::$op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
      }
    }
  };
}

binop!(equal, Equal);
binop!(different, Different);
binop!(superior, Superior);
binop!(superior_equal, SuperiorEqual);
binop!(inferior, Inferior);
binop!(inferior_equal, InferiorEqual);
binop!(and, And);
binop!(or, Or);
binop!(addition, Addition);
binop!(substraction, Substraction);
binop!(division, Division);
binop!(multiplication, Multiplication);

/// Return a none expression
pub fn none() -> Option<Expression>
{
  None
}

/// Create a function call
pub fn function_call(name: impl AsRef<str>, args: impl IntoFunctionArgs) -> Expression
{
  Expression::FunctionCall {
    name: name.as_ref().to_string(),
    arguments: args.into_args(),
  }
}

/// Access a member of a dictionnary
pub fn get(expression: Expression, path: Vec<String>) -> Expression
{
  let expression = Box::new(expression);
  Expression::Get { expression, path }
}
