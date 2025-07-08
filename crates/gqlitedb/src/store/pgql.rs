use crate::{graph, Result};

//  _____                               _   _
// |_   _| __ __ _ _ __  ___  __ _  ___| |_(_) ___  _ __
//   | || '__/ _` | '_ \/ __|/ _` |/ __| __| |/ _ \| '_ \
//   | || | | (_| | | | \__ \ (_| | (__| |_| | (_) | | | |
//   |_||_|  \__,_|_| |_|___/\__,_|\___|\__|_|\___/|_| |_|
//

pub(crate) struct Transaction {}

impl Transaction
{
  pub(crate) fn commit(&self) -> Result<()>
  {
    Ok(())
  }
}

//  ____  _
// / ___|| |_ ___  _ __ ___
// \___ \| __/ _ \| '__/ _ \
//  ___) | || (_) | | |  __/
// |____/ \__\___/|_|  \___|

pub(crate) struct Store {}

impl Store
{
  pub(crate) fn new() -> Result<Store>
  {
    Ok(Store {})
  }
  pub(crate) fn create_graph(&self, name: impl Into<String>) -> Result<()>
  {
    Err(InternalError::Unimplemented("pgql"))
  }
  pub(crate) fn begin(&self) -> Result<Transaction>
  {
    Ok(Transaction {})
  }
  pub(crate) fn add_nodes<'a, T: Iterator<Item = &'a crate::graph::Node>>(
    &self,
    _: &Transaction,
    graph_name: impl Into<String>,
    nodes_iter: T,
  ) -> Result<()>
  {
    Err(InternalError::Unimplemented("pgql"))
  }
  /// Select nodes according to a given query
  pub(crate) fn select_nodes<'a, TKeys, TLabels, TProperties>(
    &self,
    _: &Transaction,
    graph_name: impl Into<String>,
    query: super::SelectNodeQuery<'a, TKeys, TLabels, TProperties>,
  ) -> Result<Vec<crate::graph::Node>>
  where
    TKeys: Iterator<Item = &'a crate::graph::Key>,
    TLabels: Iterator<Item = &'a String>,
    TProperties: Iterator<Item = (&'a String, &'a value::Value)>,
  {
    Err(InternalError::Unimplemented("pgql"))
  }
  /// Add edge
  pub(crate) fn add_edges<'a, T: Iterator<Item = &'a crate::graph::Edge>>(
    &self,
    _: &Transaction,
    graph_name: impl Into<String>,
    edges_iter: T,
  ) -> Result<()>
  {
    Err(InternalError::Unimplemented("pgql"))
  }
  /// Select edges
  pub(crate) fn select_edges<
    'a,
    TSourceKeys,
    TSourceLabels,
    TSourceProperties,
    TKeys,
    TLabels,
    TProperties,
    TDestinationKeys,
    TDestinationLabels,
    TDestinationProperties,
  >(
    &self,
    _: &Transaction,
    graph_name: impl Into<String>,
    query: super::SelectEdgeQuery<
      'a,
      TSourceKeys,
      TSourceLabels,
      TSourceProperties,
      TKeys,
      TLabels,
      TProperties,
      TDestinationKeys,
      TDestinationLabels,
      TDestinationProperties,
    >,
  ) -> Result<Vec<crate::graph::Edge>>
  where
    TSourceKeys: Iterator<Item = &'a crate::graph::Key>,
    TSourceLabels: Iterator<Item = &'a String>,
    TSourceProperties: Iterator<Item = (&'a String, &'a value::Value)>,
    TKeys: Iterator<Item = &'a crate::graph::Key>,
    TLabels: Iterator<Item = &'a String>,
    TProperties: Iterator<Item = (&'a String, &'a value::Value)>,
    TDestinationKeys: Iterator<Item = &'a crate::graph::Key>,
    TDestinationLabels: Iterator<Item = &'a String>,
    TDestinationProperties: Iterator<Item = (&'a String, &'a value::Value)>,
  {
    Err(InternalError::Unimplemented("pgql"))
  }
  pub(crate) fn compute_statistics(&self, _: &Transaction) -> Result<super::Statistics>
  {
    Err(InternalError::Unimplemented("pgql"))
  }
}
