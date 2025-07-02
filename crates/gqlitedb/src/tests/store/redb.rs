#[test]
fn test_add_nodes()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_add_nodes(store);
}

#[test]
fn test_add_edges()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_add_edges(store);
}
