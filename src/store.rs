#[cfg(feature = "persy")]
mod persy;
#[cfg(feature = "pgql")]
mod pgql;

#[cfg(feature = "persy")]
pub(crate) use persy::Store;

#[cfg(feature = "pgql")]
pub(crate) use pgql::Store;

use crate::graph;

//  ____  _        _   _     _   _
// / ___|| |_ __ _| |_(_)___| |_(_) ___ ___
// \___ \| __/ _` | __| / __| __| |/ __/ __|
//  ___) | || (_| | |_| \__ \ |_| | (__\__ \
// |____/ \__\__,_|\__|_|___/\__|_|\___|___/

pub(crate) struct Statistics
{
  pub nodes_count: usize,
  pub edges_count: usize,
  pub labels_nodes_count: usize,
  pub properties_count: usize,
}

//  ____       _           _   _   _           _       ___
// / ___|  ___| | ___  ___| |_| \ | | ___   __| | ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  \| |/ _ \ / _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |\  | (_) | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_| \_|\___/ \__,_|\___|\__\_\\__,_|\___|_|   \__, |
//                                                                          |___/

#[derive(Default)]
pub(crate) struct SelectNodeQuery<'a, TKeys, TLabels, TProperties>
where
  TKeys: Iterator<Item = &'a crate::graph::Key>,
  TLabels: Iterator<Item = &'a String>,
  TProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
{
  keys: Option<TKeys>,
  labels: Option<TLabels>,
  properties: Option<TProperties>,
}

impl<'a, TKeys, TLabels, TProperties> SelectNodeQuery<'a, TKeys, TLabels, TProperties>
where
  TKeys: Iterator<Item = &'a crate::graph::Key>,
  TLabels: Iterator<Item = &'a String>,
  TProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
{
  fn is_select_all(&self) -> bool
  {
    self.keys.is_none() && self.labels.is_none() && self.properties.is_none()
  }
}

pub(crate) struct NullIterator<'a, T: 'a>
{
  _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: 'a> Default for NullIterator<'a, T>
{
  fn default() -> Self
  {
    Self {
      _phantom: Default::default(),
    }
  }
}

impl<'a, T: 'a> Iterator for NullIterator<'a, T>
{
  type Item = T;
  fn next(&mut self) -> Option<Self::Item>
  {
    None
  }
}

impl
  SelectNodeQuery<
    'static,
    NullIterator<'static, &crate::graph::Key>,
    NullIterator<'static, &String>,
    NullIterator<'static, (&'static String, &'static graph::Value)>,
  >
{
  pub(crate) fn select_all() -> Self
  {
    Self::default()
  }
}

impl<'a, TKeys: Iterator<Item = &'a crate::graph::Key>>
  SelectNodeQuery<
    'a,
    TKeys,
    NullIterator<'a, &'a String>,
    NullIterator<'a, (&'a String, &'a graph::Value)>,
  >
{
  #[allow(unused)]
  pub(crate) fn select_keys(keys: TKeys) -> Self
  {
    Self {
      keys: Some(keys),
      labels: None,
      properties: None,
    }
  }
}

#[cfg(test)]
impl<'a, TLabels: Iterator<Item = &'a String>>
  SelectNodeQuery<
    'a,
    NullIterator<'a, &'a graph::Key>,
    TLabels,
    NullIterator<'a, (&'a String, &'a graph::Value)>,
  >
{
  pub(crate) fn select_labels(labels: TLabels) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels),
      properties: None,
    }
  }
}

impl<
    'a,
    TLabels: Iterator<Item = &'a String>,
    TProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
  > SelectNodeQuery<'a, NullIterator<'a, &'a graph::Key>, TLabels, TProperties>
{
  pub(crate) fn select_labels_properties(labels: TLabels, properties: TProperties) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels),
      properties: Some(properties),
    }
  }
}

//  ____       _           _   _____    _             ___
// / ___|  ___| | ___  ___| |_| ____|__| | __ _  ___ / _ \ _   _  ___ _ __ _   _
// \___ \ / _ \ |/ _ \/ __| __|  _| / _` |/ _` |/ _ \ | | | | | |/ _ \ '__| | | |
//  ___) |  __/ |  __/ (__| |_| |__| (_| | (_| |  __/ |_| | |_| |  __/ |  | |_| |
// |____/ \___|_|\___|\___|\__|_____\__,_|\__, |\___|\__\_\\__,_|\___|_|   \__, |
//                                        |___/                            |___/

