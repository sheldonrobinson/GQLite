use std::borrow::BorrowMut;

#[cfg(feature = "redb")]
mod redb;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

use crate::{
  prelude::*,
  store::{SelectNodeQuery, TransactionBoxable},
  tests::check_stats,
};

fn test_graphs<TStore>(store: TStore)
where
  TStore: store::Store,
{
  assert_eq!(
    store.graphs_list(&mut store.begin_read().unwrap()).unwrap(),
    vec!["default"]
  );

  // Create test graph
  let mut tx = store.begin_write().unwrap();
  store
    .create_graph(&mut tx, &"test_graph".to_string(), false)
    .unwrap();
  tx.close().unwrap();

  // Check the list of graphs
  let mut graphs = store.graphs_list(&mut store.begin_read().unwrap()).unwrap();
  graphs.sort();
  assert_eq!(
    graphs,
    vec!["default".to_string(), "test_graph".to_string()]
  );

  // Check creating the graph if it already exists
  let mut tx = store.begin_write().unwrap();
  store
    .create_graph(&mut tx, &"test_graph".to_string(), false)
    .expect_err("graph already exists");
  store
    .create_graph(&mut tx, &"test_graph".to_string(), true)
    .unwrap();
  tx.close().unwrap();

  // Check that there are still only two graphs
  let mut graphs = store.graphs_list(&mut store.begin_read().unwrap()).unwrap();
  graphs.sort();
  assert_eq!(
    graphs,
    vec!["default".to_string(), "test_graph".to_string()]
  );

  // Delete graph
  let mut tx = store.begin_write().unwrap();
  store
    .delete_graph(&mut tx, &"test_graph".to_string())
    .unwrap();
  tx.close().unwrap();

  // And gone from list
  assert_eq!(
    store.graphs_list(&mut store.begin_read().unwrap()).unwrap(),
    vec!["default".to_string()]
  );

  // Re-create test graph
  let mut tx = store.begin_write().unwrap();
  store
    .create_graph(&mut tx, &"test_graph".to_string(), false)
    .unwrap();
  tx.close().unwrap();

  // Check the list of graphs
  let mut graphs = store.graphs_list(&mut store.begin_read().unwrap()).unwrap();
  graphs.sort();
  assert_eq!(
    graphs,
    vec!["default".to_string(), "test_graph".to_string()]
  );

  let mut tx = store.begin_write().unwrap();
  store
    .delete_graph(&mut tx, &"unknown".to_string())
    .expect_err("Attempt at deleting unknown graph.");
  drop(tx);
}

fn test_select_nodes<TStore>(store: TStore)
where
  TStore: store::Store,
{
  // Add two nodes
  let nodes = [
    graph::Node {
      labels: graph::labels!("hello", "world"),
      properties: value::map!("key" => 42i64),
      key: graph::Key::default(),
    },
    graph::Node {
      labels: graph::labels!("not"),
      properties: Default::default(),
      key: graph::Key::default(),
    },
  ];

  check_stats(&store, None, 0, 0, 0, 0);

  let mut tx = store.begin_write().unwrap();

  store
    .create_nodes(tx.borrow_mut(), &"default".into(), nodes.iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 2, 0, 3, 1);

  let selected_nodes = store
    .select_nodes(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectNodeQuery::select_keys([nodes[0].key]),
    )
    .unwrap();

  assert_eq!(selected_nodes.len(), 1);
  assert_eq!(nodes[0], selected_nodes[0]);
  // Add a single node with label
  let selected_nodes = store
    .select_nodes(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectNodeQuery::select_labels(["not".to_string()]),
    )
    .unwrap();

  assert_eq!(selected_nodes.len(), 1);
  assert_eq!(nodes[1], selected_nodes[0]);
}

fn test_update_nodes<TStore>(store: TStore)
where
  TStore: store::Store,
{
  // Add a node
  let node = graph::Node {
    labels: graph::labels!("hello", "world"),
    properties: value::map!("key" => 42i64),
    key: graph::Key::default(),
  };

  check_stats(&store, None, 0, 0, 0, 0);

  let mut tx = store.begin_write().unwrap();

  store
    .create_nodes(
      tx.borrow_mut(),
      &"default".into(),
      vec![node.clone()].iter(),
    )
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 1, 0, 2, 1);

  // Modify node
  let modified_node = graph::Node {
    labels: graph::labels!("world"),
    properties: value::map!("key" => 12i64),
    key: node.key.clone(),
  };

  let mut tx = store.begin_write().unwrap();
  store
    .update_node(&mut tx, &"default".to_string(), &modified_node)
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 1, 0, 1, 1);

  let selected_nodes = store
    .select_nodes(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectNodeQuery::select_all(),
    )
    .unwrap();

  assert_eq!(selected_nodes.len(), 1);
  assert_eq!(modified_node, selected_nodes[0]);

  let mut tx = store.begin_write().unwrap();
  store
    .delete_nodes(
      &mut tx,
      &"default".into(),
      store::SelectNodeQuery::select_keys([modified_node.key]),
      true,
    )
    .unwrap();
  tx.close().unwrap();

  check_stats(&store, None, 0, 0, 0, 0);
}

