use crate::{
  parser::{self, parse},
  tests::templates::ast,
};

fn compare_ast(actual: Vec<parser::ast::Statement>, expected: Vec<parser::ast::Statement>)
{
  assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
}

#[test]
fn test_parse_simple_create_node()
{
  let ast = parse("CREATE ()").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::simple_create_node())
}

#[test]
fn test_parse_create_named_node()
{
  let ast = parse("CREATE (n {name: 'foo'}) RETURN n.name AS p").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::create_named_node())
}

#[test]
fn test_parse_create_named_node_double_return()
{
  let ast = parse("CREATE (n {id: 12, name: 'foo'}) RETURN n.id AS id, n.name AS p").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::create_named_node_double_return())
}

#[test]
fn test_parse_double_with_return()
{
  let ast = parse("WITH 1 AS n, 2 AS m WITH n AS a, m AS b RETURN a").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::double_with_return())
}

#[test]
fn test_parse_unwind()
{
  let ast = parse("UNWIND [0] AS i RETURN i").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::unwind())
}

#[test]
fn test_parse_match_loop()
{
  let ast = parse("MATCH (n)-[]->(n) RETURN n").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::match_loop());
}

#[test]
fn test_parse_optional_match()
{
  let ast = parse("OPTIONAL MATCH (a) RETURN a").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::optional_match());
}

#[test]
fn test_parse_match_count()
{
  let ast = parse("MATCH (a) RETURN count(*)").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::match_count());
}

#[test]
fn test_parse_aggregation()
{
  let ast = parse("MATCH (n) RETURN n.name, count(n.num)").unwrap();
  let ast = ast.into_iter().next().unwrap();
  compare_ast(ast, ast::aggregation());
}
