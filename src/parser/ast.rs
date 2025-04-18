#![allow(unused)]

use crate::graph;

#[derive(Debug)]
pub(crate) enum Statement
{
  CreateGraph(CreateGraph),
  UseGraph(UseGraph),
  Create(Create),
  Match(Match),
  Return(Return),
  Call(Call),
  With(With),
  Unwind(Unwind),
}

pub(crate) type Statements = Vec<Statement>;

#[derive(Debug)]
pub(crate) enum Node {}

#[derive(Debug)]
pub(crate) struct CreateGraph
{
  pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct UseGraph
{
  pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct Create
{
  pub(crate) patterns: Vec<Pattern>,
}

#[derive(Debug)]
pub(crate) struct Match
{
  pub(crate) patterns: Vec<Pattern>,
  pub(crate) where_expression: Option<Expression>,
  pub(crate) optional: bool,
}

#[derive(Debug)]
pub(crate) struct Return
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
}

#[derive(Debug)]
pub(crate) struct With
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
}

#[derive(Debug)]
pub(crate) struct Unwind
{
  pub(crate) name: String,
  pub(crate) expression: Expression,
}

#[derive(Debug)]
pub(crate) struct Delete
{
  pub(crate) detach: bool,
  pub(crate) expressions: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct Set
{
  pub(crate) expressions: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct Remove
{
  pub(crate) expressions: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct Call
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
  pub(crate) yield_: Vec<String>,
}

// Set/remove Statements

#[derive(Debug)]
pub(crate) struct SetProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
  pub(crate) expression: Expression,
}

#[derive(Debug)]
pub(crate) struct AddProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
  pub(crate) expression: Expression,
}

#[derive(Debug)]
pub(crate) struct RemoveProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct EditLabels
{
  pub(crate) target: String,
  pub(crate) labels: Vec<String>,
}

// Modifiers

#[derive(Debug)]
pub(crate) struct OrderBy
{
  pub(crate) expressions: Vec<OrderByExpression>,
}

#[derive(Default, Debug)]
pub(crate) struct Modifiers
{
  pub(crate) skip: Option<Expression>,
  pub(crate) limit: Option<Expression>,
  pub(crate) order_by: Option<OrderBy>,
}

// Expressions

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression
{
  Array(Array),
  FunctionCall(FunctionCall),
  Map(Map),
  MemberAccess(Box<MemberAccess>),
  Parameter(Parameter),
  Value(Value),
  Variable(Variable),
}

// Order By Expression

#[derive(Debug)]
pub(crate) struct OrderByExpression
{
  asc: bool,
  expression: Expression,
}

// Values: CreatePatterns

#[derive(Debug)]
pub(crate) enum Pattern
{
  Node(NodePattern),
  Edge(EdgePattern),
  Path(PathPattern),
}

#[derive(Debug, Clone)]
pub(crate) struct NodePattern
{
  pub(crate) variable: Option<String>,
  pub(crate) labels: LabelExpression,
  pub(crate) properties: Option<Expression>,
}

#[derive(Debug, Clone)]
pub(crate) struct EdgePattern
{
  pub(crate) variable: Option<String>,
  pub(crate) source: NodePattern,
  pub(crate) destination: NodePattern,
  pub(crate) directivity: graph::EdgeDirectivity,
  pub(crate) labels: LabelExpression,
  pub(crate) properties: Option<Expression>,
}

#[derive(Debug, Clone)]
pub(crate) struct PathPattern
{
  pub(crate) variable: String,
  pub(crate) edge: EdgePattern,
}

// Label Expression
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LabelExpression
{
  Not(Box<LabelExpression>),
  And(Vec<Box<LabelExpression>>),
  Or(Vec<Box<LabelExpression>>),
  String(String),
  None,
}

