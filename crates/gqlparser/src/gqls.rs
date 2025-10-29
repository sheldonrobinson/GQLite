//! GQL Schema parser

pub mod ast;
mod parser;
pub mod prelude;
pub mod properties;

use indexmap::IndexMap;
use nom::Finish;

use crate::prelude::*;

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
  let mut properties_definitions: IndexMap<String, ast::PropertiesDefinition> = Default::default();

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
        let parents = parent.map_or_else(Vec::new, |x| vec![x]);
        properties_definitions.insert(
          label,
          ast::PropertiesDefinition {
            parents,
            properties,
          },
        );
      }
      parser::ParseTreeElement::Node { label } =>
      {
        nodes.push(ast::Node { label });
      }
      parser::ParseTreeElement::Edge {
        source,
        label,
        destination,
      } =>
      {
        edges.push(ast::Edge {
          source,
          label,
          destination,
        });
      }
    }
  }

  Ok(ast::Ast {
    elements: properties_definitions,
    nodes,
    edges,
  })
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
    assert_eq!(&ast.nodes[0].label, "Person");
    assert_eq!(&ast.nodes[1].label, "Post");
    assert_eq!(&ast.nodes[2].label, "Comment");

    assert_eq!(ast.edges.len(), 4);
    let e0 = &ast.edges[0];
    assert_eq!(&e0.source, "Person");
    assert_eq!(&e0.label, "KNOWS");
    assert_eq!(&e0.destination, "Person");
    let e1 = &ast.edges[1];
    assert_eq!(&e1.source, "Person");
    assert_eq!(&e1.label, "LIKES");
    assert_eq!(&e1.destination, "Message");
    let e2 = &ast.edges[2];
    assert_eq!(&e2.source, "Message");
    assert_eq!(&e2.label, "HAS_CREATOR");
    assert_eq!(&e2.destination, "Person");
    let e3 = &ast.edges[3];
    assert_eq!(&e3.source, "Comment");
    assert_eq!(&e3.label, "REPLY_OF");
    assert_eq!(&e3.destination, "Message");

    let (pd0_key, pd0_v) = &ast.elements.get_index(0).unwrap();
    assert_eq!(pd0_key.as_str(), "Person");
    assert!(pd0_v.parents.is_empty());
    assert_eq!(pd0_v.properties.len(), 2);
    assert_eq!(
      pd0_v.properties["firstName"],
      LiteralBaseType::String.into()
    );
    assert_eq!(pd0_v.properties["lastName"], LiteralBaseType::String.into());

    let (pd1_key, pd1_v) = &ast.elements.get_index(1).unwrap();
    assert_eq!(pd1_key.as_str(), "Message");
    assert!(pd1_v.parents.is_empty());
    assert_eq!(pd1_v.properties.len(), 2);
    assert_eq!(
      pd1_v.properties["creationDate"],
      LiteralBaseType::TimeStamp.into()
    );
    assert_eq!(
      pd1_v.properties["browserUsed"],
      LiteralBaseType::String.into()
    );

    let (pd2_key, pd2_v) = &ast.elements.get_index(2).unwrap();
    assert_eq!(pd2_key.as_str(), "Comment");
    assert_eq!(pd2_v.parents, vec!["Message".to_string()]);
    assert!(pd2_v.properties.is_empty());

    let (pd3_key, pd3_v) = &ast.elements.get_index(3).unwrap();
    assert_eq!(pd3_key.as_str(), "Post");
    assert_eq!(pd3_v.parents, vec!["Message".to_string()]);
    assert_eq!(pd3_v.properties.len(), 1);
    assert_eq!(
      pd3_v.properties["imageFile"],
      Property::Optional(Box::new(LiteralBaseType::String.into()))
    );

    let (pd4_key, pd4_v) = &ast.elements.get_index(4).unwrap();
    assert_eq!(pd4_key.as_str(), "REPLY_OF");
    assert!(pd4_v.parents.is_empty());
    assert!(pd4_v.properties.is_empty());

    let (pd5_key, pd5_v) = &ast.elements.get_index(5).unwrap();
    assert_eq!(pd5_key.as_str(), "KNOWS");
    assert!(pd5_v.parents.is_empty());
    assert_eq!(pd5_v.properties.len(), 1);
    assert_eq!(
      pd5_v.properties["creationDate"],
      LiteralBaseType::TimeStamp.into()
    );
  }
}
