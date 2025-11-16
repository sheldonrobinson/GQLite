use crate::{graph::EdgeDirectivity, parser::ast::*};

fn return_statement(var_id: VariableIdentifier) -> Statement
{
  Return {
    all: false,
    expressions: vec![NamedExpression {
      identifier: var_id.clone(),
      expression: Variable { identifier: var_id }.into(),
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
  let var_ids = VariableIdentifiers::default();
  vec![
    Create {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some(var_ids.create_variable_from_name("n")),
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
        identifier: var_ids.create_variable_from_name("p"),
        expression: MemberAccess {
          left: Variable {
            identifier: var_ids.create_variable_from_name("n"),
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
  let var_ids = VariableIdentifiers::default();
  vec![
    Create {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some(var_ids.create_variable_from_name("n")),
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
          identifier: var_ids.create_variable_from_name("id"),
          expression: MemberAccess {
            left: Variable {
              identifier: var_ids.create_variable_from_name("n"),
            }
            .into(),
            path: vec!["id".into()],
          }
          .into(),
        },
        NamedExpression {
          identifier: var_ids.create_variable_from_name("p"),
          expression: MemberAccess {
            left: Variable {
              identifier: var_ids.create_variable_from_name("n"),
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
  let var_ids = VariableIdentifiers::default();
  vec![
    With {
      all: false,
      expressions: vec![
        NamedExpression {
          identifier: var_ids.create_variable_from_name("n"),
          expression: Value { value: 1.into() }.into(),
        },
        NamedExpression {
          identifier: var_ids.create_variable_from_name("m"),
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
          identifier: var_ids.create_variable_from_name("a"),
          expression: Variable {
            identifier: var_ids.create_variable_from_name("n"),
          }
          .into(),
        },
        NamedExpression {
          identifier: var_ids.create_variable_from_name("b"),
          expression: Variable {
            identifier: var_ids.create_variable_from_name("m"),
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
    return_statement(var_ids.create_variable_from_name("a")),
  ]
}

/// AST for `UNWIND [0] AS i RETURN i`
pub(crate) fn unwind() -> Statements
{
  let var_ids = VariableIdentifiers::default();
  vec![
    Unwind {
      identifier: var_ids.create_variable_from_name("i"),
      expression: Array {
        array: vec![Value { value: 0.into() }.into()],
      }
      .into(),
    }
    .into(),
    return_statement(var_ids.create_variable_from_name("i")),
  ]
}

/// AST for `MATCH (n)-[]->(n) RETURN n`
pub(crate) fn match_loop() -> Statements
{
  let var_ids = VariableIdentifiers::default();
  vec![
    Match {
      patterns: vec![Pattern::Edge(EdgePattern {
        variable: None,
        source: NodePattern {
          variable: Some(var_ids.create_variable_from_name("n")),
          labels: LabelExpression::None,
          properties: None,
        },
        destination: NodePattern {
          variable: Some(var_ids.create_variable_from_name("n")),
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
    return_statement(var_ids.create_variable_from_name("n")),
  ]
}

/// AST for `OPTIONAL MATCH (a) RETURN a`
pub(crate) fn optional_match() -> Statements
{
  let var_ids = VariableIdentifiers::default();
  vec![
    Match {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some(var_ids.create_variable_from_name("a")),
        labels: LabelExpression::None,
        properties: None,
      })],
      where_expression: None,
      optional: true,
    }
    .into(),
    return_statement(var_ids.create_variable_from_name("a")),
  ]
}

/// AST for `MATCH (a) RETURN COUNT(*)`
pub(crate) fn match_count() -> Statements
{
  let var_ids = VariableIdentifiers::default();
  vec![
    Match {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some(var_ids.create_variable_from_name("a")),
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
        identifier: var_ids.create_variable_from_name("count(*)"),
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

/// AST for `MATCH (n) RETURN n.name, count(n.num)`
pub(crate) fn aggregation() -> Statements
{
  let var_ids = VariableIdentifiers::default();
  vec![
    Match {
      patterns: vec![Pattern::Node(NodePattern {
        variable: Some(var_ids.create_variable_from_name("n")),
        labels: LabelExpression::None,
        properties: None,
      })],
      where_expression: None,
      optional: false,
    }
    .into(),
    Return {
      all: false,
      expressions: vec![
        NamedExpression {
          identifier: var_ids.create_variable_from_name("n.name"),
          expression: MemberAccess {
            left: Variable {
              identifier: var_ids.create_variable_from_name("n"),
            }
            .into(),
            path: vec!["name".into()],
          }
          .into(),
        },
        NamedExpression {
          identifier: var_ids.create_variable_from_name("count(n.num)"),
          expression: FunctionCall {
            name: "count".into(),
            arguments: vec![MemberAccess {
              left: Variable {
                identifier: var_ids.create_variable_from_name("n"),
              }
              .into(),
              path: vec!["num".into()],
            }
            .into()],
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