impl LabelExpression
{
  pub(crate) fn is_none(&self) -> bool
  {
    match self
    {
      LabelExpression::None => true,
      _ => false,
    }
  }
  pub(crate) fn and(self, rhs: LabelExpression) -> LabelExpression
  {
    match self
    {
      LabelExpression::None => rhs,
      LabelExpression::And(mut vec) => match rhs
      {
        LabelExpression::None => LabelExpression::And(vec),
        LabelExpression::And(mut rhs_vec) =>
        {
          vec.append(&mut rhs_vec);
          LabelExpression::And(vec)
        }
        other =>
        {
          vec.push(other.boxed());
          LabelExpression::And(vec)
        }
      },
      _ => match rhs
      {
        LabelExpression::None => self,
        LabelExpression::And(mut vec) =>
        {
          vec.push(self.boxed());
          LabelExpression::And(vec)
        }
        _ => LabelExpression::And(vec![self.boxed(), rhs.boxed()]),
      },
    }
  }
  pub(crate) fn or(self, rhs: LabelExpression) -> LabelExpression
  {
    match self
    {
      LabelExpression::None => rhs,
      LabelExpression::Or(mut vec) => match rhs
      {
        LabelExpression::None => LabelExpression::And(vec),
        LabelExpression::Or(mut rhs_vec) =>
        {
          vec.append(&mut rhs_vec);
          LabelExpression::Or(vec)
        }
        other =>
        {
          vec.push(other.boxed());
          LabelExpression::Or(vec)
        }
      },
      _ => match rhs
      {
        LabelExpression::None => self,
        LabelExpression::Or(mut vec) =>
        {
          vec.push(self.boxed());
          LabelExpression::Or(vec)
        }
        _ => LabelExpression::Or(vec![self.boxed(), rhs.boxed()]),
      },
    }
  }
  fn clone_boxed(&self) -> Box<LabelExpression>
  {
    self.clone().boxed()
  }
  pub(crate) fn boxed(self) -> Box<LabelExpression>
  {
    Box::new(self)
  }
  pub(crate) fn is_all_inclusive(&self) -> bool
  {
    match self
    {
      LabelExpression::None => true,
      LabelExpression::And(exprs) => !exprs.iter().any(|f| !f.is_all_inclusive()),
      LabelExpression::Or(_) => false,
      LabelExpression::String(_) => true,
      LabelExpression::Not(_) => false,
    }
  }
}

// Values

#[derive(Debug)]
pub(crate) struct All {}
#[derive(Debug)]
pub(crate) struct EndOfList {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Value
{
  pub(crate) value: crate::graph::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Map
{
  pub(crate) map: std::collections::HashMap<String, Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Array
{
  pub(crate) array: Vec<Expression>,
}

// Expressions

#[derive(Debug)]
pub(crate) struct NamedExpression
{
  pub(crate) name: String,
  pub(crate) expression: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Parameter
{
  pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Variable
{
  pub(crate) identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MemberAccess
{
  pub(crate) left: Expression,
  pub(crate) path: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct IndexAccess
{
  pub(crate) left: Expression,
  pub(crate) index: Expression,
  pub(crate) end: Expression,
}

#[derive(Debug)]
pub(crate) struct HasLabels
{
  pub(crate) left: String,
  pub(crate) labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionCall
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
}

#[macro_export]
macro_rules! create_binary_op {
  ( $x:tt ) => {
    #[derive(Debug)]
    pub(crate) struct $x
    {
      pub(crate) left: Expression,
      pub(crate) right: Expression,
    }
  };
}

create_binary_op! {LogicalAnd}
create_binary_op! {LogicalOr}
create_binary_op! {LogicalXor}
create_binary_op! {RelationalEqual}
create_binary_op! {RelationalDifferent}
create_binary_op! {RelationalInferior}
create_binary_op! {RelationalSuperior}
create_binary_op! {RelationalInferiorEqual}
create_binary_op! {RelationalSuperiorEqual}
create_binary_op! {RelationalIn}
create_binary_op! {RelationalNotIn}

create_binary_op! {Addition}
create_binary_op! {Substraction}
create_binary_op! {Multiplication}
create_binary_op! {Division}
create_binary_op! {Modulo}

#[macro_export]
macro_rules! create_unary_op {
  ( $x:tt ) => {
    #[derive(Debug)]
    pub(crate) struct $x
    {
      value: Expression,
    }
  };
}

create_unary_op! {LogicalNegation}
create_unary_op! {Negation}
create_unary_op! {IsNull}
create_unary_op! {IsNotNull}
