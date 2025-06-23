use crate::parser::ast::*;

pub(crate) fn simple_create_node() -> Statements
{
  vec![Statement::Create(Create {
    patterns: vec![Pattern::Node(NodePattern {
      variable: None,
      labels: LabelExpression::None,
      properties: None,
    })],
  })]
}

/// AST for `CREATE (n {name: 'foo'}) RETURN n.name AS p`
pub(crate) fn create_named_node() -> Statements
{
  vec![
    Statement::Create(Create {
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
    }),
    Statement::Return(Return {
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
    }),
  ]
}

/// AST for `CREATE (n {id: 12, name: 'foo'}) RETURN n.id AS id, n.name AS p`
pub(crate) fn create_named_node_double_return() -> Statements
{
  vec![
    Statement::Create(Create {
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
    }),
    Statement::Return(Return {
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
    }),
  ]
}