fn test_select_edges<TStore>(store: TStore)
where
  TStore: store::Store,
{
  let source_node = graph::Node {
    labels: graph::labels!("hello"),
    properties: value::map!("key" => 42i64),
    key: graph::Key::default(),
  };
  let destination_node = graph::Node {
    labels: graph::labels!("world"),
    properties: value::map!("key" => 12i64),
    key: graph::Key::default(),
  };
  let edge = graph::Edge {
    source: source_node.clone(),
    destination: destination_node.clone(),
    key: graph::Key::default(),
    labels: vec!["!".into()],
    properties: value::map!("existence" => true),
  };

  check_stats(&store, None, 0, 0, 0, 0);

  let mut tx = store.begin_write().unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .expect_err("expect missing node");
  check_stats(&store, Some(&mut tx), 0, 0, 0, 0);
  store
    .create_nodes(
      tx.borrow_mut(),
      &"default".into(),
      [source_node, destination_node].iter(),
    )
    .unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 2, 1, 2, 3);

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_keys([edge.key]),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  assert_eq!(1, selected_edges.len());
  assert_eq!(edge, selected_edges[0].edge);
  assert!(!selected_edges[0].reversed);

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_source_destination_labels_properties(
        store::SelectNodeQuery::select_all(),
        vec![],
        Default::default(),
        store::SelectNodeQuery::select_all(),
      ),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  assert_eq!(1, selected_edges.len());
  assert_eq!(edge, selected_edges[0].edge);
  assert!(!selected_edges[0].reversed);

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_source_destination_labels_properties(
        store::SelectNodeQuery::select_labels_properties(vec![], Default::default()),
        vec![],
        Default::default(),
        store::SelectNodeQuery::select_labels_properties(vec![], Default::default()),
      ),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  assert_eq!(1, selected_edges.len());
  assert_eq!(edge, selected_edges[0].edge);
  assert!(!selected_edges[0].reversed);

  // Check reverse direction
  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_source_destination_labels_properties(
        store::SelectNodeQuery::select_keys(vec![edge.destination.key]),
        vec![],
        Default::default(),
        store::SelectNodeQuery::select_keys(vec![edge.source.key]),
      ),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  assert_eq!(0, selected_edges.len());

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_source_destination_labels_properties(
        store::SelectNodeQuery::select_keys(vec![edge.destination.key]),
        vec![],
        Default::default(),
        store::SelectNodeQuery::select_keys(vec![edge.source.key]),
      ),
      graph::EdgeDirectivity::Undirected,
    )
    .unwrap();

  assert_eq!(1, selected_edges.len());
  assert_eq!(edge, selected_edges[0].edge);
  assert!(selected_edges[0].reversed);
}

fn test_update_edges<TStore>(store: TStore)
where
  TStore: store::Store,
{
  let source_node = graph::Node {
    labels: graph::labels!("hello"),
    properties: value::map!("key" => 42i64),
    key: graph::Key::default(),
  };
  let destination_node = graph::Node {
    labels: graph::labels!("world"),
    properties: value::map!("key" => 12i64),
    key: graph::Key::default(),
  };
  let edge = graph::Edge {
    source: source_node.clone(),
    destination: destination_node.clone(),
    key: graph::Key::default(),
    labels: vec!["!".into()],
    properties: value::map!("existence" => true),
  };

  check_stats(&store, None, 0, 0, 0, 0);

  // Insert edge in store
  let mut tx = store.begin_write().unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .expect_err("expect missing node");
  check_stats(&store, Some(&mut tx), 0, 0, 0, 0);
  store
    .create_nodes(
      tx.borrow_mut(),
      &"default".into(),
      [source_node, destination_node].iter(),
    )
    .unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 2, 1, 2, 3);

  // Modify edge

  let modified_edge = graph::Edge {
    key: edge.key.clone(),
    source: edge.source.clone(),
    destination: edge.destination.clone(),
    labels: vec!["?".into()],
    properties: value::map!("existence" => false),
  };

  let mut tx = store.begin_write().unwrap();
  store
    .update_edge(&mut tx, &"default".into(), &modified_edge)
    .unwrap();
  tx.close().unwrap();

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_all(),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  check_stats(&store, None, 2, 1, 2, 3);

  assert_eq!(1, selected_edges.len());
  assert_eq!(modified_edge, selected_edges[0].edge);
  assert!(!selected_edges[0].reversed);

  // Remove edge

  let mut tx = store.begin_write().unwrap();
  store
    .delete_edges(
      &mut tx,
      &"default".into(),
      store::SelectEdgeQuery::select_all(),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 2, 0, 2, 2);

  let selected_edges = store
    .select_edges(
      store.begin_read().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectEdgeQuery::select_all(),
      graph::EdgeDirectivity::Directed,
    )
    .unwrap();

  assert_eq!(0, selected_edges.len());

  // Add edge back, and remove one of the node
  let mut tx = store.begin_write().unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 2, 1, 2, 3);

  let mut tx = store.begin_write().unwrap();
  store
    .delete_nodes(
      &mut tx,
      &"default".into(),
      SelectNodeQuery::select_all(),
      false,
    )
    .expect_err("should fails, nodes are still connected.");
  tx.close().unwrap();

  let mut tx = store.begin_write().unwrap();
  store
    .delete_nodes(
      &mut tx,
      &"default".into(),
      SelectNodeQuery::select_all(),
      true,
    )
    .unwrap();
  tx.close().unwrap();
  check_stats(&store, None, 0, 0, 0, 0);
}
