#[test]
fn test_graphs()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::redb::Store::new(temp_file.path()).unwrap();
  super::test_graphs(store);
}

#[test]
fn test_select_nodes()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::sqlite::Store::new(temp_file.path()).unwrap();
  super::test_select_nodes(store);
}

#[test]
fn test_update_nodes()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::sqlite::Store::new(temp_file.path()).unwrap();
  super::test_update_nodes(store);
}

#[test]
fn test_select_edges()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::sqlite::Store::new(temp_file.path()).unwrap();
  super::test_select_edges(store);
}

#[test]
fn test_update_edges()
{
  let temp_file = crate::tests::create_tmp_file();
  let store = crate::store::sqlite::Store::new(temp_file.path()).unwrap();
  super::test_update_edges(store);
}
