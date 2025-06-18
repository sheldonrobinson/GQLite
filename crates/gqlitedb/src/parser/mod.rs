use std::str::FromStr;

use pest::{
  pratt_parser::{Assoc, Op, PrattParser},
  Parser,
};
use pest_derive::Parser;

use crate::prelude::*;

trait TryNext: Iterator
{
  fn try_next(&mut self) -> Result<Self::Item>
  {
    self
      .next()
      .ok_or_else(|| crate::Error::InternalError("Missing element in iterator."))
  }
}

fn remove_hex_prefix<'a>(string: &'a str) -> String
{
  if &string[0..1] == "-"
  {
    format!("-{}", &string[3..])
  }
  else
  {
    string[2..].into()
  }
}

fn validate_float<'a>(value: f64, text: &'a str) -> Result<f64>
{
  if value.is_finite()
  {
    Ok(value)
  }
  else
  {
    Err(
      CompileTimeError::FloatingPointOverflow {
        text: text.to_owned(),
      }
      .into(),
    )
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
      Rule::member_access =>
      {
        let it = op.into_inner();
        Ok(ast::Expression::MemberAccess(Box::new(ast::MemberAccess {
          left: lhs?,
          path: it
            .map(|el| match el.as_rule()
            {
              Rule::ident => el.as_str().to_string(),
              Rule::string_literal => el.into_inner().as_str().to_string(),
              _ => todo!(),
            })
            .collect(),
        })))
      }
      Rule::index_access =>
      {
        let mut it = op.into_inner();

        Ok(ast::Expression::IndexAccess(Box::new(ast::IndexAccess {
          left: lhs?,
          index: it
            .next()
            .map(|el| build_expression(el.into_inner(), pratt))
            .unwrap()?,
        })))
      }
      Rule::range_access =>
      {
        let mut it = op.into_inner();
        let start = Some(
          it.next()
            .map(|el| build_expression(el.into_inner(), pratt))
            .unwrap()?,
        );
        let end = it
          .next()
          .map(|el| build_expression(el.into_inner(), pratt))
          .transpose()?;

        Ok(ast::Expression::RangeAccess(Box::new(ast::RangeAccess {
          left: lhs?,
          start,
          end,
        })))
      }
      Rule::range_access_to =>
      {
        let mut it = op.into_inner();
        let end = Some(
          it.next()
            .map(|el| build_expression(el.into_inner(), pratt))
            .unwrap()?,
        );

        Ok(ast::Expression::RangeAccess(Box::new(ast::RangeAccess {
          left: lhs?,
          start: None,
          end,
        })))
      }
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
      Rule::subtraction => Ok(
        ast::Subtraction {
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
      Rule::xor => Ok(
        ast::LogicalXor {
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
          value: {
            graph::Value::Integer(
              i64::from_str(pair.as_str())
                .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
            )
          },
        })),
        Rule::octa_int => Ok(ast::Expression::Value(ast::Value {
          value: {
            graph::Value::Integer(
              i64::from_str_radix(&remove_hex_prefix(pair.as_str()), 8)
                .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
            )
          },
        })),
        Rule::hexa_int => Ok(ast::Expression::Value(ast::Value {
          value: {
            graph::Value::Integer(
              i64::from_str_radix(&remove_hex_prefix(pair.as_str()), 16)
                .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
            )
          },
        })),
        Rule::num => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Float(validate_float(pair.as_str().parse()?, pair.as_str())?),
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
      let map = pair
        .into_inner()
        .map(|k_v_pair| build_pair(k_v_pair, pratt))
        .collect::<Result<_>>()?;
      ast::Map { map }
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
  let mut skip = None;
  let mut limit = None;
  let mut order_by = None;
  for subpair in pair.into_inner()
  {
    match subpair.as_rule()
    {
      Rule::limit =>
      {
        let mut subpair = subpair.into_inner();
        subpair.try_next()?; // eat limit_kw

        limit = Some(build_expression(subpair.try_next()?.into_inner(), pratt)?)
      }
      Rule::skip =>
      {
        let mut subpair = subpair.into_inner();
        subpair.try_next()?; // eat limit_kw

        skip = Some(build_expression(subpair.try_next()?.into_inner(), pratt)?)
      }
      Rule::order_by =>
      {
        let mut subpair = subpair.into_inner();
        subpair.try_next()?; // eat order_by_kw

        order_by = Some(ast::OrderBy {
          expressions: subpair
            .map(|r| match r.as_rule()
            {
              Rule::order_by_asc_expression => Ok(ast::OrderByExpression {
                asc: true,
                expression: build_expression(r.into_inner().try_next()?.into_inner(), pratt)?,
              }),
              Rule::order_by_desc_expression => Ok(ast::OrderByExpression {
                asc: false,
                expression: build_expression(r.into_inner().try_next()?.into_inner(), pratt)?,
              }),
              _ => Err::<_, crate::Error>(
                InternalError::UnexpectedPair {
                  context: "build_modifiers/order_by",
                  pair: format!("{:#?}", r),
                }
                .into(),
              ),
            })
            .collect::<Result<_>>()?,
        })
      }
      _ => Err(InternalError::UnexpectedPair {
        context: "build_modifiers",
        pair: format!("{:#?}", subpair),
      })?,
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
  source_node: ast::NodePattern,
  edge_pair: pest::iterators::Pair<Rule>,
  destination_node: ast::NodePattern,
  allow_undirected_edge: bool,
  pratt: &PrattParser<Rule>,
) -> Result<ast::EdgePattern>
{
  let edge_rule = edge_pair.as_rule();
  let it = edge_pair.into_inner().try_next()?.into_inner();
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

fn build_return_with_statement(
  pairs: pest::iterators::Pairs<Rule>,
  pratt: &PrattParser<Rule>,
) -> Result<(
  bool,
  Vec<ast::NamedExpression>,
  ast::Modifiers,
  Option<ast::Expression>,
)>
{
  let mut all = false;
  let mut expressions = vec![];
  let mut modifiers = Default::default();
  let mut where_expression = Default::default();

  for sub_pair in pairs
  {
    match sub_pair.as_rule()
    {
      Rule::star => all = true,
      Rule::named_expression => expressions.push(build_named_expression(sub_pair, pratt)?),
      Rule::modifiers => modifiers = build_modifiers(sub_pair, pratt)?,
      Rule::where_modifier =>
      {
        where_expression = Some(build_expression(
          sub_pair.into_inner().try_next()?.into_inner(),
          pratt,
        )?)
      }
      _ => Err(InternalError::UnexpectedPair {
        context: "build_ast_from_statement/with_statement",
        pair: sub_pair.as_str().to_string(),
      })?,
    }
  }
  Ok((all, expressions, modifiers, where_expression))
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
      let (all, expressions, modifiers, where_expression) =
        build_return_with_statement(pair.into_inner(), pratt)?;

      Ok(ast::Statement::Return(ast::Return {
        all,
        expressions,
        modifiers,
        where_expression,
      }))
    }
    Rule::with_statement =>
    {
      let (all, expressions, modifiers, where_expression) =
        build_return_with_statement(pair.into_inner(), pratt)?;

      Ok(ast::Statement::With(ast::With {
        all,
        expressions,
        modifiers,
        where_expression,
      }))
    }
    Rule::unwind_statement =>
    {
      let pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| InternalError::MissingPair {
          context: "build_ast_from_statement/unwind_statement",
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
    Rule::delete_statement | Rule::detach_delete_statement =>
    {
      let detach = match pair.as_rule()
      {
        Rule::delete_statement => false,
        Rule::detach_delete_statement => true,
        _ =>
        {
          return Err(
            InternalError::UnexpectedPair {
              context: "build_ast_from_statement/delete_statement",
              pair: pair.to_string(),
            }
            .into(),
          )
        }
      };

      let pairs = pair.into_inner();

      let expressions = pairs
        .into_iter()
        .map(|pair| build_expression(pair.into_inner(), pratt))
        .collect::<Result<_>>()?;

      Ok(ast::Statement::Delete(ast::Delete {
        detach,
        expressions,
      }))
    }
    Rule::set_statement =>
    {
      let mut updates = Vec::<ast::OneUpdate>::new();
      for pair in pair.into_inner()
      {
        match pair.as_rule()
        {
          Rule::set_eq_expression | Rule::set_add_expression =>
          {
            let add_property = pair.as_rule() == Rule::set_add_expression;

            let mut pair = pair.into_inner();
            let mut pair_left = pair.try_next()?.into_inner();
            let target = pair_left.try_next()?.as_str().to_string();
            let path = pair_left.map(|el| el.as_str().to_string()).collect();
            let expression = build_expression(pair.try_next()?.into_inner(), pratt)?;
            let update_property = ast::UpdateProperty {
              target,
              path,
              expression,
            };
            if add_property
            {
              updates.push(ast::OneUpdate::AddProperty(update_property));
            }
            else
            {
              updates.push(ast::OneUpdate::SetProperty(update_property));
            }
          }
          Rule::set_label_expression =>
          {
            let mut pair = pair.into_inner();
            let target = pair.try_next()?.as_str().to_string();
            let labels = pair.map(|el| el.as_str().to_string()).collect();
            updates.push(ast::OneUpdate::AddLabels(ast::AddRemoveLabels {
              target,
              labels,
            }));
          }
          unknown_expression => Err(crate::Error::UnxpectedExpression(
            "build_ast_from_statement/set_statement",
            format!("{unknown_expression:?}"),
          ))?,
        }
      }
      Ok(ast::Statement::Update(ast::Update { updates }))
    }
    Rule::remove_statement =>
    {
      let mut updates = Vec::<ast::OneUpdate>::new();
      for pair in pair.into_inner()
      {
        match pair.as_rule()
        {
          Rule::remove_member_access =>
          {
            let mut pair = pair.into_inner();
            let target = pair.try_next()?.as_str().to_string();
            let path = pair.map(|el| el.as_str().to_string()).collect();
            updates.push(ast::OneUpdate::RemoveProperty(ast::RemoveProperty {
              target,
              path,
            }));
          }
          Rule::set_label_expression =>
          {
            let mut pair = pair.into_inner();
            let target = pair.try_next()?.as_str().to_string();
            let labels = pair.map(|el| el.as_str().to_string()).collect();
            updates.push(ast::OneUpdate::RemoveLabels(ast::AddRemoveLabels {
              target,
              labels,
            }));
          }
          unknown_expression => Err(crate::Error::UnxpectedExpression(
            "build_ast_from_statement/remove_statement",
            format!("{unknown_expression:?}"),
          ))?,
        }
      }
      Ok(ast::Statement::Update(ast::Update { updates }))
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
    .op(Op::infix(Rule::xor, Assoc::Left))
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
    .op(Op::infix(Rule::addition, Assoc::Left) | Op::infix(Rule::subtraction, Assoc::Left))
    .op(
      Op::infix(Rule::multiplication, Assoc::Left)
        | Op::infix(Rule::division, Assoc::Left)
        | Op::infix(Rule::modulo, Assoc::Left),
    )
    .op(Op::prefix(Rule::not) | Op::prefix(Rule::negation))
    .op(
      Op::postfix(Rule::is_null)
        | Op::postfix(Rule::is_not_null)
        | Op::postfix(Rule::member_access)
        | Op::postfix(Rule::index_access)
        | Op::postfix(Rule::range_access)
        | Op::postfix(Rule::range_access_to),
    );
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
