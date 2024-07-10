pub(crate) enum Statement
{
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
  name: String,
}

pub(crate) struct UseGraph
{
  name: String,
}

pub(crate) struct Create
{
  pub(crate) patterns: Vec<GraphNodeOrEdge>,
}


pub(crate) struct Match
{
  patterns: Vec<GraphNodeOrEdge>,
  where_expression: Expression,
  optional: bool,
}

pub(crate) struct Return
{
  all: bool,
  expressions: Vec<NamedExpression>,
  modifiers: Modifiers,
}

pub(crate) struct With
{
  all: bool,
  expressions: Vec<NamedExpression>,
  modifiers: Modifiers,
}

pub(crate) struct Unwind
{
  name: String,
  expression: Expression,
}

pub(crate) struct Delete
{
  detach: bool,
  expressions: Vec<Node>,
}

pub(crate) struct Set
{
  expressions: Vec<Node>,
}

pub(crate) struct Remove
{
  expressions: Vec<Node>,
}

pub(crate) struct Call
{
  name: String,
  arguments: Vec<Expression>,
  yield_: Vec<String>
}

// Set/remove Statements

pub(crate) struct SetProperty
{
  left: String,
  path: Vec<String>,
  expression: Expression,
}

pub(crate) struct AddProperty
{
  left: String,
  path: Vec<String>,
  expression: Expression,
}

pub(crate) struct RemoveProperty
{
  left: String,
  path: Vec<String>,
}

pub(crate) struct EditLabels
{
  target: String,
  labels: Vec<String>,
}

// Modifiers

pub(crate) struct OrderBy
{
  expressions: Vec<OrderByExpression>,
}

pub(crate) struct Modifiers
{
  skip: Expression,
  limit: Expression,
  order_by: OrderBy,
}

// Expressions

pub(crate) enum Expression
{

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
  value: crate::graph::Value,
}

pub(crate) struct Map
{
  map: std::collections::HashMap<String, Expression>,
}

pub(crate) struct Array
{
  array: Vec<Expression>,
}

// Expressions

pub(crate) struct NamedExpression
{
  name: String,
  expression: Expression,
}

pub(crate) struct Variable
{
  identifier: String,
}

pub(crate) struct MemberAccess
{
  left: Expression,
  path: Vec<String>,
}

pub(crate) struct IndexAccess
{
  left: Expression,
  index: Expression,
  end: Expression,
}

pub(crate) struct HasLabels
{
  left: String,
  labels: Vec<String>,
}

pub(crate) struct FunctionCall
{
  name: String,
  arguments: Vec<Expression>,
}

#[macro_export]
macro_rules! create_binary_op {
  ( $x:tt ) => (
    pub(crate) struct $x {
      left: Expression,
      right: Expression,
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
