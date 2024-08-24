use ast::Expression;
use pest::Parser;
use pest_derive::Parser;

use crate::{graph, Result};

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

fn build_labels(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<String>>
{
  let mut vec = vec![];
  while let Some(pair) = iterator.next()
  {
    vec.push(pair.as_str().to_string());
  }
  Ok(vec)
}

fn build_node_pattern(pair: pest::iterators::Pair<Rule>) -> Result<ast::GraphNode>
{
  let it = pair.into_inner();
  let mut variable = None;
  let mut labels = Vec::new();
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
        labels = build_labels(pair.into_inner())?;
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
  Ok(ast::GraphNode {
    variable,
    labels,
    properties,
  })
}

fn build_edge_pattern(
  pair: pest::iterators::Pair<Rule>,
) -> Result<(Option<String>, Option<String>, Option<Expression>)>
{
  let it = pair.into_inner();
  let mut variable = None;
  let mut label = None;
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
        label = Some(pair.into_inner().as_str().to_string());
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
  Ok((variable, label, properties))
}

fn build_pattern(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<ast::Pattern>>
{
  let mut vec = vec![];

  while let Some(pair) = iterator.next()
  {
    match pair.as_rule()
    {
      Rule::node_pattern =>
      {
        vec.push(ast::Pattern::GraphNode(build_node_pattern(
          pair.into_inner().next().unwrap(),
        )?));
      }
      Rule::edge_pattern =>
      {
        println!("()-[]->() ->->->->->->-> pair: {:?}", pair);
        pair
          .clone()
          .into_inner()
          .for_each(|p| println!("     {:?}", p));
        let mut it = pair.into_inner();
        let source_node = build_node_pattern(it.next().unwrap())?;
        let edge_pattern = build_edge_pattern(it.next().unwrap())?;
        let destination_node = build_node_pattern(it.next().unwrap())?;
        vec.push(ast::Pattern::GraphEdge(ast::GraphEdge {
          variable: edge_pattern.0,
          source: source_node,
          destination: destination_node,
          directivity: ast::EdgeDirectivity::Directed,
          label: edge_pattern.1,
          properties: edge_pattern.2,
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
      patterns: build_pattern(pair.into_inner())?,
    })),
    Rule::match_statement => Ok(ast::Statement::Match(ast::Match {
      where_expression: None,
      patterns: build_pattern(pair.into_inner())?,
      optional: false,
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
  println!("\n\n\n{:?}\n\n\n", input);
  let pairs = GQLParser::parse(Rule::query, input)?;
  let mut stmts = ast::Statements::new();
  println!("{:?}", pairs);
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
  println!("{:?}", &stmts);
  Ok(stmts)
}
