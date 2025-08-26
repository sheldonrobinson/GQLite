use crate::{
  error, functions,
  graph::EdgeDirectivity,
  interpreter::{instructions::*, Program},
  ValueMap,
};

fn create_variable_size(persistent_variables: usize, temporary_variables: usize) -> VariablesSizes
{
  VariablesSizes {
    temporary_variables,
    persistent_variables,
  }
}

pub(crate) fn simple_create() -> Program
{
  vec![Block::Create {
    actions: vec![CreateAction {
      instructions: vec![
        Instruction::Push {
          value: ValueMap::default().into(),
        },
        Instruction::CreateNodeLiteral {
          labels: Default::default(),
        },
      ],
      variables: vec![None],
    }],
    variables_size: create_variable_size(0, 0),
  }]
}

pub(crate) fn create_named_node() -> Program
{
  vec![
    Block::Create {
      actions: vec![CreateAction {
        instructions: vec![
          Instruction::Push {
            value: "foo".into(),
          },
          Instruction::CreateMap {
            keys: vec!["name".into()],
          },
          Instruction::CreateNodeLiteral {
            labels: Default::default(),
          },
        ],
        variables: vec![Some(0)],
      }],
      variables_size: create_variable_size(1, 0),
    },
    Block::Return {
      variables: vec![(
        "p".into(),
        RWExpression {
          col_id: 1,
          instructions: vec![
            Instruction::GetVariable { col_id: 0 },
            Instruction::MemberAccess {
              path: vec!["name".into()],
            },
          ],
          aggregations: Default::default(),
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(2, 0),
    },
  ]
}

/// Program for `CREATE (n {id: 12, name: 'foo'}) RETURN n.id AS id, n.name AS p`
pub(crate) fn create_named_node_double_return() -> Program
{
  vec![
    Block::Create {
      actions: vec![CreateAction {
        instructions: vec![
          Instruction::Push { value: 12.into() },
          Instruction::Push {
            value: "foo".into(),
          },
          Instruction::CreateMap {
            keys: vec!["id".into(), "name".into()],
          },
          Instruction::CreateNodeLiteral {
            labels: Default::default(),
          },
        ],
        variables: vec![Some(0)],
      }],
      variables_size: create_variable_size(1, 0),
    },
    Block::Return {
      variables: vec![
        (
          "id".into(),
          RWExpression {
            col_id: 1,
            instructions: vec![
              Instruction::GetVariable { col_id: 0 },
              Instruction::MemberAccess {
                path: vec!["id".into()],
              },
            ],
            aggregations: Default::default(),
          },
        ),
        (
          "p".into(),
          RWExpression {
            col_id: 2,
            instructions: vec![
              Instruction::GetVariable { col_id: 0 },
              Instruction::MemberAccess {
                path: vec!["name".into()],
              },
            ],
            aggregations: Default::default(),
          },
        ),
      ],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(3, 0),
    },
  ]
}

/// Program for `WITH 1 AS n, 2 AS m WITH n AS a, m AS b RETURN a`
pub(crate) fn double_with_return() -> Program
{
  vec![
    Block::With {
      variables: vec![
        RWExpression {
          col_id: 0,
          instructions: vec![Instruction::Push { value: 1.into() }],
          aggregations: vec![],
        },
        RWExpression {
          col_id: 1,
          instructions: vec![Instruction::Push { value: 2.into() }],
          aggregations: vec![],
        },
      ],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(2, 0),
    },
    Block::With {
      variables: vec![
        RWExpression {
          col_id: 2,
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
        RWExpression {
          col_id: 3,
          instructions: vec![Instruction::GetVariable { col_id: 1 }],
          aggregations: vec![],
        },
      ],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(4, 0),
    },
    Block::Return {
      variables: vec![(
        "a".into(),
        RWExpression {
          col_id: 0,
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(2, 0),
    },
  ]
}

/// Program for `UNWIND [0] AS i RETURN i`
pub(crate) fn unwind() -> Program
{
  vec![
    Block::Unwind {
      col_id: 0,
      instructions: vec![
        Instruction::Push { value: 0.into() },
        Instruction::CreateArray { length: 1 },
      ],
      variables_size: VariablesSizes {
        persistent_variables: 1,
        temporary_variables: 0,
      },
    },
    Block::Return {
      variables: vec![(
        "i".into(),
        RWExpression {
          col_id: 0,
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(1, 0),
    },
  ]
}

/// Program for `MATCH (n)-[]->(n) RETURN n`
pub(crate) fn match_loop() -> Program
{
  vec![
    Block::Match {
      blocks: vec![BlockMatch::MatchEdge {
        instructions: vec![
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateNodeQuery { labels: vec![] },
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateNodeQuery { labels: vec![] },
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateEdgeQuery { labels: vec![] },
        ],
        left_variable: Some(0),
        edge_variable: None,
        right_variable: Some(0),
        path_variable: None,
        filter: vec![],
        directivity: EdgeDirectivity::Directed,
      }],
      filter: vec![],
      optional: false,
      variables_size: VariablesSizes {
        persistent_variables: 1,
        temporary_variables: 0,
      },
    },
    Block::Return {
      variables: vec![(
        "n".into(),
        RWExpression {
          col_id: 0,
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(1, 0),
    },
  ]
}

/// Program for `OPTIONAL MATCH (a) RETURN a`
pub(crate) fn optional_match() -> Program
{
  vec![
    Block::Match {
      blocks: vec![BlockMatch::MatchNode {
        instructions: vec![
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateNodeQuery { labels: vec![] },
        ],
        variable: Some(0),
        filter: vec![],
      }],
      filter: vec![],
      optional: true,
      variables_size: create_variable_size(1, 0),
    },
    Block::Return {
      variables: vec![(
        "a".into(),
        RWExpression {
          col_id: 0,
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(1, 0),
    },
  ]
}

/// Program for `MATCH (a) RETURN COUNT(*)`
pub(crate) fn match_count(function_manager: &functions::Manager) -> Program
{
  vec![
    Block::Match {
      blocks: vec![BlockMatch::MatchNode {
        instructions: vec![
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateNodeQuery { labels: vec![] },
        ],
        variable: Some(0),
        filter: vec![],
      }],
      filter: vec![],
      optional: false,
      variables_size: VariablesSizes {
        persistent_variables: 1,
        temporary_variables: 0,
      },
    },
    Block::Return {
      variables: vec![(
        "count(*)".into(),
        RWExpression {
          col_id: 1,
          instructions: vec![Instruction::GetVariable { col_id: 1 }],
          aggregations: vec![(
            1,
            RWAggregation {
              init_instructions: vec![],
              argument_instructions: vec![Instruction::Push { value: 0.into() }],
              aggregator: function_manager
                .get_aggregator::<error::CompileTimeError>("count")
                .unwrap(),
            },
          )],
        },
      )],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(2, 1),
    },
  ]
}

/// Program for `MATCH (n) RETURN n.name, count(n.num)`
pub(crate) fn aggregation(function_manager: &functions::Manager) -> Program
{
  vec![
    Block::Match {
      blocks: vec![BlockMatch::MatchNode {
        instructions: vec![
          Instruction::Push {
            value: ValueMap::default().into(),
          },
          Instruction::CreateNodeQuery { labels: vec![] },
        ],
        variable: Some(0),
        filter: vec![],
      }],
      filter: vec![],
      optional: false,
      variables_size: VariablesSizes {
        persistent_variables: 1,
        temporary_variables: 0,
      },
    },
    Block::Return {
      variables: vec![
        (
          "n.name".into(),
          RWExpression {
            col_id: 1,
            instructions: vec![
              Instruction::GetVariable { col_id: 0 },
              Instruction::MemberAccess {
                path: vec!["name".into()],
              },
            ],
            aggregations: Default::default(),
          },
        ),
        (
          "count(n.num)".into(),
          RWExpression {
            col_id: 2,
            instructions: vec![Instruction::GetVariable { col_id: 2 }],
            aggregations: vec![(
              2,
              RWAggregation {
                init_instructions: vec![],
                argument_instructions: vec![
                  Instruction::GetVariable { col_id: 0 },
                  Instruction::MemberAccess {
                    path: vec!["num".into()],
                  },
                ],
                aggregator: function_manager
                  .get_aggregator::<error::CompileTimeError>("count")
                  .unwrap(),
              },
            )],
          },
        ),
      ],
      filter: vec![],
      modifiers: Modifiers {
        limit: None,
        skip: None,
        order_by: vec![],
      },
      variables_size: create_variable_size(3, 1),
    },
  ]
}
