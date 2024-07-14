pub(crate) enum Statement
{
  CreateGraph(CreateGraph),
  UseGraph(UseGraph),
  Create(Create),
  Match(Match),
  Return(Return),
}

pub(crate) type Statements = Vec<Statement>;

pub(crate) enum Node
{
  
}

pub(crate) struct CreateGraph
{
  pub(crate) name: String,
}

pub(crate) struct UseGraph
{
  pub(crate) name: String,
}

pub(crate) struct Create
{
  pub(crate) patterns: Vec<GraphNodeOrEdge>,
}


pub(crate) struct Match
{
  pub(crate) patterns: Vec<GraphNodeOrEdge>,
  pub(crate) where_expression: Option<Expression>,
  pub(crate) optional: bool,
}

pub(crate) struct Return
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
}

pub(crate) struct With
{
  pub(crate) all: bool,
  pub(crate) expressions: Vec<NamedExpression>,
  pub(crate) modifiers: Modifiers,
}

pub(crate) struct Unwind
{
  pub(crate) name: String,
  pub(crate) expression: Expression,
}

pub(crate) struct Delete
{
  pub(crate) detach: bool,
  pub(crate) expressions: Vec<Node>,
}

pub(crate) struct Set
{
  pub(crate) expressions: Vec<Node>,
}

pub(crate) struct Remove
{
  pub(crate) expressions: Vec<Node>,
}

pub(crate) struct Call
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
  pub(crate) yield_: Vec<String>
}

// Set/remove Statements

pub(crate) struct SetProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
  pub(crate) expression: Expression,
}

pub(crate) struct AddProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
  pub(crate) expression: Expression,
}

pub(crate) struct RemoveProperty
{
  pub(crate) left: String,
  pub(crate) path: Vec<String>,
}

pub(crate) struct EditLabels
{
  pub(crate) target: String,
  pub(crate) labels: Vec<String>,
}

// Modifiers

pub(crate) struct OrderBy
{
  pub(crate) expressions: Vec<OrderByExpression>,
}

#[derive(Default)]
pub(crate) struct Modifiers
{
  pub(crate) skip: Option<Expression>,
  pub(crate) limit: Option<Expression>,
  pub(crate) order_by: Option<OrderBy>,
}

// Expressions

pub(crate) enum Expression
{
  Value(Value),
  Variable(Variable),
}

// Order By Expression

pub(crate) struct OrderByExpression
{
  asc: bool,
  expression: Expression,
}

// Values: Patterns

pub(crate) enum GraphNodeOrEdge
{
  GraphNode(GraphNode),
  GraphEdge(GraphEdge),
}

pub(crate) struct GraphNode
{
  pub(crate) variable: Option<String>,
  pub(crate) labels: Vec<String>,
  pub(crate) properties: Option<Expression>,
}

enum EdgeDirectivity
{
  Undirected, Directed
}

pub(crate) struct GraphEdge
{
  pub(crate) variable: Option<String>,
  pub(crate) source: GraphNode,
  pub(crate) destination: GraphNode,
  pub(crate) directivity: EdgeDirectivity,
  pub(crate) labels: Vec<String>,
  pub(crate) propeties: Expression,
}

// Values

pub(crate) struct All {}
pub(crate) struct EndOfList {}

pub(crate) struct Value
{
  pub(crate) value: crate::graph::Value,
}

pub(crate) struct Map
{
  pub(crate) map: std::collections::HashMap<String, Expression>,
}

pub(crate) struct Array
{
  pub(crate) array: Vec<Expression>,
}

// Expressions

pub(crate) struct NamedExpression
{
  pub(crate) name: String,
  pub(crate) expression: Expression,
}

pub(crate) struct Variable
{
  pub(crate) identifier: String,
}

pub(crate) struct MemberAccess
{
  pub(crate) left: Expression,
  pub(crate) path: Vec<String>,
}

pub(crate) struct IndexAccess
{
  pub(crate) left: Expression,
  pub(crate) index: Expression,
  pub(crate) end: Expression,
}

pub(crate) struct HasLabels
{
  pub(crate) left: String,
  pub(crate) labels: Vec<String>,
}

pub(crate) struct FunctionCall
{
  pub(crate) name: String,
  pub(crate) arguments: Vec<Expression>,
}

#[macro_export]
macro_rules! create_binary_op {
  ( $x:tt ) => (
    pub(crate) struct $x {
      pub(crate) left: Expression,
      pub(crate) right: Expression,
    }
  )
}

create_binary_op!{LogicalAnd}
create_binary_op!{LogicalOr}
create_binary_op!{LogicalXor}
create_binary_op!{RelationalEqual}
create_binary_op!{RelationalDifferent}
create_binary_op!{RelationalInferior}
create_binary_op!{RelationalSuperior}
create_binary_op!{RelationalInferiorEqual}
create_binary_op!{RelationalSuperiorEqual}
create_binary_op!{RelationalIn}
create_binary_op!{RelationalNotIn}

create_binary_op!{Addition}
create_binary_op!{Substraction}
create_binary_op!{Multiplication}
create_binary_op!{Division}
create_binary_op!{Modulo}

#[macro_export]
macro_rules! create_unary_op {
  ( $x:tt ) => (
    pub(crate) struct $x {
      value: Expression,
    }
  )
}

create_unary_op!{LogicalNegation}
create_unary_op!{Negation}
create_unary_op!{IsNull}
create_unary_op!{IsNotNull}
