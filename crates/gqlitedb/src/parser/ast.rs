use std::{
  cell::RefCell,
  collections::{hash_map::Entry, HashMap},
  sync::atomic::AtomicU64,
};

use crate::graph;

/// Represent a variable name. Some variable are explicitly created by the parser, if a node/edge should be considered equal and appear in different expression.
/// For instance `()-[]->()-[]->()` needs the creation of a variable.
#[derive(Debug, Clone, Eq)]
pub(crate) struct VariableIdentifier
{
  /// Name of the variable, only useful for debug purposes
  name: String,
  /// Unique identifier of the variable, the uniqueness is only guaranteed within a compilation unit.
  id: u64,
}

impl VariableIdentifier
{
  pub(crate) fn name(&self) -> &String
  {
    &self.name
  }
  pub(crate) fn take_name(self) -> String
  {
    self.name
  }
}

impl PartialEq for VariableIdentifier
{
  fn eq(&self, other: &Self) -> bool
  {
    self.id == other.id
  }
}

impl std::hash::Hash for VariableIdentifier
{
  fn hash<H: std::hash::Hasher>(&self, state: &mut H)
  {
    self.id.hash(state);
  }
}

#[derive(Default)]
pub(crate) struct VariableIdentifiers
{
  next_id: AtomicU64,
  identifiers: RefCell<HashMap<String, VariableIdentifier>>,
}

impl VariableIdentifiers
{
  fn next_id(&self) -> u64
  {
    self
      .next_id
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
  }
  pub(crate) fn create_variable_from_name(&self, name: impl Into<String>) -> VariableIdentifier
  {
    let name = name.into();
    match self.identifiers.borrow_mut().entry(name)
    {
      Entry::Occupied(entry) => entry.get().clone(),
      Entry::Vacant(entry) =>
      {
        let vn = VariableIdentifier {
          name: entry.key().clone(),
          id: self.next_id(),
        };
        entry.insert(vn.clone());
        vn
      }
    }
  }
  pub(crate) fn create_variable_from_name_optional(
    &self,
    name: Option<String>,
  ) -> Option<VariableIdentifier>
  {
    name.map(|name| self.create_variable_from_name(name))
  }
  pub(crate) fn create_anonymous_variable(&self) -> VariableIdentifier
  {
    let id = self.next_id();
    VariableIdentifier {
      name: format!("anonymous_{}", id),
      id,
    }
  }
}

#[derive(Debug)]
pub(crate) enum Statement
{
  CreateGraph(CreateGraph),
  DropGraph(DropGraph),
  UseGraph(UseGraph),
  Create(Create),
  Match(Match),
  Return(Return),
  Call(Call),
  With(With),
  Unwind(Unwind),
  Delete(Delete),
  Update(Update),
}

macro_rules! create_from_statement {
  ( $x:tt ) => {
    impl From<$x> for Statement
    {
      fn from(v: $x) -> Statement
      {
        Statement::$x(v)
      }
    }
  };
}

pub(crate) type Statements = Vec<Statement>;
pub(crate) type Queries = Vec<Statements>;

#[derive(Debug)]
pub(crate) struct CreateGraph
{
  pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct DropGraph
{
  pub(crate) name: String,
  pub(crate) if_exists: bool,
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

create_from_statement! {Create}

#[derive(Debug)]
pub(crate) struct Match
{
  pub(crate) patterns: Vec<Pattern>,
  pub(crate) where_expression: Option<Expression>,
  pub(crate) optional: bool,
}

create_from_statement! {Match}

#[derive(Debug)]
pub(crate) struct Return
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
  pub(crate) where_expression: Option<Expression>,
}

create_from_statement! {Return}

#[derive(Debug)]
pub(crate) struct With
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
  pub(crate) where_expression: Option<Expression>,
}

create_from_statement! {With}

#[derive(Debug)]
pub(crate) struct Unwind
{
  pub(crate) identifier: VariableIdentifier,
  pub(crate) expression: Expression,
}

create_from_statement! {Unwind}

#[derive(Debug)]
pub(crate) struct Delete
{
  pub(crate) detach: bool,
  pub(crate) expressions: Vec<Expression>,
}

#[derive(Debug)]
pub(crate) struct Update
{
  pub(crate) updates: Vec<OneUpdate>,
}

#[derive(Debug)]
pub(crate) struct Call
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
}

// Set/remove Statements

#[derive(Debug)]
pub(crate) enum OneUpdate
{
  SetProperty(UpdateProperty),
  AddProperty(UpdateProperty),
  RemoveProperty(RemoveProperty),
  AddLabels(AddRemoveLabels),
  RemoveLabels(AddRemoveLabels),
}

#[derive(Debug)]
pub(crate) struct UpdateProperty
{
  pub(crate) target: VariableIdentifier,
  pub(crate) path: Vec<String>,
  pub(crate) expression: Expression,
}

