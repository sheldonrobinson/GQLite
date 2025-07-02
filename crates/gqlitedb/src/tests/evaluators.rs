use crate::{
  graph::array,
  interpreter::evaluators::eval_program,
  prelude::*,
  store::{Store, TransactionBoxable},
  tests::templates::programs,
};

fn check_stats(
  store: &crate::store::redb::Store,
  nodes_count: usize,
  edges_count: usize,
  labels_node_count: usize,
  properties_count: usize,
)
{
  let mut tx = store.begin_read().unwrap();
  let stats = store.compute_statistics(&mut tx).unwrap();

  assert_eq!(stats.nodes_count, nodes_count);
  assert_eq!(stats.edges_count, edges_count);
  assert_eq!(stats.labels_nodes_count, labels_node_count);
  assert_eq!(stats.properties_count, properties_count);
  tx.close().unwrap();
}

#[test]
fn test_evaluate_simple_create_node()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  eval_program(&store, &programs::simple_create(), Default::default()).unwrap();
  check_stats(&store, 1, 0, 0, 0);
}

#[test]
fn test_evaluate_create_named_node()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(&store, &programs::create_named_node(), Default::default()).unwrap();
  check_stats(&store, 1, 0, 0, 1);

  assert_eq!(value, array![array!["p"], array!["foo"]]);
}

#[test]
fn test_evaluate_create_named_node_double_return()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(
    &store,
    &programs::create_named_node_double_return(),
    Default::default(),
  )
  .unwrap();
  check_stats(&store, 1, 0, 0, 2);

  assert_eq!(value, array![array!["id", "p"], array![12, "foo"]]);
}

#[test]
fn test_evaluate_double_with_return()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(&store, &programs::double_with_return(), Default::default()).unwrap();
  check_stats(&store, 0, 0, 0, 0);

  assert_eq!(value, array![array!["a"], array![1]]);
}

#[test]
fn test_evaluate_unwind()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(&store, &programs::unwind(), Default::default()).unwrap();
  check_stats(&store, 0, 0, 0, 0);

  assert_eq!(value, array![array!["i"], array![0]]);
}

#[test]
fn test_evaluate_match_loop()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

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

  let value = eval_program(&store, &programs::match_loop(), Default::default()).unwrap();
  check_stats(&store, 1, 1, 0, 0);

  assert_eq!(value, array![array!["n"], array![node]]);
}

#[test]
fn test_evaluate_optional_match()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(&store, &programs::optional_match(), Default::default()).unwrap();
  check_stats(&store, 0, 0, 0, 0);

  assert_eq!(value, array![array!["a"], array![graph::Value::Invalid]]);
}

#[test]
fn test_evaluate_match_count()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  let function_manager = functions::Manager::new();
  let program = programs::match_count(&function_manager);

  // Count 0
  let value = eval_program(&store, &program, Default::default()).unwrap();
  check_stats(&store, 0, 0, 0, 0);

  assert_eq!(value, array![array!["count(*)"], array![0]]);

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
  check_stats(&store, 1, 0, 0, 0);

  let value = eval_program(&store, &program, Default::default()).unwrap();
  check_stats(&store, 1, 0, 0, 0);

  assert_eq!(value, array![array!["count(*)"], array![1]]);
}
