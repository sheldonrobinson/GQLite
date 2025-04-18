use ast::{Expression, LabelExpression};
use pest::Parser;
use pest_derive::Parser;

use crate::{error::CompileTimeError, error::InternalError, graph, Result};

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

fn build_pair(pair: pest::iterators::Pair<Rule>) -> Result<(String, ast::Expression)>
{
  let mut it = pair.into_inner();
  let k = it.next().unwrap();
  let v = build_expression(it.next().unwrap())?;
  return Ok((k.as_str().to_string(), v));
}

fn build_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression>
{
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
    Rule::ident => Ok(ast::Expression::Variable(ast::Variable {
      identifier: pair.as_str().to_string(),
    })),
    Rule::parameter => Ok(ast::Expression::Parameter(ast::Parameter {
      name: pair.as_str().to_string(),
    })),
    Rule::array => Ok(ast::Expression::Array(ast::Array {
      array: pair
        .into_inner()
        .map(|pair| build_expression(pair))
        .collect::<Result<Vec<ast::Expression>>>()?,
    })),
    Rule::map => Ok(ast::Expression::Map({
      let mut map = std::collections::HashMap::new();
      for k_v_pair in pair.into_inner()
      {
        let (k, v) = build_pair(k_v_pair)?;
        map.insert(k, v);
      }
      ast::Map { map: map }
    })),
    Rule::member_access =>
    {
      let mut it = pair.into_inner();
      let left =
        build_expression(it.next().ok_or_else(|| {
          crate::Error::InternalError("Missing first element of member access.")
        })?)?;
      Ok(ast::Expression::MemberAccess(Box::new(ast::MemberAccess {
        left,
        path: it.map(|el| el.as_str().to_string()).collect(),
      })))
    }
    Rule::string_literal => Ok(ast::Expression::Value(ast::Value {
      value: graph::Value::String(pair.into_inner().next().unwrap().as_str().to_string()),
    })),
    Rule::num =>
    {
      let mut it = pair.into_inner();
      let num_str = it
        .next()
        .ok_or_else(|| crate::Error::InternalError("Missing first element of number."))?
        .as_str();
      match it.next()
      {
        Some(frag) =>
        {
          let num_str = num_str.to_owned() + frag.as_str();
          match it.next()
          {
            Some(frag) => Ok(ast::Expression::Value(ast::Value {
              value: graph::Value::Float((num_str + frag.as_str()).as_str().parse()?),
            })),
            None => Ok(ast::Expression::Value(ast::Value {
              value: graph::Value::Float(num_str.as_str().parse()?),
            })),
          }
        }
        None => Ok(ast::Expression::Value(ast::Value {
          value: graph::Value::Integer(num_str.parse()?),
        })),
      }
    }
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
          .map(|pair| build_expression(pair))
          .collect::<Result<Vec<ast::Expression>>>()?,
      }))
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_expression",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_named_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::NamedExpression>
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
          let expr = inner.next().unwrap();
          Ok(ast::NamedExpression {
            name: expr.as_str().to_string(),
            expression: build_expression(expr)?,
          })
        }
        2 =>
        {
          let expression = build_expression(inner.next().unwrap())?;
          let name = inner.next().unwrap().as_str().to_string();
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
    Rule::labels => build_labels(pair.into_inner().next().unwrap()),
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

fn build_node_pattern(pair: pest::iterators::Pair<Rule>) -> Result<ast::NodePattern>
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
      Rule::map => properties = Some(build_expression(pair)?),
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
  pair: pest::iterators::Pair<Rule>,
) -> Result<(Option<String>, LabelExpression, Option<Expression>)>
{
  let it = pair.into_inner();
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
      Rule::map => properties = Some(build_expression(pair)?),
      unknown_expression =>
      {
        return Err(crate::Error::UnxpectedExpression(
          "build_edge_pattern",
          format!("{unknown_expression:?}"),
        ));
      }
    }
  }
  Ok((variable, labels, properties))
}

