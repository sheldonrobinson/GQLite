use crate::parser::ast::*;

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
            map: std::collections::HashMap::<String, Expression>::from([(
              "name".into(),
              Value {
                value: "foo".into(),
              }
              .into(),
            )]),
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
            map: std::collections::HashMap::<String, Expression>::from([
              (
                "name".into(),
                Value {
                  value: "foo".into(),
                }
                .into(),
              ),
              ("id".into(), Value { value: 12.into() }.into()),
            ]),
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
    Return {
      all: false,
      expressions: vec![NamedExpression {
        name: "a".into(),
        expression: Variable {
          identifier: "a".into(),
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
