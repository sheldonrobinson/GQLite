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
