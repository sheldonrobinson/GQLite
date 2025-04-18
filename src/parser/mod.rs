use pest::Parser;
use pest_derive::Parser;

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

fn build_expression(pair: pest::iterators::Pair<Rule>) -> crate::Result<ast::Expression>
{
  match pair.as_rule() {
    Rule::ident => {
      Ok(ast::Expression::Variable(ast::Variable {
        identifier: pair.as_str().to_string()
      }))
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression("build_expression",
      format!("{unknown_expression:?}")))
  }
}

fn build_named_expression(pair: pest::iterators::Pair<Rule>) -> crate::Result<ast::NamedExpression>
{
  match pair.as_rule() {
    Rule::named_expression => {
      let mut inner = pair.into_inner();
      match inner.len()
      {
        1 => {
          let expr = inner.next().unwrap();
          Ok(
            ast::NamedExpression {
              name: expr.as_str().to_string(),
              expression: build_expression(expr)?
            }
          )
        }
        2 => {
          Ok(
            ast::NamedExpression {
              name: inner.next().unwrap().as_str().to_string(),
              expression: build_expression(inner.next().unwrap())?
            }
          )
        }
        _ => {
          panic!("Invalid number of terms in named expressions {}", inner.len());
        }
      }
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression("build_named_expressions",
      format!("{unknown_expression:?}")))
  }
}

fn build_ast_from_statement(pair: pest::iterators::Pair<Rule>) -> crate::Result<ast::Statement>
{
  match pair.as_rule() {
    Rule::create_node => {
      let variable = pair.into_inner().next().unwrap().as_str();
      Ok(ast::Statement::Create(ast::Create {
        patterns: vec![
          ast::GraphNodeOrEdge::GraphNode(ast::GraphNode {
            variable: Some(variable.to_string()),
            labels: vec![],
            properties: None,
          })
        ]
      }))
    },
    Rule::match_node => {
      let variable = pair.into_inner().next().unwrap().as_str();
      Ok(ast::Statement::Match(ast::Match {
        where_expression: None,
        patterns:  vec![
          ast::GraphNodeOrEdge::GraphNode(ast::GraphNode {
            variable: Some(variable.to_string()),
            labels: vec![],
            properties: None,
          })
        ],
        optional: false
      }))
    }
    Rule::return_statement => {
      let named_expressions = pair
        .into_inner().map(|pair| build_named_expression(pair))
        .collect::<crate::Result<Vec<ast::NamedExpression>>>()?;
      Ok(ast::Statement::Return(ast::Return{
        all: false,
        expressions: named_expressions,
        modifiers: ast::Modifiers::default(),
      }))
    }
    unknown_expression => Err(crate::Error::UnxpectedExpression("build_ast_from_statement",
      format!("{unknown_expression:?}")))
  }
}

pub(crate) fn parse(input: &str) -> crate::Result<ast::Statements>
{
  let pairs = GQLParser::parse(Rule::query, input)?;
  let mut stmts = ast::Statements::new();

  for pair in pairs
  {
    match pair.as_rule()
    {
      Rule::statement => {
        stmts.push(build_ast_from_statement(pair.into_inner().next().unwrap())?);
      }
      Rule::EOI => {}
      unknown_expression => {
        Err(crate::Error::UnxpectedExpression("parse", format!("{unknown_expression:?}")))?;
      }
    }
  }
  Ok(stmts)
}
