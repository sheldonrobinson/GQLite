#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]

mod common;
mod generate_module;
mod generate_rune_module;

/// Generate an ORM-like API according to a GQL Schema.
///
/// ```rust
/// generate_module!(module_name, "path/to/file.gqls");
/// ```
///
/// The path to the file is relative to the root of the crate.
#[proc_macro]
pub fn generate_module(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
  let parsed_input = syn::parse_macro_input!(input as generate_module::ParsedInput);

  match generate_module::generate_module_impl(parsed_input)
  {
    Ok(ts) => ts,
    Err(e) => e.into_compile_error(),
  }
  .into()
}

/// Generate an ORM-like API according to a GQL Schema, for Rune.
///
/// ```rust
/// generate_rune_module!(run_module_name, rust_module_name, "path/to/file.gqls");
/// ```
///
/// The path to the file is relative to the root of the crate.
#[proc_macro]
pub fn generate_rune_module(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
  let parsed_input = syn::parse_macro_input!(input as generate_rune_module::ParsedInput);

  match generate_rune_module::generate_rune_module_impl(parsed_input)
  {
    Ok(ts) => ts,
    Err(e) => e.into_compile_error(),
  }
  .into()
}
