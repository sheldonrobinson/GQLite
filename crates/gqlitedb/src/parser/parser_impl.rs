use std::str::FromStr;

use pest::{
  pratt_parser::{Assoc, Op, PrattParser},
  Parser,
};
use pest_derive::Parser;

use crate::prelude::*;
use parser::ast;

trait TryNext: Iterator
{
  fn try_next(&mut self) -> Result<Self::Item>
  {
    self
      .next()
      .ok_or_else(|| error::InternalError::MissingElementIterator.into())
  }
}

fn remove_hex_prefix(string: &str) -> String
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

fn validate_float(value: f64, text: &str) -> Result<f64>
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

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

pub(crate) struct AstBuilder
{
  pratt: PrattParser<Rule>,
  var_ids: ast::VariableIdentifiers,
}

impl AstBuilder
{
  fn build_pair(&self, pair: pest::iterators::Pair<Rule>) -> Result<(String, ast::Expression)>
  {
    let mut it = pair.into_inner();
    let k = it.try_next()?;
    let v = self.build_expression(it.try_next()?.into_inner())?;
    Ok((k.as_str().to_string(), v))
  }

  fn build_expression(&self, pairs: pest::iterators::Pairs<Rule>) -> Result<ast::Expression>
  {
    self
      .pratt
      .map_primary(|primary| self.build_expression_primary(primary))
      .map_prefix(|op, rhs| match op.as_rule()
      {
        Rule::negation => Ok(ast::Negation { value: rhs? }.into()),
        Rule::not => Ok(ast::LogicalNegation { value: rhs? }.into()),
        unknown_expression => Err(
          error::InternalError::UnxpectedExpression(
            "build_expression/map_prefix",
            format!("{unknown_expression:?}"),
          )
          .into(),
        ),
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
              .map(|el| self.build_expression(el.into_inner()))
              .unwrap()?,
          })))
        }
        Rule::range_access =>
        {
          let mut it = op.into_inner();
          let start = Some(
            it.next()
              .map(|el| self.build_expression(el.into_inner()))
              .unwrap()?,
          );
          let end = it
            .next()
            .map(|el| self.build_expression(el.into_inner()))
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
              .map(|el| self.build_expression(el.into_inner()))
              .unwrap()?,
          );

          Ok(ast::Expression::RangeAccess(Box::new(ast::RangeAccess {
            left: lhs?,
            start: None,
            end,
          })))
        }
        unknown_expression => Err(
          error::InternalError::UnxpectedExpression(
            "build_expression/map_postfix",
            format!("{unknown_expression:?}"),
          )
          .into(),
        ),
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
        Rule::exponent => Ok(
          ast::Exponent {
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
        unknown_expression => Err(
          error::InternalError::UnxpectedExpression(
            "build_expression/map_postfix",
            format!("{unknown_expression:?}"),
          )
          .into(),
        ),
      })
      .parse(pairs)
  }

  fn build_expression_primary(&self, pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression>
  {
    match pair.as_rule()
    {
      Rule::expression_term =>
      {
        let pair = pair.into_inner().try_next()?;
        match pair.as_rule()
        {
          Rule::null_lit => Ok(ast::Expression::Value(ast::Value {
            value: value::Value::Null,
          })),
          Rule::true_lit => Ok(ast::Expression::Value(ast::Value {
            value: value::Value::Boolean(true),
          })),
          Rule::false_lit => Ok(ast::Expression::Value(ast::Value {
            value: value::Value::Boolean(false),
          })),
          Rule::int => Ok(ast::Expression::Value(ast::Value {
            value: {
              value::Value::Integer(
                i64::from_str(pair.as_str())
                  .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
              )
            },
          })),
          Rule::octa_int => Ok(ast::Expression::Value(ast::Value {
            value: {
              value::Value::Integer(
                i64::from_str_radix(&remove_hex_prefix(pair.as_str()), 8)
                  .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
              )
            },
          })),
          Rule::hexa_int => Ok(ast::Expression::Value(ast::Value {
            value: {
              value::Value::Integer(
                i64::from_str_radix(&remove_hex_prefix(pair.as_str()), 16)
                  .map_err(|e| error::parse_int_error_to_compile_error(pair.as_str(), e))?,
              )
            },
          })),
          Rule::num => Ok(ast::Expression::Value(ast::Value {
            value: value::Value::Float(validate_float(pair.as_str().parse()?, pair.as_str())?),
          })),
          Rule::ident => Ok(ast::Expression::Variable(ast::Variable {
            identifier: self.var_ids.create_variable_from_name(pair.as_str()),
          })),
          Rule::parameter => Ok(ast::Expression::Parameter(ast::Parameter {
            name: pair.as_str().to_string(),
          })),
          Rule::array => Ok(ast::Expression::Array(ast::Array {
            array: pair
              .into_inner()
              .map(|pair| self.build_expression(pair.into_inner()))
              .collect::<Result<Vec<ast::Expression>>>()?,
          })),
          Rule::map => self.build_map(pair),
          Rule::string_literal => Ok(ast::Expression::Value(ast::Value {
            value: value::Value::String(pair.into_inner().try_next()?.as_str().to_string()),
          })),
          Rule::function_call =>
          {
            let mut it = pair.into_inner();
            let function_name = it
              .next()
              .ok_or_else(|| error::InternalError::MissingFunctionName)?
              .as_str();
            Ok(ast::Expression::FunctionCall(ast::FunctionCall {
              name: function_name.to_string(),
              arguments: it
                .map(|pair| self.build_expression(pair.into_inner()))
                .collect::<Result<_>>()?,
            }))
          }
          Rule::function_star =>
          {
            let mut it = pair.into_inner();
            let function_name = it
              .next()
              .ok_or_else(|| error::InternalError::MissingFunctionName)?
              .as_str();
            if function_name.to_lowercase() != "count"
            {
              Err(error::CompileTimeError::UnknownFunction {
                name: function_name.to_owned(),
              })?;
            }

            Ok(ast::Expression::FunctionCall(ast::FunctionCall {
              name: "count".into(),
              arguments: vec![ast::Expression::Value(ast::Value { value: 0.into() })],
            }))
          }
          Rule::parenthesised_expression =>
          {
            let mut it = pair.into_inner();
            self.build_expression(it.try_next()?.into_inner())
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
                      identifier: self.var_ids.create_variable_from_name(pair.as_str()),
                    })
                  }
                  else
                  {
                    ast::Expression::Value(ast::Value {
                      value: value::Value::String(pair.as_str().into()),
                    })
                  }
                })
                .collect(),
            }))
          }
          unknown_expression => Err(
            error::InternalError::UnxpectedExpression(
              "build_expression_term",
              format!("{unknown_expression:?}"),
            )
            .into(),
          ),
        }
      }
      unknown_expression => Err(
        error::InternalError::UnxpectedExpression(
          "build_expression_term",
          format!("{unknown_expression:?}"),
        )
        .into(),
      ),
    }
  }

  fn build_map(&self, pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression>
  {
    match pair.as_rule()
    {
      Rule::map => Ok(ast::Expression::Map({
        let map = pair
          .into_inner()
          .map(|k_v_pair| self.build_pair(k_v_pair))
          .collect::<Result<_>>()?;
        ast::Map { map }
      })),
      unknown_expression => Err(
        error::InternalError::UnxpectedExpression("build_map", format!("{unknown_expression:?}"))
          .into(),
      ),
    }
  }

  fn build_modifiers(&self, pair: pest::iterators::Pair<Rule>) -> Result<ast::Modifiers>
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

          limit = Some(self.build_expression(subpair.try_next()?.into_inner())?)
        }
        Rule::skip =>
        {
          let mut subpair = subpair.into_inner();
          subpair.try_next()?; // eat limit_kw

          skip = Some(self.build_expression(subpair.try_next()?.into_inner())?)
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
                  expression: self.build_expression(r.into_inner().try_next()?.into_inner())?,
                }),
                Rule::order_by_desc_expression => Ok(ast::OrderByExpression {
                  asc: false,
                  expression: self.build_expression(r.into_inner().try_next()?.into_inner())?,
                }),
                _ => Err::<_, crate::prelude::ErrorType>(
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
    &self,
    pair: pest::iterators::Pair<Rule>,
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
              identifier: self.var_ids.create_variable_from_name(expr.as_str().trim()),
              expression: self.build_expression(expr.into_inner())?,
            })
          }
          2 => Ok({
            let expression = self.build_expression(inner.try_next()?.into_inner())?;
            let identifier = self
              .var_ids
              .create_variable_from_name(inner.try_next()?.as_str());
            ast::NamedExpression {
              identifier,
              expression,
            }
          }),
          _ =>
          {
            panic!(
              "Invalid number of terms in named expressions {}",
              inner.len()
            );
          }
        }
      }
      unknown_expression => Err(
        error::InternalError::UnxpectedExpression(
          "build_named_expressions",
          format!("{unknown_expression:?}"),
        )
        .into(),
      ),
    }
  }

  fn build_labels(pair: pest::iterators::Pair<Rule>) -> Result<ast::LabelExpression>
  {
    match pair.as_rule()
    {
      Rule::labels => Self::build_labels(pair.into_inner().try_next()?),
      Rule::label_alternative =>
      {
        let mut r = ast::LabelExpression::None;
        let inner = pair.into_inner();
        for next in inner
        {
          r = r.or(Self::build_labels(next)?);
        }
        Ok(r)
      }
      Rule::label_inclusion =>
      {
        let mut r = ast::LabelExpression::None;
        let inner = pair.into_inner();
        for next in inner
        {
          r = r.and(Self::build_labels(next)?);
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

  fn build_node_pattern(&self, pair: pest::iterators::Pair<Rule>) -> Result<ast::NodePattern>
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
          labels = Self::build_labels(pair)?;
        }
        Rule::map => properties = Some(self.build_map(pair)?),
        Rule::parameter =>
        {
          properties = Some(ast::Expression::Parameter(ast::Parameter {
            name: pair.as_str().to_string(),
          }))
        }
        unknown_expression =>
        {
          return Err(
            error::InternalError::UnxpectedExpression(
              "build_node_pattern",
              format!("{unknown_expression:?}"),
            )
            .into(),
          );
        }
      }
    }
    Ok(ast::NodePattern {
      variable: self.var_ids.create_variable_from_name_optional(variable),
      labels,
      properties,
    })
  }

  fn build_edge_pattern(
    &self,
    source_node: ast::NodePattern,
    edge_pair: pest::iterators::Pair<Rule>,
    destination_node: ast::NodePattern,
    allow_undirected_edge: bool,
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
          labels = Self::build_labels(pair)?;
        }
        Rule::map => properties = Some(self.build_map(pair)?),
        Rule::parameter =>
        {
          properties = Some(ast::Expression::Parameter(ast::Parameter {
            name: pair.as_str().to_string(),
          }))
        }
        unknown_expression =>
        {
          return Err(
            error::InternalError::UnxpectedExpression(
              "build_edge_pattern",
              format!("{unknown_expression:?}"),
            )
            .into(),
          );
        }
      }
    }

    match edge_rule
    {
      Rule::directed_edge_pattern => Ok(ast::EdgePattern {
        variable: self.var_ids.create_variable_from_name_optional(variable),
        source: source_node,
        destination: destination_node,
        directivity: graph::EdgeDirectivity::Directed,
        labels,
        properties,
      }),
      Rule::reversed_edge_pattern => Ok(ast::EdgePattern {
        variable: self.var_ids.create_variable_from_name_optional(variable),
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
          variable: self.var_ids.create_variable_from_name_optional(variable),
          source: source_node,
          destination: destination_node,
          directivity: graph::EdgeDirectivity::Undirected,
          labels,
          properties,
        })
      }
      unknown_expression => Err(
        error::InternalError::UnxpectedExpression(
          "build_pattern/edge_pattern",
          format!("{unknown_expression:?}"),
        )
        .into(),
      ),
    }
  }

  fn build_patterns(
    &self,
    iterator: &mut pest::iterators::Pairs<Rule>,
    allow_undirected_edge: bool,
  ) -> Result<Vec<ast::Pattern>>
  {
    let mut vec = vec![];

    for pair in iterator
    {
      vec.append(&mut self.build_pattern(pair, allow_undirected_edge)?);
    }
    Ok(vec)
  }

  fn build_pattern(
    &self,
    pair: pest::iterators::Pair<Rule>,
    allow_undirected_edge: bool,
  ) -> Result<Vec<ast::Pattern>>
  {
    let mut vec = vec![];

    match pair.as_rule()
    {
      Rule::node_pattern =>
      {
        vec.push(ast::Pattern::Node(
          self.build_node_pattern(pair.into_inner().try_next()?)?,
        ));
      }
      Rule::edge_pattern =>
      {
        let mut it = pair.into_inner().peekable();
        let mut source_node = self.build_node_pattern(it.try_next()?)?;

        while let Some(next) = it.next()
        {
          let mut destination_node = self.build_node_pattern(it.try_next()?)?;

          if it.peek().is_some() && destination_node.variable.is_none()
          {
            destination_node.variable = Some(self.var_ids.create_anonymous_variable());
          }

          let edge_pattern = self.build_edge_pattern(
            source_node,
            next,
            destination_node.clone(),
            allow_undirected_edge,
          )?;
          vec.push(ast::Pattern::Edge(edge_pattern));
          source_node = destination_node;
        }
      }
      Rule::path_pattern =>
      {
        let mut it = pair.into_inner();
        let variable = it.try_next()?.as_str().to_string();
        let source_node = self.build_node_pattern(it.try_next()?)?;
        let edge_it = it.try_next()?;
        let destination_node = self.build_node_pattern(it.try_next()?)?;

        let edge_pattern = self.build_edge_pattern(
          source_node,
          edge_it,
          destination_node,
          allow_undirected_edge,
        )?;
        vec.push(ast::Pattern::Path(ast::PathPattern {
          variable: self.var_ids.create_variable_from_name(variable),
          edge: edge_pattern,
        }));
      }
      unknown_expression =>
      {
        return Err(
          error::InternalError::UnxpectedExpression(
            "build_node_or_edge_vec",
            format!("{unknown_expression:?}"),
          )
          .into(),
        );
      }
    };
    Ok(vec)
  }

  fn build_match(&self, pair: pest::iterators::Pair<Rule>, optional: bool)
    -> Result<ast::Statement>
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
          where_expression =
            Some(self.build_expression(pair.into_inner().try_next()?.into_inner())?)
        }
        _ => patterns.append(&mut self.build_pattern(pair, true)?),
      }
    }

    Ok(ast::Statement::Match(ast::Match {
      patterns,
      where_expression,
      optional,
    }))
  }

  fn build_return_with_statement(
    &self,
    pairs: pest::iterators::Pairs<Rule>,
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
        Rule::named_expression => expressions.push(self.build_named_expression(sub_pair)?),
        Rule::modifiers => modifiers = self.build_modifiers(sub_pair)?,
        Rule::where_modifier =>
        {
          where_expression =
            Some(self.build_expression(sub_pair.into_inner().try_next()?.into_inner())?)
        }
        _ => Err(InternalError::UnexpectedPair {
          context: "build_ast_from_statement/with_statement",
          pair: sub_pair.as_str().to_string(),
        })?,
      }
    }
    Ok((all, expressions, modifiers, where_expression))
  }
  fn build_ident(&self, iterator: &mut pest::iterators::Pairs<Rule>) -> Result<String>
  {
    let pair = iterator.next().ok_or_else(|| InternalError::MissingPair {
      context: "build_ident",
    })?;
    match pair.as_rule()
    {
      Rule::ident => Ok(pair.as_str().to_string()),
      _ => Err(
        InternalError::UnexpectedPair {
          context: "build_ident",
          pair: pair.to_string(),
        }
        .into(),
      ),
    }
  }
  fn build_ast_from_statement(&self, pair: pest::iterators::Pair<Rule>) -> Result<ast::Statement>
  {
    match pair.as_rule()
    {
      Rule::create_graph_statement => Ok(ast::Statement::CreateGraph(ast::CreateGraph {
        name: self.build_ident(&mut pair.into_inner())?,
      })),
      Rule::drop_graph_statement => Ok(ast::Statement::DropGraph(ast::DropGraph {
        name: self.build_ident(&mut pair.into_inner())?,
        if_exists: false,
      })),
      Rule::drop_graph_if_exists_statement => Ok(ast::Statement::DropGraph(ast::DropGraph {
        name: self.build_ident(&mut pair.into_inner())?,
        if_exists: true,
      })),
      Rule::use_graph_statement => Ok(ast::Statement::UseGraph(ast::UseGraph {
        name: self.build_ident(&mut pair.into_inner())?,
      })),
      Rule::create_statement => Ok(ast::Statement::Create(ast::Create {
        patterns: self.build_patterns(&mut pair.into_inner(), false)?,
      })),
      Rule::match_statement => self.build_match(pair, false),
      Rule::optional_match_statement => self.build_match(pair.into_inner().try_next()?, true),
      Rule::return_statement =>
      {
        let (all, expressions, modifiers, where_expression) =
          self.build_return_with_statement(pair.into_inner())?;

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
          self.build_return_with_statement(pair.into_inner())?;

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
          Rule::named_expression => self.build_named_expression(pair),
          _ => Err(
            InternalError::UnexpectedPair {
              context: "build_ast_from_statement/with_statement",
              pair: pair.as_str().to_string(),
            }
            .into(),
          ),
        }?;
        Ok(ast::Statement::Unwind(ast::Unwind {
          identifier: ne.identifier,
          expression: ne.expression,
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
          .map(|pair| self.build_expression(pair.into_inner()))
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
              let target = self
                .var_ids
                .create_variable_from_name(pair_left.try_next()?.as_str());
              let path = pair_left.map(|el| el.as_str().to_string()).collect();
              let expression = self.build_expression(pair.try_next()?.into_inner())?;
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
              let target = self
                .var_ids
                .create_variable_from_name(pair.try_next()?.as_str());
              let labels = pair.map(|el| el.as_str().to_string()).collect();
              updates.push(ast::OneUpdate::AddLabels(ast::AddRemoveLabels {
                target,
                labels,
              }));
            }
            unknown_expression => Err(error::InternalError::UnxpectedExpression(
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
              let target = self
                .var_ids
                .create_variable_from_name(pair.try_next()?.as_str());
              let path = pair.map(|el| el.as_str().to_string()).collect();
              updates.push(ast::OneUpdate::RemoveProperty(ast::RemoveProperty {
                target,
                path,
              }));
            }
            Rule::set_label_expression =>
            {
              let mut pair = pair.into_inner();
              let target = self
                .var_ids
                .create_variable_from_name(pair.try_next()?.as_str());
              let labels = pair.map(|el| el.as_str().to_string()).collect();
              updates.push(ast::OneUpdate::RemoveLabels(ast::AddRemoveLabels {
                target,
                labels,
              }));
            }
            unknown_expression => Err(error::InternalError::UnxpectedExpression(
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
          name,
          arguments: Default::default(),
        }))
      }
      unknown_expression => Err(
        error::InternalError::UnxpectedExpression(
          "build_ast_from_statement",
          format!("{unknown_expression:?}"),
        )
        .into(),
      ),
    }
  }
}

