use pest::Parser;
use pest_derive::Parser;

use crate::Result;

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

fn build_expression(pair: pest::iterators::Pair<Rule>) -> Result<ast::Expression> {
  match pair.as_rule() {
    Rule::ident => {
      Ok(
        ast::Expression::Variable(ast::Variable {
          identifier: pair.as_str().to_string(),
        })
      )
    }
    unknown_expression =>
      Err(crate::Error::UnxpectedExpression("build_expression", format!("{unknown_expression:?}"))),
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
        2 => {
          Ok(ast::NamedExpression {
            name: inner.next().unwrap().as_str().to_string(),
            expression: build_expression(inner.next().unwrap())?,
          })
        }
        _ => {
          panic!("Invalid number of terms in named expressions {}", inner.len());
        }
      }
    }
    unknown_expression =>
      Err(
        crate::Error::UnxpectedExpression(
          "build_named_expressions",
          format!("{unknown_expression:?}")
        )
      ),
  }
}

fn build_labels(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<String>> {
  println!("!!!!!!!!! {:?}", iterator);
  let mut vec = vec![];
  while let Some(pair) = iterator.next() {
    println!("------------------ {:?}", pair);
    vec.push(pair.as_str().to_string());
  }
  Ok(vec)
}

fn build_pattern(mut iterator: pest::iterators::Pairs<Rule>) -> Result<Vec<ast::Pattern>> {
  let mut vec = vec![];

  while let Some(pair) = iterator.next() {
    match pair.as_rule() {
      Rule::node_pattern => {
        println!("pair: {:?}", pair);
        let mut it = pair.into_inner().next().unwrap().into_inner();
        let variable = it.next().map(|x| x.as_str().to_string());
        let labels = if let Some(labels_it) = it.next() {
          println!("labels: {:?}", labels_it);
          build_labels(labels_it.into_inner())?
        } else {
          vec![]
        };
        vec.push(
          ast::Pattern::GraphNode(ast::GraphNode {
            variable: variable,
            labels: labels,
            properties: None,
          })
        );
      }
      unknown_expression => {
        return Err(
          crate::Error::UnxpectedExpression(
            "build_node_or_edge_vec",
            format!("{unknown_expression:?}")
          )
        );
      }
    }
  }

  Ok(vec)
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> Result<ast::Statement> {
  match pair.as_rule() {
    Rule::create_statement => {
      Ok(
        ast::Statement::Create(ast::Create {
          patterns: build_pattern(pair.into_inner())?,
        })
      )
    }
    Rule::match_statement => {
      Ok(
        ast::Statement::Match(ast::Match {
          where_expression: None,
          patterns: build_pattern(pair.into_inner())?,
          optional: false,
        })
      )
    }
    Rule::return_statement => {
      let named_expressions = pair
        .into_inner()
        .map(|pair| build_named_expression(pair))
        .collect::<Result<Vec<ast::NamedExpression>>>()?;
      Ok(
        ast::Statement::Return(ast::Return {
          all: false,
          expressions: named_expressions,
          modifiers: ast::Modifiers::default(),
        })
      )
    }
    unknown_expression =>
      Err(
        crate::Error::UnxpectedExpression(
          "build_ast_from_statement",
          format!("{unknown_expression:?}")
        )
      ),
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
        Err(crate::Error::UnxpectedExpression("parse", format!("{unknown_expression:?}")))?;
      }
    }
  }
  println!("{:?}", &stmts);
  Ok(stmts)
}
