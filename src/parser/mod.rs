use std::env::var;

use ast::Expression;
use pest::{error::Error, Parser};
use pest_derive::Parser;

use crate::{graph, Result};

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

fn build_pair(pair: pest::iterators::Pair<Rule>) -> Result<(String, ast::Expression)> {
  let mut it = pair.into_inner();
  let k = it.next().unwrap();
  let v = build_expression(it.next().unwrap())?;
  return Ok((k.as_str().to_string(), v));
}

fn build_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression> {
  match pair.as_rule() {
    Rule::ident => Ok(ast::Expression::Variable(ast::Variable {
      identifier: pair.as_str().to_string(),
    })),
    Rule::map => Ok(ast::Expression::Map({
      let mut map = std::collections::HashMap::new();
      for k_v_pair in pair.into_inner() {
        let (k, v) = build_pair(k_v_pair)?;
        map.insert(k, v);
      }
      ast::Map { map: map }
    })),
    Rule::string_literal => Ok(ast::Expression::Value(ast::Value {
      value: graph::Value::String(pair.into_inner().next().unwrap().as_str().to_string()),
    })),
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_expression",
      format!("{unknown_expression:?}"),
    )),
  }
}

fn build_named_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::NamedExpression> {
  match pair.as_rule() {
    Rule::named_expression => {
      let mut inner = pair.into_inner();
      match inner.len() {
        1 => {
          let expr = inner.next().unwrap();
          Ok(ast::NamedExpression {
            name: expr.as_str().to_string(),
            expression: build_expression(expr)?,
          })
        }
        2 => Ok(ast::NamedExpression {
          name: inner.next().unwrap().as_str().to_string(),
          expression: build_expression(inner.next().unwrap())?,
        }),
        _ => {
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

fn build_labels(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<String>> {
  let mut vec = vec![];
  while let Some(pair) = iterator.next() {
    vec.push(pair.as_str().to_string());
  }
  Ok(vec)
}

fn build_node_pattern(pair: pest::iterators::Pair<Rule>) -> Result<ast::GraphNode> {
  let mut it = pair.into_inner();
  let variable = it.next().map(|x| x.as_str().to_string());
  let labels = if let Some(labels_it) = it.next() {
    println!("labels: {:?}", labels_it);
    build_labels(labels_it.into_inner())?
  } else {
    vec![]
  };
  let properties = if let Some(properties_it) = it.next() {
    Some(build_expression(properties_it)?)
  } else {
    None
  };
  Ok(ast::GraphNode {
    variable: variable,
    labels: labels,
    properties: properties,
  })
}

fn build_edge_pattern(
  pair: pest::iterators::Pair<Rule>,
) -> Result<(Option<String>, Option<String>, Option<Expression>)> {
  let mut it = pair.into_inner();
  let variable = it.next().map(|x| x.as_str().to_string());
  let label = if let Some(label) = it.next() {
    Some(label.as_str().to_string())
  } else {
    None
  };
  let properties = if let Some(properties_it) = it.next() {
    Some(build_expression(properties_it)?)
  } else {
    None
  };

  Ok((variable, label, properties))
}

fn build_pattern(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<ast::Pattern>> {
  let mut vec = vec![];

  while let Some(pair) = iterator.next() {
    match pair.as_rule() {
      Rule::node_pattern => {
        vec.push(ast::Pattern::GraphNode(build_node_pattern(
          pair.into_inner().next().unwrap(),
        )?));
      }
      Rule::edge_pattern => {
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
      unknown_expression => {
        return Err(crate::Error::UnxpectedExpression(
          "build_node_or_edge_vec",
          format!("{unknown_expression:?}"),
        ));
      }
    }
  }

  Ok(vec)
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> Result<ast::Statement> {
  match pair.as_rule() {
    Rule::create_statement => Ok(ast::Statement::Create(ast::Create {
      patterns: build_pattern(pair.into_inner())?,
    })),
    Rule::match_statement => Ok(ast::Statement::Match(ast::Match {
      where_expression: None,
      patterns: build_pattern(pair.into_inner())?,
      optional: false,
    })),
    Rule::return_statement => {
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
    unknown_expression => Err(crate::Error::UnxpectedExpression(
      "build_ast_from_statement",
      format!("{unknown_expression:?}"),
    )),
  }
}

pub(crate) fn parse(input: &str) -> Result<ast::Statements> {
  println!("\n\n\n{:?}\n\n\n", input);
  let pairs = GQLParser::parse(Rule::query, input)?;
  let mut stmts = ast::Statements::new();
  println!("{:?}", pairs);
  for pair in pairs {
    match pair.as_rule() {
      Rule::statement => {
        stmts.push(build_ast_from_statement(pair.into_inner().next().unwrap())?);
      }
      Rule::EOI => {}
      unknown_expression => {
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
