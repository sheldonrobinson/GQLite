use indexmap::IndexMap;
use nom::{
  branch::alt,
  bytes::complete::{is_not, tag, take_while1},
  character::complete::multispace0,
  combinator::{cut, map, opt, value},
  error::ParseError,
  multi::{many0, separated_list0},
  sequence::{delimited, preceded, terminated},
  AsChar, IResult, Input, Parser,
};
use nom_language::error::VerboseError;

type SResult<I, O> = IResult<I, O, VerboseError<I>>;

use crate::gqls::prelude::*;

#[derive(Debug)]
pub(crate) enum ParseTreeElement
{
  Property
  {
    label: String,
    properties: IndexMap<String, Property>,
    parent: Option<String>,
  },
  Node
  {
    label: String
  },
  Edge
  {
    source: String,
    label: String,
    destination: String,
  },
}

fn ws<I, E: ParseError<I>, F>(
  f: F,
) -> impl Parser<I, Output = <F as Parser<I>>::Output, Error = E>
where
  F: Parser<I, Error = E>,
  I: Input,
  <I as Input>::Item: AsChar,
{
  preceded(multispace0, f)
}

#[inline]
fn is_label(chr: char) -> bool
{
  chr.is_alphanumeric() || chr == '_'
}

fn parse_label(input: &str) -> SResult<&str, &str>
{
  take_while1(is_label).parse(input)
}

#[inline]
fn is_identifier(chr: char) -> bool
{
  chr.is_alphanumeric() || chr == '_'
}

fn parse_identifier(input: &str) -> SResult<&str, &str>
{
  take_while1(is_identifier).parse(input)
}

fn parse_property(input: &str) -> SResult<&str, Property>
{
  map(
    (
      alt((
        map(tag("STRING"), |_| {
          Property::Literal(LiteralBaseType::String.into())
        }),
        map(tag("FLOAT"), |_| {
          Property::Literal(LiteralBaseType::Float.into())
        }),
        map(tag("INTEGER"), |_| {
          Property::Literal(LiteralBaseType::Integer.into())
        }),
        map(tag("BOOLEAN"), |_| {
          Property::Literal(LiteralBaseType::Boolean.into())
        }),
        map(tag("TIMESTAMP"), |_| {
          Property::Literal(LiteralBaseType::TimeStamp.into())
        }),
        map(parse_property_map, |property_map| {
          Property::Map(property_map)
        }),
        map((tag("["), parse_property, tag("]")), |(_, property, _)| {
          Property::Array(Box::new(property))
        }),
      )),
      opt(ws(tag("?"))),
    ),
    |(p, q)| match q
    {
      Some(_) => Property::Optional(Box::new(p)),
      None => p,
    },
  )
  .parse(input)
}

fn parse_property_map(input: &str) -> SResult<&str, IndexMap<String, Property>>
{
  map(
    (
      tag("{"),
      separated_list0(
        ws(tag(",")),
        map(
          (ws(parse_identifier), ws(tag(":")), ws(parse_property)),
          |(name, _, property)| (name.to_string(), property),
        ),
      ),
      ws(tag("}")),
    ),
    |(_, properties, _)| properties.into_iter().collect(),
  )
  .parse(input)
}

/// parse a property definition
fn parse_property_definition(input: &str) -> SResult<&str, ParseTreeElement>
{
  alt((
    map(
      (
        parse_label,
        ws(tag("<:")),
        ws(parse_label),
        ws(parse_property_map),
      ),
      |(label, _, parent, properties)| ParseTreeElement::Property {
        label: label.to_string(),
        properties,
        parent: Some(parent.to_string()),
      },
    ),
    map(
      (parse_label, ws(parse_property_map)),
      |(label, properties)| ParseTreeElement::Property {
        label: label.to_string(),
        properties,
        parent: None,
      },
    ),
  ))
  .parse(input)
}

fn parse_node_definition(input: &str) -> SResult<&str, ParseTreeElement>
{
  map(
    (tag("("), ws(parse_label), ws(tag(")"))),
    |(_, label, _)| ParseTreeElement::Node {
      label: label.to_string(),
    },
  )
  .parse(input)
}
fn parse_edge_definition(input: &str) -> SResult<&str, ParseTreeElement>
{
  map(
    (
      tag("("),
      ws(parse_label),
      ws(tag(")-[")),
      ws(parse_label),
      ws(tag("]->(")),
      ws(parse_label),
      ws(tag(")")),
    ),
    |(_, source, _, label, _, destination, _)| ParseTreeElement::Edge {
      source: source.to_string(),
      label: label.to_string(),
      destination: destination.to_string(),
    },
  )
  .parse(input)
}

fn single_line_comment(input: &str) -> SResult<&str, ()>
{
  value((), (multispace0, tag("//"), is_not("\n\r"), multispace0)).parse(input)
}

/// parse the schema
pub(crate) fn parse_schema(input: &str) -> SResult<&str, Vec<ParseTreeElement>>
{
  terminated(
    separated_list0(
      tag(","),
      cut(ws(delimited(
        many0(single_line_comment),
        alt((
          parse_property_definition,
          parse_edge_definition,
          parse_node_definition,
        )),
        many0(single_line_comment),
      ))),
    ),
    multispace0,
  )
  .parse(input)
}

#[cfg(test)]
mod tests
{
  #[test]
  fn test_single_line_comment()
  {
    assert!(super::single_line_comment("// hello").is_ok());
    assert!(super::single_line_comment("// hello\n").is_ok());
    assert!(super::single_line_comment("/ hello").is_err());
    assert!(super::single_line_comment(" // hello").is_ok());
    assert!(super::single_line_comment("\n // hello").is_ok());
  }
}
