use crate::{
  compiler::compile,
  functions,
  interpreter::Program,
  tests::templates::{ast, programs},
};

fn compare_program(actual: Program, expected: Program)
{
  assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
}

#[test]
fn test_compile_simple_create_node()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::simple_create_node()).unwrap();
  compare_program(program, programs::simple_create())
}

#[test]
fn test_compile_create_named_node()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::create_named_node()).unwrap();
  compare_program(program, programs::create_named_node())
}
