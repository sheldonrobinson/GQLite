//! GQL Schema parser

pub mod ast;
mod parser;
pub mod prelude;
pub mod properties;

use indexmap::IndexMap;
use nom::Finish;

use crate::prelude::*;
use prelude::*;

fn search_properties(
  properties_definitions: &IndexMap<String, (IndexMap<String, Property>, Option<String>)>,
  label: String,
  required: bool,
) -> Result<(Vec<String>, IndexMap<String, Property>)>
{
  let definition = properties_definitions.get(&label);

  match definition
  {
    Some((current_properties, Some(parent))) =>
    {
      let (mut labels, mut properties) =
        search_properties(properties_definitions, parent.to_owned(), true)?;
      labels.push(label);
      properties.extend(
        current_properties
          .iter()
          .map(|(k, v)| (k.to_owned(), v.to_owned())),
      );
      Ok((labels, properties))
    }
    Some((current_properties, None)) => Ok((vec![label], current_properties.to_owned())),
    None =>
    {
      if required
      {
        Err(Error::UnknownPropertyDefinition {
          label: label.clone(),
        })
      }
      else
      {
        Ok((vec![label], Default::default()))
      }
    }
  }
}

/// Parse a schmea into an AST.
pub fn parse_schema(input: &str) -> Result<ast::Ast>
{
  // Parse schema
  let (rem, parse_tree) = parser::parse_schema(input)
    .finish()
    .map_err(|e| Error::Parse(nom_language::error::convert_error(input, e)))?;

  if !rem.is_empty()
  {
    return Err(Error::IncompleteParsing(rem.to_string()));
  }

  // Convert to AST
  let mut nodes: Vec<ast::Node> = Default::default();
  let mut edges: Vec<ast::Edge> = Default::default();
  let mut properties_definitions: IndexMap<String, (IndexMap<String, Property>, Option<String>)> =
    Default::default();

  for element in parse_tree
  {
    match element
    {
      parser::ParseTreeElement::Property {
        label,
        properties,
        parent,
      } =>
      {
        properties_definitions.insert(label, (properties, parent));
      }
      parser::ParseTreeElement::Node { label } =>
      {
        let (labels, properties) =
          search_properties(&properties_definitions, label.clone(), false)?;
        nodes.push(ast::Node {
          identifier: label,
          labels,
          properties,
        });
      }
      parser::ParseTreeElement::Edge {
        source,
        label,
        destination,
      } =>
      {
        let (labels, properties) =
          search_properties(&properties_definitions, label.clone(), false)?;
        edges.push(ast::Edge {
          identifer: label,
          source,
          labels,
          properties,
          destination,
        });
      }
    }
  }

  Ok(ast::Ast { nodes, edges })
}

#[cfg(test)]
mod test
{
  use crate::gqls::prelude::*;

  #[test]
  fn parse_simple()
  {
    let ast = super::parse_schema(
      r#"
// Properties definitions
Person{
  firstName: STRING, lastName : STRING
},
Message {
  creationDate: TIMESTAMP, browserUsed: STRING
},
Comment <: Message {},
Post <: Message {
  imageFile: STRING?
},
REPLY_OF {},
KNOWS {
  creationDate : TIMESTAMP
},

// Nodes
(Person), (Post), (Comment),

// Edges
(Person)-[KNOWS]->(Person),
(Person)-[LIKES]->(Message),
(Message)-[HAS_CREATOR]->(Person),
(Comment)-[REPLY_OF]->(Message)
    "#,
    )
    .unwrap();

    // Check nodes
    assert_eq!(ast.nodes.len(), 3);
    let n0 = &ast.nodes[0];
    assert_eq!(n0.labels, vec!["Person".to_string()]);
    assert_eq!(n0.properties.len(), 2);
    assert_eq!(n0.properties["firstName"], LiteralBaseType::String.into());
    assert_eq!(n0.properties["lastName"], LiteralBaseType::String.into());
    let n1 = &ast.nodes[1];
    assert_eq!(n1.labels, vec!["Message".to_string(), "Post".to_string()]);
    assert_eq!(n1.properties.len(), 3);
    assert_eq!(
      n1.properties["creationDate"],
      LiteralBaseType::TimeStamp.into()
    );
    assert_eq!(n1.properties["browserUsed"], LiteralBaseType::String.into());
    assert_eq!(
      n1.properties["imageFile"],
      Property::Optional(Box::new(LiteralBaseType::String.into()))
    );
    let n2 = &ast.nodes[2];
    assert_eq!(
      n2.labels,
      vec!["Message".to_string(), "Comment".to_string()]
    );
    assert_eq!(n2.properties.len(), 2);
    assert_eq!(
      n2.properties["creationDate"],
      LiteralBaseType::TimeStamp.into()
    );
    assert_eq!(n2.properties["browserUsed"], LiteralBaseType::String.into());

    // Check edges
    assert_eq!(ast.edges.len(), 4);
    let e0 = &ast.edges[0];
    assert_eq!(e0.source, "Person");
    assert_eq!(e0.labels, vec!["KNOWS".to_string()]);
    assert_eq!(e0.destination, "Person");
    let e1 = &ast.edges[1];
    assert_eq!(e1.source, "Person");
    assert_eq!(e1.labels, vec!["LIKES".to_string()]);
    assert_eq!(e1.destination, "Message");
    let e2 = &ast.edges[2];
    assert_eq!(e2.source, "Message");
    assert_eq!(e2.labels, vec!["HAS_CREATOR".to_string()]);
    assert_eq!(e2.destination, "Person");
    let e3 = &ast.edges[3];
    assert_eq!(e3.source, "Comment");
    assert_eq!(e3.labels, vec!["REPLY_OF".to_string()]);
    assert_eq!(e3.destination, "Message");
  }
}