#[derive(Debug)]
pub(crate) struct RemoveProperty
{
  pub(crate) target: VariableIdentifier,
  pub(crate) path: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct AddRemoveLabels
{
  pub(crate) target: VariableIdentifier,
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
#[allow(clippy::large_enum_variant)]
pub(crate) enum Expression
{
  Array(Array),
  FunctionCall(FunctionCall),
  Map(Map),
  IndexAccess(Box<IndexAccess>),
  RangeAccess(Box<RangeAccess>),
  MemberAccess(Box<MemberAccess>),
  Parameter(Parameter),
  Value(Value),
  Variable(Variable),

  LogicalAnd(Box<LogicalAnd>),
  LogicalOr(Box<LogicalOr>),
  LogicalXor(Box<LogicalXor>),
  RelationalEqual(Box<RelationalEqual>),
  RelationalDifferent(Box<RelationalDifferent>),
  RelationalInferior(Box<RelationalInferior>),
  RelationalSuperior(Box<RelationalSuperior>),
  RelationalInferiorEqual(Box<RelationalInferiorEqual>),
  RelationalSuperiorEqual(Box<RelationalSuperiorEqual>),
  RelationalIn(Box<RelationalIn>),
  RelationalNotIn(Box<RelationalNotIn>),

  Addition(Box<Addition>),
  Subtraction(Box<Subtraction>),
  Multiplication(Box<Multiplication>),
  Division(Box<Division>),
  Modulo(Box<Modulo>),
  Exponent(Box<Exponent>),

  Negation(Box<Negation>),
  LogicalNegation(Box<LogicalNegation>),
  IsNull(Box<IsNull>),
  IsNotNull(Box<IsNotNull>),
}

// Order By Expression

#[derive(Debug)]
pub(crate) struct OrderByExpression
{
  pub asc: bool,
  pub expression: Expression,
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
  pub(crate) variable: Option<VariableIdentifier>,
  pub(crate) labels: LabelExpression,
  pub(crate) properties: Option<Expression>,
}

#[derive(Debug, Clone)]
pub(crate) struct EdgePattern
{
  pub(crate) variable: Option<VariableIdentifier>,
  pub(crate) source: NodePattern,
  pub(crate) destination: NodePattern,
  pub(crate) directivity: graph::EdgeDirectivity,
  pub(crate) labels: LabelExpression,
  pub(crate) properties: Option<Expression>,
}

#[derive(Debug, Clone)]
pub(crate) struct PathPattern
{
  pub(crate) variable: VariableIdentifier,
  pub(crate) edge: EdgePattern,
}

// Label Expression
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LabelExpression
{
  #[allow(dead_code)]
  Not(Box<LabelExpression>),
  And(Vec<LabelExpression>),
  Or(Vec<LabelExpression>),
  String(String),
  None,
}

impl LabelExpression
{
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
          vec.push(other);
          LabelExpression::And(vec)
        }
      },
      _ => match rhs
      {
        LabelExpression::None => self,
        LabelExpression::And(mut vec) =>
        {
          vec.push(self);
          LabelExpression::And(vec)
        }
        _ => LabelExpression::And(vec![self, rhs]),
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
          vec.push(other);
          LabelExpression::Or(vec)
        }
      },
      _ => match rhs
      {
        LabelExpression::None => self,
        LabelExpression::Or(mut vec) =>
        {
          vec.push(self);
          LabelExpression::Or(vec)
        }
        _ => LabelExpression::Or(vec![self, rhs]),
      },
    }
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
  pub(crate) fn is_none(&self) -> bool
  {
    matches!(self, LabelExpression::None)
  }
  pub(crate) fn is_string(&self) -> bool
  {
    matches!(self, LabelExpression::String(_))
  }
}

// Expressions

macro_rules! create_from_expr {
  ( $x:tt ) => {
    impl From<$x> for Expression
    {
      fn from(v: $x) -> Expression
      {
        Expression::$x(v)
      }
    }
  };
}

macro_rules! create_from_boxed_expr {
  ( $x:tt ) => {
    impl From<$x> for Expression
    {
      fn from(v: $x) -> Expression
      {
        Expression::$x(Box::new(v))
      }
    }
  };
}

#[derive(Debug)]
pub(crate) struct NamedExpression
{
  pub(crate) identifier: VariableIdentifier,
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
  pub(crate) identifier: VariableIdentifier,
}

create_from_expr! {Variable}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MemberAccess
{
  pub(crate) left: Expression,
  pub(crate) path: Vec<String>,
}

create_from_boxed_expr! {MemberAccess}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexAccess
{
  pub(crate) left: Expression,
  pub(crate) index: Expression,
}

create_from_boxed_expr! {IndexAccess}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RangeAccess
{
  pub(crate) left: Expression,
  pub(crate) start: Option<Expression>,
  pub(crate) end: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionCall
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
}

create_from_expr! {FunctionCall}

macro_rules! create_binary_op {
  ( $x:tt ) => {
    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct $x
    {
      pub(crate) left: Expression,
      pub(crate) right: Expression,
    }

    create_from_boxed_expr! { $x }
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
create_binary_op! {Subtraction}
create_binary_op! {Multiplication}
create_binary_op! {Division}
create_binary_op! {Modulo}
create_binary_op! {Exponent}

macro_rules! create_unary_op {
  ( $x:tt ) => {
    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct $x
    {
      pub(crate) value: Expression,
    }
    create_from_boxed_expr! { $x }
  };
}

create_unary_op! {LogicalNegation}
create_unary_op! {Negation}
create_unary_op! {IsNull}
create_unary_op! {IsNotNull}

// Values

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Value
{
  pub(crate) value: crate::value::Value,
}

create_from_expr! {Value}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Map
{
  pub(crate) map: Vec<(String, Expression)>,
}

create_from_expr! {Map}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Array
{
  pub(crate) array: Vec<Expression>,
}

create_from_expr! {Array}
