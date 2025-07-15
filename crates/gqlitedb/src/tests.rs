mod compiler;
mod evaluators;
mod parser;
mod store;
mod templates;

pub(crate) fn create_tmp_file() -> ccutils::temporary::TemporaryFile
{
  ccutils::temporary::TemporaryFile::builder()
    .should_create_file(false)
    .label("gqlite")
    .create()
}

fn check_stats<TStore: crate::store::Store>(
  store: &TStore,
  transaction: Option<&mut TStore::TransactionBox>,
  nodes_count: usize,
  edges_count: usize,
  labels_node_count: usize,
  properties_count: usize,
)
{
  let stats = match transaction
  {
    Some(mut tx) => store.compute_statistics(&mut tx).unwrap(),
    None =>
    {
      // use crate::store::TransactionBoxable;
      let mut tx = store.begin_read().unwrap();
      store.compute_statistics(&mut tx).unwrap()
    }
  };

  assert_eq!(stats.nodes_count, nodes_count);
  assert_eq!(stats.edges_count, edges_count);
  assert_eq!(stats.labels_nodes_count, labels_node_count);
  assert_eq!(stats.properties_count, properties_count);
}
