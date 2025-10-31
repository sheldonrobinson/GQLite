use std::{env, path::Path};

use gqlparser::gqls::{ast::PropertiesDefinition, prelude::*};
use indexmap::IndexMap;
use proc_macro2::Span;

/// This funcion is used to find if this macro is invoked from `gqls` and must be refered as `crate`
/// or if it is invoked from a different crate, and must be referred as `gqls`.
pub(crate) fn my_crate() -> proc_macro2::TokenStream
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

pub(crate) fn parse_gqls_file(
  filename: &syn::LitStr,
) -> Result<gqlparser::gqls::ast::Ast, syn::Error>
{
  // Find gqls file, relative to CARGO_MANIFEST_DIR/src
  let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());

  let gqls_path = Path::new(&root).join("src/").join(filename.value());
  if !gqls_path.exists()
  {
    return Err(syn::Error::new(
      filename.span(),
      format!(
        "Cannot find: {:?} in src, make sure the path is relative to src.",
        filename.value()
      ),
    ));
  }

  let contents = std::fs::read_to_string(gqls_path);

  match contents
  {
    Ok(contents) =>
    {
      // Parse the file
      Ok(gqlparser::gqls::parse_schema(&contents).map_err(|e| {
        syn::Error::new(filename.span(), format!("Failed to parse schema: {:?}", e))
      })?)
    }
    Err(e) => Err(syn::Error::new(
      filename.span(),
      format!("Error while reading file '{}': '{}'.", filename.value(), e),
    )),
  }
}

pub(crate) struct ExpandedElement
{
  pub(crate) labels: Vec<String>,
  pub(crate) properties: IndexMap<String, Property>,
}

pub(crate) fn expand_element(
  elements: &IndexMap<String, PropertiesDefinition>,
  label: String,
) -> Result<ExpandedElement, syn::Error>
{
  let definition = elements.get(&label);

  match definition
  {
    Some(definition) =>
    {
      let mut labels = Vec::<String>::default();
      let mut properties = IndexMap::<String, Property>::default();

      for parent in definition.parents.iter()
      {
        let mut parent_ee = expand_element(elements, parent.clone())?;
        labels.append(&mut parent_ee.labels);
        properties.extend(parent_ee.properties.into_iter());
      }
      labels.push(label);
      properties.extend(definition.properties.clone());
      Ok(ExpandedElement { labels, properties })
    }
    None => Err(syn::Error::new(
      Span::call_site(),
      format!("UnknownPropertyDefinitionError: '{label}' was not defined."),
    )),
  }
}
