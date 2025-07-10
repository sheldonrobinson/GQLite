use crate::{
  compiler::compile,
  interpreter::Program,
  prelude::*,
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

#[test]
fn test_compile_create_named_node_double_return()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::create_named_node_double_return()).unwrap();
  compare_program(program, programs::create_named_node_double_return())
}

#[test]
fn test_compile_double_with_return()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::double_with_return()).unwrap();
  compare_program(program, programs::double_with_return())
}

#[test]
fn test_compile_unwind()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::unwind()).unwrap();
  compare_program(program, programs::unwind())
}

#[test]
fn test_compile_match_loop()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::match_loop()).unwrap();
  compare_program(program, programs::match_loop())
}

#[test]
fn test_compile_optional_match()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::optional_match()).unwrap();
  compare_program(program, programs::optional_match())
}

#[test]
fn test_compile_match_count()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::match_count()).unwrap();
  compare_program(program, programs::match_count(&function_manager))
}

#[test]
fn test_compile_aggregation()
{
  let function_manager = functions::Manager::new();

  let program = compile(&function_manager, ast::aggregation()).unwrap();
  compare_program(program, programs::aggregation(&function_manager))
}
