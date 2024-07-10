use std::borrow::Borrow;

use persy::PersyError;

use crate::graph;

pub(crate) struct Store
{
  persy_store: persy::Persy,
}

impl Store
{
  pub(crate) fn new<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Store>
  {
    let path = path.as_ref();
    let create = !path.exists();
    if create
    {
      persy::Persy::create(path)?;
    }
    let s = Store {
      persy_store: persy::Persy::open(path, persy::Config::new())?,
    };
    if create
    {
      s.create_graph("default")?;
    }
    Ok(s)
  }
  fn graph_main_index_name(graph_name: impl Into<String>) -> String
  {
    format!("{}_uuid", graph_name.into())
  }
  pub(crate) fn create_graph(&self, name: impl Into<String>) -> crate::Result<()>
  {
    let mut tx = self.persy_store.begin()?;
    let graph_name = name.into();
    tx.create_segment(graph_name.as_str())?;
    tx.create_index::<u128, persy::PersyId>(Self::graph_main_index_name(graph_name).as_str(), persy::ValueMode::Exclusive)?;
    tx.commit()?;
    Ok(())
  }
  pub(crate) fn begin(&self) -> crate::Result<persy::Transaction>
  {
    let s = self.persy_store.begin()?;
    Ok(s)
  }
  /// Add node to a graph
  pub(crate) fn add_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>
    (&self, transaction: &mut persy::Transaction, graph_name: impl Into<String>, nodes_iter: T) -> crate::Result<()>
  {
    let graph_name = graph_name.into();
    let graph_name_index = Self::graph_main_index_name(graph_name.clone());
    for x in nodes_iter
    {
      let mut data = Vec::<u8>::new();
      ciborium::into_writer(&x, &mut data)?;
      let pid = transaction.insert(graph_name.clone(), &data)?;
      transaction.put::<u128, persy::PersyId>(graph_name_index.as_str(), x.key.borrow().into(), pid)?;
    }
    Ok(())
  }
  pub(crate) fn select_node<'a, T: Iterator<Item = &'a crate::graph::Key>>
    (&self, transaction: &mut persy::Transaction, graph_name: impl Into<String>, keys_iter: T) -> crate::Result<Vec<crate::graph::Node>>
  {
    let graph_name = graph_name.into();
    let graph_name_index = Self::graph_main_index_name(graph_name.clone());
    let r = keys_iter.map(|key| {
      let key: u128 = key.into();
      if let Some(key) = transaction.one::<u128, persy::PersyId>(
        graph_name_index.as_str(), &key)?
      {
        if let Some(v) = transaction.read(graph_name.clone(), key.borrow())?
        {
          Ok::<graph::Node, crate::Error>(ciborium::from_reader::<graph::Node, &[u8]>(&mut v.as_ref())?)
        } else {
          Err(crate::Error::UnknownNode)
        }
      } else {
        Err(crate::Error::UnknownNode)
      }
    });
    let r = r.collect::<crate::Result<Vec<crate::graph::Node>>>()?;
    Ok(r)
  }
}

// Error

impl<T> From<persy::PE<T>> for crate::Error
  where T: Into<PersyError>
{
  fn from(value: persy::PE<T>) -> Self {
    match value {
      persy::PE::PE(err) => crate::Error::StoreError(err.into().to_string())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::graph::ToValue;
  #[test]
  fn test_add_nodes() {
    let nodes = [
      crate::graph::Node {
        labels: crate::labels!("hello", "world"),
        properties: crate::properties!("key" => 42i64),
        key: crate::graph::Key::default()
      }
    ];
    let store = super::Store::new(crate::tests::get_tmp_file().unwrap()).unwrap();

    store.add_nodes("default", nodes.iter()).unwrap();

    let selected_nodes = store.select_node("default", [nodes[0].key].iter()).unwrap();

    assert_eq!(nodes.len(), selected_nodes.len());
    assert_eq!(nodes[0], selected_nodes[0]);
  }
}