fn build_pattern(
  mut iterator: pest::iterators::Pairs<Rule>,
  allow_undirected_edge: bool,
) -> Result<Vec<ast::Pattern>>
{
  let mut vec = vec![];

  while let Some(pair) = iterator.next()
  {
    match pair.as_rule()
    {
      Rule::node_pattern =>
      {
        vec.push(ast::Pattern::Node(build_node_pattern(
          pair.into_inner().next().unwrap(),
        )?));
      }
      Rule::edge_pattern =>
      {
        let mut it = pair.into_inner();
        let mut source_node = build_node_pattern(it.next().unwrap())?;

        while let Some(next) = it.next()
        {
          let rule = next.as_rule();
          let mut it_edge = next.into_inner();
          let edge_pattern = build_edge_pattern(it_edge.next().unwrap())?;
          let destination_node = build_node_pattern(it.next().unwrap())?;

          match rule
          {
            Rule::directed_edge_pattern =>
            {
              vec.push(ast::Pattern::Edge(ast::EdgePattern {
                variable: edge_pattern.0,
                source: source_node,
                destination: destination_node.clone(),
                directivity: graph::EdgeDirectivity::Directed,
                labels: edge_pattern.1,
                properties: edge_pattern.2,
              }));
            }
            Rule::reversed_edge_pattern =>
            {
              vec.push(ast::Pattern::Edge(ast::EdgePattern {
                variable: edge_pattern.0,
                source: destination_node.clone(),
                destination: source_node,
                directivity: graph::EdgeDirectivity::Directed,
                labels: edge_pattern.1,
                properties: edge_pattern.2,
              }));
            }
            Rule::undirected_edge_pattern =>
            {
              if !allow_undirected_edge
              {
                Err(CompileTimeError::RequiresDirectedRelationship {
                  context: "creation",
                })?;
              }
              vec.push(ast::Pattern::Edge(ast::EdgePattern {
                variable: edge_pattern.0,
                source: source_node,
                destination: destination_node.clone(),
                directivity: graph::EdgeDirectivity::Undirected,
                labels: edge_pattern.1,
                properties: edge_pattern.2,
              }));
            }
            unknown_expression =>
            {
              return Err(crate::Error::UnxpectedExpression(
                "build_pattern/edge_pattern",
                format!("{unknown_expression:?}"),
              ));
            }
          }
          source_node = destination_node;
        }
      }
      Rule::path_pattern =>
      {
        let mut it = pair.into_inner();
        let variable = it.next().unwrap().as_str().to_string();
        let mut it = it.next().unwrap().into_inner();
        let source_node = build_node_pattern(it.next().unwrap())?;
        let edge_pattern = build_edge_pattern(it.next().unwrap())?;
        let destination_node = build_node_pattern(it.next().unwrap())?;
        vec.push(ast::Pattern::Path(ast::PathPattern {
          variable,
          edge: ast::EdgePattern {
            variable: edge_pattern.0,
            source: source_node,
            destination: destination_node,
            directivity: graph::EdgeDirectivity::Directed,
            labels: edge_pattern.1,
            properties: edge_pattern.2,
          },
        }));
      }

      unknown_expression =>
      {
        return Err(crate::Error::UnxpectedExpression(
          "build_node_or_edge_vec",
          format!("{unknown_expression:?}"),
        ));
      }
    }
  }

  Ok(vec)
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> Result<ast::Statement>
{
  match pair.as_rule()
  {
    Rule::create_statement => Ok(ast::Statement::Create(ast::Create {
      patterns: build_pattern(pair.into_inner(), false)?,
    })),
    Rule::match_statement => Ok(ast::Statement::Match(ast::Match {
      where_expression: None,
      patterns: build_pattern(pair.into_inner(), true)?,
      optional: false,
    })),
    Rule::optional_match_statement => Ok(ast::Statement::Match(ast::Match {
      where_expression: None,
      patterns: build_pattern(
        pair.into_inner().into_iter().next().unwrap().into_inner(),
        true,
      )?,
      optional: true,
    })),
    Rule::return_statement =>
    {
      let named_expressions = pair
        .into_inner()
        .map(|pair| build_named_expression(pair))
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
      let mut it = pair.into_inner();
      let first = it.next();
      match first
      {
        Some(pair) => match pair.as_rule()
        {
          Rule::star => all = true,
          Rule::named_expression => expressions.push(build_named_expression(pair)?),
          _ => Err(InternalError::UnexpectedPair {
            context: "build_ast_from_statement/with_statement",
            pair: pair.as_str().to_string(),
          })?,
        },
        _ => Err(InternalError::MissingPair {
          context: "build_ast_from_statement/with_statement",
        })?,
      }
      expressions.append(
        &mut it
          .map(|pair| build_named_expression(pair))
          .collect::<Result<Vec<ast::NamedExpression>>>()?,
      );

      Ok(ast::Statement::With(ast::With {
        all,
        expressions,
        modifiers: ast::Modifiers::default(),
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
        Rule::named_expression => build_named_expression(pair),
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
        stmts.push(build_ast_from_statement(pair.into_inner().next().unwrap())?);
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
