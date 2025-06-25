use crate::{
  interpreter::{instructions::*, Program},
  ValueObject,
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
          value: ValueObject::default().into(),
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
    },
  ]
}

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
    },
  ]
}

pub(crate) fn double_with_return() -> Program
{
  vec![
    Block::With {
      variables: vec![
        RWExpression {
          instructions: vec![Instruction::Push { value: 1.into() }],
          aggregations: vec![],
        },
        RWExpression {
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
    },
    Block::With {
      variables: vec![
        RWExpression {
          instructions: vec![Instruction::GetVariable { col_id: 0 }],
          aggregations: vec![],
        },
        RWExpression {
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
    },
    Block::Return {
      variables: vec![(
        "a".into(),
        RWExpression {
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
    },
  ]
}
