use std::borrow::BorrowMut;

#[cfg(feature = "redb")]
mod redb;

use crate::{
  graph,
  store::{self, WriteTransaction},
};

fn test_add_nodes<TStore>(store: TStore)
where
  TStore: store::Store,
{
  // Add a single empty node
  let nodes = [
    crate::graph::Node {
      labels: graph::labels!("hello", "world"),
      properties: graph::properties!("key" => 42i64),
      key: crate::graph::Key::default(),
    },
    crate::graph::Node {
      labels: graph::labels!("not"),
      properties: Default::default(),
      key: crate::graph::Key::default(),
    },
  ];

  let mut tx = store.begin().unwrap();
  store
    .create_nodes(tx.borrow_mut(), &"default".into(), nodes.iter())
    .unwrap();
  tx.commit().unwrap();

  let selected_nodes = store
    .select_nodes(
      store.begin().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectNodeQuery::select_keys([nodes[0].key]),
    )
    .unwrap();

  assert_eq!(selected_nodes.len(), 1);
  assert_eq!(nodes[0], selected_nodes[0]);
  // Add a single node with label
  let selected_nodes = store
    .select_nodes(
      store.begin().unwrap().borrow_mut(),
      &"default".into(),
      store::SelectNodeQuery::select_labels(["not".to_string()]),
    )
    .unwrap();

  assert_eq!(selected_nodes.len(), 1);
  assert_eq!(nodes[1], selected_nodes[0]);
}

fn test_add_edges<TStore>(store: TStore)
where
  TStore: store::Store,
{
  let source_node = crate::graph::Node {
    labels: graph::labels!("hello"),
    properties: graph::properties!("key" => 42i64),
    key: crate::graph::Key::default(),
  };
  let destination_node = crate::graph::Node {
    labels: graph::labels!("world"),
    properties: graph::properties!("key" => 12i64),
    key: crate::graph::Key::default(),
  };
  let edge = crate::graph::Edge {
    source: source_node.clone(),
    destination: destination_node.clone(),
    key: crate::graph::Key::default(),
    labels: vec!["!".into()],
    properties: graph::properties!("existence" => true),
  };

  let mut tx = store.begin().unwrap();
  store
    .create_edges(tx.borrow_mut(), &"default".into(), [edge.clone()].iter())
    .expect_err("expect missing node");
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
  tx.commit().unwrap();

  let selected_edges = store
    .select_edges(
      store.begin().unwrap().borrow_mut(),
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
      store.begin().unwrap().borrow_mut(),
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
      store.begin().unwrap().borrow_mut(),
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
}
