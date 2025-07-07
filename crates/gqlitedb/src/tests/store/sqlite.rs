#[test]
fn test_graphs()
{
  let store = crate::store::redb::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_graphs(store);
}

#[test]
fn test_select_nodes()
{
  let store = crate::store::sqlite::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_select_nodes(store);
}

#[test]
fn test_update_nodes()
{
  let store = crate::store::sqlite::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_update_nodes(store);
}

#[test]
fn test_select_edges()
{
  let store = crate::store::sqlite::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_select_edges(store);
}

#[test]
fn test_update_edges()
{
  let store = crate::store::sqlite::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();
  super::test_update_edges(store);
}