#[derive(Default)]
pub(crate) struct SelectEdgeQuery<
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
> where
  TSourceKeys: Iterator<Item = &'a crate::graph::Key>,
  TSourceLabels: Iterator<Item = &'a String>,
  TSourceProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
  TKeys: Iterator<Item = &'a crate::graph::Key>,
  TLabels: Iterator<Item = &'a String>,
  TProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
  TDestinationKeys: Iterator<Item = &'a crate::graph::Key>,
  TDestinationLabels: Iterator<Item = &'a String>,
  TDestinationProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
{
  keys: Option<TKeys>,
  labels: Option<TLabels>,
  properties: Option<TProperties>,
  source: SelectNodeQuery<'a, TSourceKeys, TSourceLabels, TSourceProperties>,
  destination: SelectNodeQuery<'a, TDestinationKeys, TDestinationLabels, TDestinationProperties>,
}

impl
  SelectEdgeQuery<
    'static,
    NullIterator<'static, &crate::graph::Key>,
    NullIterator<'static, &String>,
    NullIterator<'static, (&'static String, &'static graph::Value)>,
    NullIterator<'static, &crate::graph::Key>,
    NullIterator<'static, &String>,
    NullIterator<'static, (&'static String, &'static graph::Value)>,
    NullIterator<'static, &crate::graph::Key>,
    NullIterator<'static, &String>,
    NullIterator<'static, (&'static String, &'static graph::Value)>,
  >
{
  pub(crate) fn select_all() -> Self
  {
    Self::default()
  }
}

impl<'a, T: Iterator<Item = &'a crate::graph::Key>>
  SelectEdgeQuery<
    'a,
    NullIterator<'a, &'a crate::graph::Key>,
    NullIterator<'a, &'a String>,
    NullIterator<'a, (&'a String, &'a graph::Value)>,
    T,
    NullIterator<'a, &'a String>,
    NullIterator<'a, (&'a String, &'a graph::Value)>,
    NullIterator<'a, &'a crate::graph::Key>,
    NullIterator<'a, &'a String>,
    NullIterator<'a, (&'a String, &'a graph::Value)>,
  >
{
  #[allow(unused)]
  pub(crate) fn select_keys(keys: T) -> Self
  {
    Self {
      keys: Some(keys),
      labels: None,
      properties: None,
      source: SelectNodeQuery::select_all(),
      destination: SelectNodeQuery::select_all(),
    }
  }
}

impl<
    'a,
    TSourceKeys,
    TSourceLabels,
    TSourceProperties,
    TLabels,
    TProperties,
    TDestinationKeys,
    TDestinationLabels,
    TDestinationProperties,
  >
  SelectEdgeQuery<
    'a,
    TSourceKeys,
    TSourceLabels,
    TSourceProperties,
    NullIterator<'a, &'a crate::graph::Key>,
    TLabels,
    TProperties,
    TDestinationKeys,
    TDestinationLabels,
    TDestinationProperties,
  >
where
  TSourceKeys: Iterator<Item = &'a crate::graph::Key>,
  TSourceLabels: Iterator<Item = &'a String>,
  TSourceProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
  TLabels: Iterator<Item = &'a String>,
  TProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
  TDestinationKeys: Iterator<Item = &'a crate::graph::Key>,
  TDestinationLabels: Iterator<Item = &'a String>,
  TDestinationProperties: Iterator<Item = (&'a String, &'a graph::Value)>,
{
  #[allow(unused)]
  pub(crate) fn select_source_destination_labels_properties(
    source_query: SelectNodeQuery<'a, TSourceKeys, TSourceLabels, TSourceProperties>,
    labels: TLabels,
    properties: TProperties,
    destination_query: SelectNodeQuery<
      'a,
      TDestinationKeys,
      TDestinationLabels,
      TDestinationProperties,
    >,
  ) -> Self
  {
    Self {
      keys: None,
      labels: Some(labels),
      properties: Some(properties),
      source: source_query,
      destination: destination_query,
    }
  }
}
