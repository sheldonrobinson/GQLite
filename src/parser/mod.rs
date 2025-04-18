use ast::{EdgePattern, Expression, LabelExpression, NodePattern};
use pest::{
  pratt_parser::{Assoc, Op, PrattParser},
  Parser,
};
use pest_derive::Parser;

use crate::{
  error::{CompileTimeError, InternalError},
  graph::{self, Edge},
  Result,
};

trait TryNext: Iterator
{
  fn try_next(&mut self) -> Result<Self::Item>
  {
    self
      .next()
      .ok_or_else(|| crate::Error::InternalError("Missing element in iterator."))
  }
}

impl<T: Iterator> TryNext for T {}

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

fn build_pair(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<(String, ast::Expression)>
{
  let mut it = pair.into_inner();
  let k = it.try_next()?;
  let v = build_expression(it.try_next()?.into_inner(), pratt)?;
  return Ok((k.as_str().to_string(), v));
}

fn build_expression(
  pairs: pest::iterators::Pairs<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Expression>
{
  pratt
    .map_primary(|primary| build_expression_primary(primary, pratt))
    .map_prefix(|op, rhs| match op.as_rule()
    {
      Rule::negation => Ok(ast::Negation { value: rhs? }.into()),
      Rule::not => Ok(ast::LogicalNegation { value: rhs? }.into()),
      unknown_expression => Err(crate::Error::UnxpectedExpression(
        "build_expression/map_prefix",
        format!("{unknown_expression:?}"),
      )),
    })
    .map_postfix(|lhs, op| match op.as_rule()
    {
      Rule::is_null => Ok(ast::IsNull { value: lhs? }.into()),
      Rule::is_not_null => Ok(ast::IsNotNull { value: lhs? }.into()),
      unknown_expression => Err(crate::Error::UnxpectedExpression(
        "build_expression/map_postfix",
        format!("{unknown_expression:?}"),
      )),
    })
    .map_infix(|lhs, op, rhs| match op.as_rule()
    {
      Rule::addition => Ok(
        ast::Addition {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::substraction => Ok(
        ast::Substraction {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::multiplication => Ok(
        ast::Multiplication {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::division => Ok(
        ast::Division {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::modulo => Ok(
        ast::Modulo {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::or => Ok(
        ast::LogicalOr {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::and => Ok(
        ast::LogicalAnd {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::equal => Ok(
        ast::RelationalEqual {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::different => Ok(
        ast::RelationalDifferent {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::inferior => Ok(
        ast::RelationalInferior {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::superior => Ok(
        ast::RelationalSuperior {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::inferior_equal => Ok(
        ast::RelationalInferiorEqual {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::superior_equal => Ok(
        ast::RelationalSuperiorEqual {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::not_in => Ok(
        ast::RelationalNotIn {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      Rule::in_ => Ok(
        ast::RelationalIn {
          left: lhs?,
          right: rhs?,
        }
        .into(),
      ),
      unknown_expression => Err(crate::Error::UnxpectedExpression(
        "build_expression/map_postfix",
        format!("{unknown_expression:?}"),
      )),
    })
    .parse(pairs)
}

fn build_expression_primary(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Expression>
{
  match pair.as_rule()
  {
    Rule::expression_term =>
    {
      let pair = pair.into_inner().try_next()?;
      match pair.as_rule()
      {
        Rule::null_lit => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Invalid,
        })),
        Rule::true_lit => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Boolean(true),
        })),
        Rule::false_lit => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Boolean(false),
        })),
        Rule::int => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Integer(pair.as_str().parse()?),
        })),
        Rule::num => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Float(pair.as_str().parse()?),
        })),
        Rule::ident => Ok(ast::Expression::Variable(ast::Variable {
          identifier: pair.as_str().to_string(),
        })),
        Rule::parameter => Ok(ast::Expression::Parameter(ast::Parameter {
          name: pair.as_str().to_string(),
        })),
        Rule::array => Ok(ast::Expression::Array(ast::Array {
          array: pair
            .into_inner()
            .map(|pair| build_expression(pair.into_inner(), pratt))
            .collect::<Result<Vec<ast::Expression>>>()?,
        })),
        Rule::map => build_map(pair, pratt),
        Rule::member_access =>
        {
          let mut it = pair.into_inner();
          let left = ast::Expression::Variable(ast::Variable {
            identifier: it.try_next()?.as_str().to_string(),
          });
          Ok(ast::Expression::MemberAccess(Box::new(ast::MemberAccess {
            left,
            path: it.map(|el| el.as_str().to_string()).collect(),
          })))
        }
        Rule::string_literal => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::String(pair.into_inner().try_next()?.as_str().to_string()),
        })),
        Rule::function_call =>
        {
          let mut it = pair.into_inner();
          let function_name = it
            .next()
            .ok_or_else(|| crate::Error::InternalError("Missing function name."))?
            .as_str();
          Ok(ast::Expression::FunctionCall(ast::FunctionCall {
            name: function_name.to_string(),
            arguments: it
              .map(|pair| build_expression(pair.into_inner(), pratt))
              .collect::<Result<_>>()?,
          }))
        }
        Rule::count_star => Ok(ast::Expression::FunctionCall(ast::FunctionCall {
          name: "count".into(),
          arguments: vec![ast::Expression::Value(ast::Value { value: 0.into() })],
        })),
        Rule::parenthesised_expression =>
        {
          let mut it = pair.into_inner();
          build_expression(it.try_next()?.into_inner(), pratt)
        }
        Rule::label_check_expression =>
        {
          let it = pair.into_inner();
          Ok(ast::Expression::FunctionCall(ast::FunctionCall {
            name: "has_labels".into(),
            arguments: it
              .enumerate()
              .map(|(i, pair)| {
                if i == 0
                {
                  ast::Expression::Variable(ast::Variable {
                    identifier: pair.as_str().into(),
                  })
                }
                else
                {
                  ast::Expression::Value(ast::Value {
                    value: graph::Value::String(pair.as_str().into()),
                  })
                }
              })
              .collect(),
          }))
        }
        unknown_expression => Err(crate::Error::UnxpectedExpression(
          "build_expression_term",
          format!("{unknown_expression:?}"),
        )),
      }
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_expression_term",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_map(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Expression>
{
  match pair.as_rule()
  {
    Rule::map => Ok(ast::Expression::Map({
      let mut map = std::collections::HashMap::new();
      for k_v_pair in pair.into_inner()
      {
        let (k, v) = build_pair(k_v_pair, pratt)?;
        map.insert(k, v);
      }
      ast::Map { map: map }
    })),
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_map",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_modifiers(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Modifiers>
{
  let skip = None;
  let mut limit = None;
  let order_by = None;
  for subpair in pair.into_inner()
  {
    match subpair.as_rule()
    {
      Rule::limit =>
      {
        limit = Some(build_expression(
          subpair.into_inner().try_next()?.into_inner(),
          pratt,
        )?)
      }
      _ => Err::<(), crate::Error>(
        InternalError::UnexpectedPair {
          context: "build_modifiers",
          pair: format!("{:#?}", subpair),
        }
        .into(),
      )?,
    }
  }
  Ok(ast::Modifiers {
    skip,
    limit,
    order_by,
  })
}

fn build_named_expression(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::NamedExpression>
{
  match pair.as_rule()
  {
    Rule::named_expression =>
    {
      let mut inner = pair.into_inner();
      match inner.len()
      {
        1 =>
        {
          let expr = inner.try_next()?;
          Ok(ast::NamedExpression {
            name: expr.as_str().trim().to_string(),
            expression: build_expression(expr.into_inner(), pratt)?,
          })
        }
        2 =>
        {
          let expression = build_expression(inner.try_next()?.into_inner(), pratt)?;
          let name = inner.try_next()?.as_str().to_string();
          Ok(ast::NamedExpression { name, expression })
        }
        _ =>
        {
          panic!(
            "Invalid number of terms in named expressions {}",
            inner.len()
          );
        }
      }
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_named_expressions",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_labels(pair: pest::iterators::Pair<Rule>) -> Result<ast::LabelExpression>
{
  match pair.as_rule()
  {
    Rule::labels => build_labels(pair.into_inner().try_next()?),
    Rule::label_alternative =>
    {
      let mut r = ast::LabelExpression::None;
      let mut inner = pair.into_inner();
      while let Some(next) = inner.next()
      {
        r = r.or(build_labels(next)?);
      }
      Ok(r)
    }
    Rule::label_inclusion =>
    {
      let mut r = ast::LabelExpression::None;
      let mut inner = pair.into_inner();
      while let Some(next) = inner.next()
      {
        r = r.and(build_labels(next)?);
      }
      Ok(r)
    }
    Rule::label_atom => Ok(ast::LabelExpression::String(pair.as_str().to_string())),
    _ => Err(
      InternalError::UnexpectedPair {
        context: "build_labels",
        pair: format!("{:#?}", pair),
      }
      .into(),
    ),
  }
}

fn build_node_pattern(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::NodePattern>
{
  let it = pair.into_inner();
  let mut variable = None;
  let mut labels = ast::LabelExpression::None;
  let mut properties = None;

  for pair in it
  {
    match pair.as_rule()
    {
      Rule::ident =>
      {
        variable = Some(pair.as_str().to_string());
      }
      Rule::labels =>
      {
        labels = build_labels(pair)?;
      }
      Rule::map => properties = Some(build_map(pair, pratt)?),
      unknown_expression =>
      {
        return Err(crate::Error::UnxpectedExpression(
          "build_node_pattern",
          format!("{unknown_expression:?}"),
        ));
      }
    }
  }
  Ok(ast::NodePattern {
    variable,
    labels,
    properties,
  })
}

fn build_edge_pattern(
  source_node: NodePattern,
  edge_pair: pest::iterators::Pair<Rule>,
  destination_node: NodePattern,
  allow_undirected_edge: bool,
  pratt: &PrattParser<Rule>,
) -> Result<EdgePattern>
{
  let edge_rule = edge_pair.as_rule();
  let it = edge_pair.into_inner().try_next()?.into_inner();
  let mut variable = None;
  let mut labels = LabelExpression::None;
  let mut properties = None;

  for pair in it
  {
    match pair.as_rule()
    {
      Rule::ident =>
      {
        variable = Some(pair.as_str().to_string());
      }
      Rule::labels =>
      {
        labels = build_labels(pair)?;
      }
      Rule::map => properties = Some(build_map(pair, pratt)?),
      unknown_expression =>
      {
        return Err(crate::Error::UnxpectedExpression(
          "build_edge_pattern",
          format!("{unknown_expression:?}"),
        ));
      }
    }
  }

  match edge_rule
  {
    Rule::directed_edge_pattern => Ok(ast::EdgePattern {
      variable,
      source: source_node,
      destination: destination_node,
      directivity: graph::EdgeDirectivity::Directed,
      labels,
      properties,
    }),
    Rule::reversed_edge_pattern => Ok(ast::EdgePattern {
      variable,
      source: destination_node,
      destination: source_node,
      directivity: graph::EdgeDirectivity::Directed,
      labels,
      properties,
    }),
    Rule::undirected_edge_pattern =>
    {
      if !allow_undirected_edge
      {
        Err(CompileTimeError::RequiresDirectedRelationship {
          context: "creation",
        })?;
      }
      Ok(ast::EdgePattern {
        variable,
        source: source_node,
        destination: destination_node,
        directivity: graph::EdgeDirectivity::Undirected,
        labels,
        properties,
      })
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_pattern/edge_pattern",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_patterns(
  iterator: &mut pest::iterators::Pairs<Rule>,
  allow_undirected_edge: bool,
  pratt: &PrattParser<Rule>,
) -> Result<Vec<ast::Pattern>>
{
  let mut vec = vec![];

  for pair in iterator
  {
    vec.append(&mut build_pattern(pair, allow_undirected_edge, pratt)?);
  }
  Ok(vec)
}

fn build_pattern(
  pair: pest::iterators::Pair<Rule>,
  allow_undirected_edge: bool,
  pratt: &PrattParser<Rule>,
) -> Result<Vec<ast::Pattern>>
{
  let mut vec = vec![];

  match pair.as_rule()
  {
    Rule::node_pattern =>
    {
      vec.push(ast::Pattern::Node(build_node_pattern(
        pair.into_inner().try_next()?,
        pratt,
      )?));
    }
    Rule::edge_pattern =>
    {
      let mut it = pair.into_inner();
      let mut source_node = build_node_pattern(it.try_next()?, pratt)?;

      while let Some(next) = it.next()
      {
        let destination_node = build_node_pattern(it.try_next()?, pratt)?;

        let edge_pattern = build_edge_pattern(
          source_node,
          next,
          destination_node.clone(),
          allow_undirected_edge,
          pratt,
        )?;
        vec.push(ast::Pattern::Edge(edge_pattern));
        source_node = destination_node;
      }
    }
    Rule::path_pattern =>
    {
      let mut it = pair.into_inner();
      let variable = it.try_next()?.as_str().to_string();
      let source_node = build_node_pattern(it.try_next()?, pratt)?;
      let edge_it = it.try_next()?;
      let destination_node = build_node_pattern(it.try_next()?, pratt)?;

      let edge_pattern = build_edge_pattern(
        source_node,
        edge_it,
        destination_node,
        allow_undirected_edge,
        pratt,
      )?;
      vec.push(ast::Pattern::Path(ast::PathPattern {
        variable,
        edge: edge_pattern,
      }));
    }
    unknown_expression =>
    {
      return Err(crate::Error::UnxpectedExpression(
        "build_node_or_edge_vec",
        format!("{unknown_expression:?}"),
      ));
    }
  };
  Ok(vec)
}

fn build_match(
  pair: pest::iterators::Pair<Rule>,
  optional: bool,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Statement>
{
  let inner = pair.into_inner();
  let mut where_expression = None;
  let mut patterns = vec![];
  for pair in inner
  {
    match pair.as_rule()
    {
      Rule::where_modifier =>
      {
        where_expression = Some(build_expression(
          pair.into_inner().try_next()?.into_inner(),
          pratt,
        )?)
      }
      _ => patterns.append(&mut build_pattern(pair, true, pratt)?),
    }
  }

  Ok(ast::Statement::Match(ast::Match {
    patterns,
    where_expression,
    optional,
  }))
}

fn build_ast_from_statement(
  pair: pest::iterators::Pair<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<ast::Statement>
{
  match pair.as_rule()
  {
    Rule::create_statement => Ok(ast::Statement::Create(ast::Create {
      patterns: build_patterns(&mut pair.into_inner(), false, pratt)?,
    })),
    Rule::match_statement => build_match(pair, false, pratt),
    Rule::optional_match_statement => build_match(pair.into_inner().try_next()?, true, pratt),
    Rule::return_statement =>
    {
      let named_expressions = pair
        .into_inner()
        .map(|pair| build_named_expression(pair, pratt))
        .collect::<Result<Vec<ast::NamedExpression>>>()?;
      Ok(ast::Statement::Return(ast::Return {
        all: false,
        expressions: named_expressions,
        modifiers: ast::Modifiers::default(),
      }))
    }
    Rule::with_statement =>
    {
      let mut all = false;
      let mut expressions = vec![];
      let mut modifiers = Default::default();

      for sub_pair in pair.into_inner()
      {
        match sub_pair.as_rule()
        {
          Rule::star => all = true,
          Rule::named_expression => expressions.push(build_named_expression(sub_pair, pratt)?),
          Rule::modifiers => modifiers = build_modifiers(sub_pair, pratt)?,
          _ => Err(InternalError::UnexpectedPair {
            context: "build_ast_from_statement/with_statement",
            pair: sub_pair.as_str().to_string(),
          })?,
        }
      }

      Ok(ast::Statement::With(ast::With {
        all,
        expressions,
        modifiers,
      }))
    }
    Rule::unwind_statement =>
    {
      let pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| InternalError::MissingPair {
          context: "build_ast_from_statement/inner",
        })?;

      let ne = match pair.as_rule()
      {
        Rule::named_expression => build_named_expression(pair, pratt),
        _ => Err(
          InternalError::UnexpectedPair {
            context: "build_ast_from_statement/with_statement",
            pair: pair.as_str().to_string(),
          }
          .into(),
        ),
      }?;
      Ok(ast::Statement::Unwind(ast::Unwind {
        expression: ne.expression,
        name: ne.name,
      }))
    }
    Rule::call_statement =>
    {
      let name = pair
        .into_inner()
        .map(|pair| pair.as_str())
        .collect::<Vec<&str>>()
        .join(".");
      Ok(ast::Statement::Call(ast::Call {
        name: name,
        arguments: Default::default(),
        yield_: Default::default(),
      }))
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_ast_from_statement",
      format!("{unknown_expression:?}"),
    )),
  }
}

pub(crate) fn parse(input: &str) -> Result<ast::Statements>
{
  let pratt = PrattParser::new()
    .op(Op::infix(Rule::or, Assoc::Left))
    .op(Op::infix(Rule::and, Assoc::Left))
    .op(Op::infix(Rule::equal, Assoc::Left) | Op::infix(Rule::different, Assoc::Left))
    .op(
      Op::infix(Rule::inferior, Assoc::Left)
        | Op::infix(Rule::inferior_equal, Assoc::Left)
        | Op::infix(Rule::superior, Assoc::Left)
        | Op::infix(Rule::superior_equal, Assoc::Left),
    )
    .op(Op::infix(Rule::not_in, Assoc::Left) | Op::infix(Rule::in_, Assoc::Left))
    .op(Op::infix(Rule::addition, Assoc::Left) | Op::infix(Rule::substraction, Assoc::Left))
    .op(
      Op::infix(Rule::multiplication, Assoc::Left)
        | Op::infix(Rule::division, Assoc::Left)
        | Op::infix(Rule::modulo, Assoc::Left),
    )
    .op(Op::prefix(Rule::not) | Op::prefix(Rule::negation))
    .op(Op::postfix(Rule::is_null) | Op::postfix(Rule::is_not_null));
  let pairs = GQLParser::parse(Rule::query, input)?;
  let mut stmts = ast::Statements::new();
  if crate::consts::SHOW_PARSE_TREE
  {
    println!("pairs = {:#?}", pairs);
  }
  for pair in pairs
  {
    match pair.as_rule()
    {
      Rule::statement =>
      {
        stmts.push(build_ast_from_statement(
          pair.into_inner().try_next()?,
          &pratt,
        )?);
      }
      Rule::EOI =>
      {}
      unknown_expression =>
      {
        Err(crate::Error::UnxpectedExpression(
          "parse",
          format!("{unknown_expression:?}"),
        ))?;
      }
    }
  }
  if crate::consts::SHOW_AST
  {
    println!("statements = {:#?}", &stmts);
  }
  Ok(stmts)
}
