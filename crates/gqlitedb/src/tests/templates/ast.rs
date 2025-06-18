use crate::{graph::EdgeDirectivity, parser::ast::*};

fn return_statement(var_name: impl Into<String>) -> Statement
{
  let var_name = var_name.into();
  Return {
    all: false,
    expressions: vec![NamedExpression {
      name: var_name.clone(),
      expression: Variable {
        identifier: var_name.clone(),
      }
      .into(),
    }],
    modifiers: Modifiers {
      skip: None,
      limit: None,
      order_by: None,
    },
    where_expression: None,
  }
  .into()
}

pub(crate) fn simple_create_node() -> Statements
{
  vec![Create {
    patterns: vec![Pattern::Node(NodePattern {
      variable: None,
      labels: LabelExpression::None,
      properties: None,
    })],
  }
  .into()]
}

/// AST for `CREATE (n {name: 'foo'}) RETURN n.name AS p`
pub(crate) fn create_named_node() -> Statements
{
  vec![
    Create {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some("n".into()),
        labels: LabelExpression::None,
        properties: Some(
          Map {
            map: vec![(
              "name".into(),
              Value {
                value: "foo".into(),
              }
              .into(),
            )],
          }
          .into(),
        ),
      })],
    }
    .into(),
    Return {
      all: false,
      expressions: vec![NamedExpression {
        name: "p".into(),
        expression: MemberAccess {
          left: Variable {
            identifier: "n".into(),
          }
          .into(),
          path: vec!["name".into()],
        }
        .into(),
      }],
      modifiers: Modifiers {
        skip: None,
        limit: None,
        order_by: None,
      },
      where_expression: None,
    }
    .into(),
  ]
}

/// AST for `CREATE (n {id: 12, name: 'foo'}) RETURN n.id AS id, n.name AS p`
pub(crate) fn create_named_node_double_return() -> Statements
{
  vec![
    Create {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some("n".into()),
        labels: LabelExpression::None,
        properties: Some(
          Map {
            map: vec![
              ("id".into(), Value { value: 12.into() }.into()),
              (
                "name".into(),
                Value {
                  value: "foo".into(),
                }
                .into(),
              ),
            ],
          }
          .into(),
        ),
      })],
    }
    .into(),
    Return {
      all: false,
      expressions: vec![
        NamedExpression {
          name: "id".into(),
          expression: MemberAccess {
            left: Variable {
              identifier: "n".into(),
            }
            .into(),
            path: vec!["id".into()],
          }
          .into(),
        },
        NamedExpression {
          name: "p".into(),
          expression: MemberAccess {
            left: Variable {
              identifier: "n".into(),
            }
            .into(),
            path: vec!["name".into()],
          }
          .into(),
        },
      ],
      modifiers: Modifiers {
        skip: None,
        limit: None,
        order_by: None,
      },
      where_expression: None,
    }
    .into(),
  ]
}

/// AST for `WITH 1 AS n, 2 AS m WITH n AS a, m AS b RETURN a`
pub(crate) fn double_with_return() -> Statements
{
  vec![
    With {
      all: false,
      expressions: vec![
        NamedExpression {
          name: "n".into(),
          expression: Value { value: 1.into() }.into(),
        },
        NamedExpression {
          name: "m".into(),
          expression: Value { value: 2.into() }.into(),
        },
      ],
      modifiers: Modifiers {
        skip: None,
        limit: None,
        order_by: None,
      },
      where_expression: None,
    }
    .into(),
    With {
      all: false,
      expressions: vec![
        NamedExpression {
          name: "a".into(),
          expression: Variable {
            identifier: "n".into(),
          }
          .into(),
        },
        NamedExpression {
          name: "b".into(),
          expression: Variable {
            identifier: "m".into(),
          }
          .into(),
        },
      ],
      modifiers: Modifiers {
        skip: None,
        limit: None,
        order_by: None,
      },
      where_expression: None,
    }
    .into(),
    return_statement("a"),
  ]
}

/// AST for `UNWIND [0] AS i RETURN i`
pub(crate) fn unwind() -> Statements
{
  vec![
    Unwind {
      name: "i".into(),
      expression: Array {
        array: vec![Value { value: 0.into() }.into()],
      }
      .into(),
    }
    .into(),
    return_statement("i"),
  ]
}

/// AST for `MATCH (n)-[]->(n) RETURN n`
pub(crate) fn match_loop() -> Statements
{
  vec![
    Match {
      patterns: vec![Pattern::Edge(EdgePattern {
        variable: None,
        source: NodePattern {
          variable: Some("n".into()),
          labels: LabelExpression::None,
          properties: None,
        },
        destination: NodePattern {
          variable: Some("n".into()),
          labels: LabelExpression::None,
          properties: None,
        },
        labels: LabelExpression::None,
        properties: None,
        directivity: EdgeDirectivity::Directed,
      })],
      where_expression: None,
      optional: false,
    }
    .into(),
    return_statement("n"),
  ]
}

/// AST for `OPTIONAL MATCH (a) RETURN a`
pub(crate) fn optional_match() -> Statements
{
  vec![
    Match {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some("a".into()),
        labels: LabelExpression::None,
        properties: None,
      })],
      where_expression: None,
      optional: true,
    }
    .into(),
    return_statement("a"),
  ]
}

/// AST for `MATCH (a) RETURN COUNT(*)`
pub(crate) fn match_count() -> Statements
{
  vec![
    Match {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some("a".into()),
        labels: LabelExpression::None,
        properties: None,
      })],
      where_expression: None,
      optional: false,
    }
    .into(),
    Return {
      all: false,
      expressions: vec![NamedExpression {
        name: "count(*)".into(),
        expression: FunctionCall {
          name: "count".into(),
          arguments: vec![Value { value: 0.into() }.into()],
        }
        .into(),
      }],
      modifiers: Modifiers {
        skip: None,
        limit: None,
        order_by: None,
      },
      where_expression: None,
    }
    .into(),
  ]
}
