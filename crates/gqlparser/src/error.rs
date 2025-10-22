#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Error
{
  #[error("ParseError: {0}.")]
  Parse(String),
  #[error("IncompleteParsingError: '{0}' was not parsed.")]
  IncompleteParsing(String),
  #[error("UnknownPropertyDefinitionError: '{label}' was not defined.")]
  UnknownPropertyDefinition
  {
    label: String
  },
}