pub(crate) fn parse(input: &str) -> Result<ast::Queries>
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
        | Op::infix(Rule::modulo, Assoc::Left)
        | Op::infix(Rule::exponent, Assoc::Left),
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
  let ast_builder = AstBuilder {
    pratt,
    var_ids: Default::default(),
  };
  let pairs = GQLParser::parse(Rule::queries, input)?;
  let mut queries = Vec::<ast::Statements>::new();
  if crate::consts::SHOW_PARSE_TREE
  {
    println!("pairs = {:#?}", pairs);
  }
  for q_pair in pairs
  {
    match q_pair.as_rule()
    {
      Rule::query =>
      {
        let mut stmts = ast::Statements::new();

        for pair in q_pair.into_inner()
        {
          match pair.as_rule()
          {
            Rule::statement =>
            {
              stmts.push(ast_builder.build_ast_from_statement(pair.into_inner().try_next()?)?);
            }
            unknown_expression =>
            {
              Err(error::InternalError::UnxpectedExpression(
                "parse",
                format!("{unknown_expression:?}"),
              ))?;
            }
          }
        }
        queries.push(stmts);
      }
      Rule::EOI =>
      {}
      unknown_expression =>
      {
        Err(error::InternalError::UnxpectedExpression(
          "parse",
          format!("{unknown_expression:?}"),
        ))?;
      }
    }
  }
  if crate::consts::SHOW_AST
  {
    println!("ast = {:#?}", &queries);
  }
  Ok(queries)
}
