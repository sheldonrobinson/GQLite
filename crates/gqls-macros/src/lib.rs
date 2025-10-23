#![doc = include_str!("../README.MD")]
#![warn(missing_docs)]
mod generate_module;

/// This funcion is used to find if this macro is invoked from `gqls` and must be refered as `crate`
/// or if it is invoked from a different crate, and must be referred as `gqls`.
fn my_crate() -> proc_macro2::TokenStream
{
  use proc_macro_crate::{crate_name, FoundCrate};
  let found_crate = crate_name("gqls").expect("gqls is present in `Cargo.toml`");

  match found_crate
  {
    FoundCrate::Itself => quote::quote!(crate),
    FoundCrate::Name(name) =>
    {
      let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
      quote::quote!( #ident )
    }
  }
}

/// Generate an API according to a GQL Schema.
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
