use crate::{
  interpreter::evaluators::eval_program,
  prelude::*,
  store::{Store, TransactionBoxable},
  tests::{check_stats, templates::programs},
};

#[test]
fn test_evaluate_simple_create_node()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  eval_program(&store, &programs::simple_create(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 0);
}

#[test]
fn test_evaluate_create_named_node()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::create_named_node(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 1);

  assert_eq!(
    value,
    value::array![value::array!["p"], value::array!["foo"]]
  );
}

#[test]
fn test_evaluate_create_named_node_double_return()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let value = eval_program(
    &store,
    &programs::create_named_node_double_return(),
    &Default::default(),
  )
  .unwrap();
  check_stats(&store, None, 1, 0, 0, 2);

  assert_eq!(
    value,
    value::array![value::array!["id", "p"], value::array![12, "foo"]]
  );
}

#[test]
fn test_evaluate_double_with_return()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::double_with_return(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, value::array![value::array!["a"], value::array![1]]);
}

#[test]
fn test_evaluate_unwind()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::unwind(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, value::array![value::array!["i"], value::array![0]]);
}

#[test]
fn test_evaluate_match_loop()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let node = graph::Node {
    key: graph::Key { uuid: 1 },
    labels: vec![],
    properties: Default::default(),
  };
  let mut tx = store.begin_write().unwrap();
  store
    .create_nodes(&mut tx, &"default".to_string(), vec![&node].into_iter())
    .unwrap();
  store
    .create_edges(
      &mut tx,
      &"default".to_string(),
      vec![&graph::Edge {
        key: graph::Key { uuid: 2 },
        source: node.clone(),
        destination: node.clone(),
        labels: vec![],
        properties: Default::default(),
      }]
      .into_iter(),
    )
    .unwrap();
  tx.close().unwrap();

  let value = eval_program(&store, &programs::match_loop(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 1, 0, 0);

  assert_eq!(
    value,
    value::array![value::array!["n"], value::array![node]]
  );
}

#[test]
fn test_evaluate_optional_match()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::optional_match(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(
    value,
    value::array![value::array!["a"], value::array![value::Value::Null]]
  );
}

#[test]
fn test_evaluate_match_count()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();
  let function_manager = functions::Manager::new();
  let program = programs::match_count(&function_manager);

  // Count 0
  let value = eval_program(&store, &program, &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(
    value,
    value::array![value::array!["count(*)"], value::array![0]]
  );

  // Count 1
  let node = graph::Node {
    key: graph::Key { uuid: 1 },
    labels: vec![],
    properties: Default::default(),
  };
  let mut tx = store.begin_write().unwrap();
  store
    .create_nodes(&mut tx, &"default".to_string(), vec![&node].into_iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 1, 0, 0, 0);

  let value = eval_program(&store, &program, &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 0);

  assert_eq!(
    value,
    value::array![value::array!["count(*)"], value::array![1]]
  );
}

#[test]
fn test_evaluate_aggregation()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();
  let function_manager = functions::Manager::new();
  let program = programs::aggregation(&function_manager);

  let nodes = vec![
    graph::Node {
      key: graph::Key { uuid: 1 },
      labels: vec![],
      properties: value::map!("name" => "a", "num" => 33),
    },
    graph::Node {
      key: graph::Key { uuid: 2 },
      labels: vec![],
      properties: value::map!("name" => "a"),
    },
    graph::Node {
      key: graph::Key { uuid: 3 },
      labels: vec![],
      properties: value::map!("name" => "b", "num" => 42),
    },
  ];
  let mut tx = store.begin_write().unwrap();
  store
    .create_nodes(&mut tx, &"default".to_string(), nodes.iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 3, 0, 0, 5);

  let value = eval_program(&store, &program, &Default::default()).unwrap();
  check_stats(&store, None, 3, 0, 0, 5);

  assert!(
    value
      == value::array![
        value::array!["n.name", "count(n.num)"],
        value::array!["a", 1],
        value::array!["b", 1]
      ]
      || value
        == value::array![
          value::array!["n.name", "count(n.num)"],
          value::array!["b", 1],
          value::array!["a", 1]
        ],
    "left ({}) == right ({} in any order) failed",
    value,
    value::array![
      value::array!["n.name", "count(n.num)"],
      value::array!["a", 1],
      value::array!["b", 1]
    ],
  );
}
