use crate::{
  graph::array, interpreter::evaluators::eval_program, store::Store, tests::templates::programs,
};

fn check_stats(
  store: crate::store::redb::Store,
  nodes_count: usize,
  edges_count: usize,
  labels_node_count: usize,
  properties_count: usize,
)
{
  let mut tx = store.begin().unwrap();
  let stats = store.compute_statistics(&mut tx).unwrap();

  assert_eq!(stats.nodes_count, nodes_count);
  assert_eq!(stats.edges_count, edges_count);
  assert_eq!(stats.labels_nodes_count, labels_node_count);
  assert_eq!(stats.properties_count, properties_count);
}

#[test]
fn test_evaluate_simple_create_node()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  eval_program(&store, programs::simple_create(), Default::default()).unwrap();
  check_stats(store, 1, 0, 0, 0);
}

#[test]
fn test_evaluate_create_named_node()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

  let value = eval_program(&store, programs::create_named_node(), Default::default()).unwrap();
  check_stats(store, 1, 0, 0, 1);

  assert_eq!(value, array![array!["p"], array!["foo"]]);
}
