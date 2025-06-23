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
  let create_ast = parse("CREATE ()").unwrap();
  compare_ast(create_ast, ast::simple_create_node())
}

#[test]
fn test_parse_create_named_node()
{
  let create_ast = parse("CREATE (n {name: 'foo'}) RETURN n.name AS p").unwrap();
  compare_ast(create_ast, ast::create_named_node())
}

#[test]
fn test_parse_create_named_node_double_return()
{
  let create_ast =
    parse("CREATE (n {id: 12, name: 'foo'}) RETURN n.id AS id, n.name AS p").unwrap();
  compare_ast(create_ast, ast::create_named_node_double_return())
}
