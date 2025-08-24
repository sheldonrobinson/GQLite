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
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  eval_program(&store, &programs::simple_create(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 0);
}

#[test]
fn test_evaluate_create_named_node()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::create_named_node(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 1);

  assert_eq!(value, graphcore::table![("p"), [["foo"]]].into());
}

#[test]
fn test_evaluate_create_named_node_double_return()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let value = eval_program(
    &store,
    &programs::create_named_node_double_return(),
    &Default::default(),
  )
  .unwrap();
  check_stats(&store, None, 1, 0, 0, 2);

  assert_eq!(value, graphcore::table![("id", "p"), [[12, "foo"]]].into());
}

#[test]
fn test_evaluate_double_with_return()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::double_with_return(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, crate::table![("a"), [[1,]]].into());
}

#[test]
fn test_evaluate_unwind()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::unwind(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, crate::table![("i"), [[0,]]].into());
}

#[test]
fn test_evaluate_match_loop()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let node = graph::Node::new(graph::Key::new(1), vec![], Default::default());
  let mut tx = store.begin_write().unwrap();
  store
    .create_nodes(&mut tx, &"default".to_string(), vec![&node].into_iter())
    .unwrap();
  store
    .create_edges(
      &mut tx,
      &"default".to_string(),
      vec![&graph::Path::new(
        graph::Key::new(2),
        node.clone(),
        vec![],
        Default::default(),
        node.clone(),
      )]
      .into_iter(),
    )
    .unwrap();
  tx.close().unwrap();

  let value = eval_program(&store, &programs::match_loop(), &Default::default()).unwrap();
  check_stats(&store, None, 1, 1, 0, 0);

  assert_eq!(value, crate::table![("n"), [[node]]].into());
}

#[test]
fn test_evaluate_optional_match()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();

  let value = eval_program(&store, &programs::optional_match(), &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, crate::table![("a"), [[value::Value::Null]]].into());
}

#[test]
fn test_evaluate_match_count()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();
  let function_manager = functions::Manager::new();
  let program = programs::match_count(&function_manager);

  // Count 0
  let value = eval_program(&store, &program, &Default::default()).unwrap();
  check_stats(&store, None, 0, 0, 0, 0);

  assert_eq!(value, crate::table![("count(*)"), [[0]]].into());

  // Count 1
  let node = graph::Node::new(graph::Key::new(1), vec![], Default::default());
  let mut tx = store.begin_write().unwrap();
  store
    .create_nodes(&mut tx, &"default".to_string(), vec![&node].into_iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 1, 0, 0, 0);

  let value = eval_program(&store, &program, &Default::default()).unwrap();
  check_stats(&store, None, 1, 0, 0, 0);

  assert_eq!(value, crate::table![("count(*)"), [[1]]].into());
}

#[test]
fn test_evaluate_aggregation()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::open(temp_file.path()).unwrap();
  let function_manager = functions::Manager::new();
  let program = programs::aggregation(&function_manager);

  let nodes = vec![
    graph::Node::new(
      graph::Key::new(1),
      vec![],
      value::value_map!("name" => "a", "num" => 33),
    ),
    graph::Node::new(graph::Key::new(2), vec![], value::value_map!("name" => "a")),
    graph::Node::new(
      graph::Key::new(3),
      vec![],
      value::value_map!("name" => "b", "num" => 42),
    ),
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
    value == crate::table![("n.name", "count(n.num)"), [["a", 1], ["b", 1]]].into()
      || value == crate::table![("n.name", "count(n.num)"), [["b", 1], ["a", 1]]].into(),
    "left ({}) == right ({:?} in any order) failed",
    value,
    crate::table![("n.name", "count(n.num)"), [["a", 1], ["b", 1]]],
  );
}
