use pest::Parser;
use pest_derive::Parser;

pub(crate) mod ast;

#[derive(Parser)]
#[grammar = "parser/gql.pest"]
pub(crate) struct GQLParser;

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
