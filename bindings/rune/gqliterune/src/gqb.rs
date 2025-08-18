use rune::{support::Result, Any};

#[derive(Any)]
struct Key
{
  key: graphcore::Key,
}

#[derive(Any)]
#[rune(item = ::gqb)]
struct Builder
{
  builder: gqb::Builder,
}

impl Builder
{
  #[rune::function(path = Self::new)]
  pub fn new() -> Builder
  {
    Builder {
      builder: Default::default(),
    }
  }
  #[rune::function]
  fn create_nodes(&mut self, values: Vec<(Vec<String>, rune::Value)>) -> Result<Vec<Key>>
  {
    Ok(
      self
        .builder
        .create_nodes(
          values
            .into_iter()
            .map(|(l, p)| {
              let p: graphcore::ValueMap = crate::to_gc_value(p)?.try_into()?;
              Ok((l, p))
            })
            .collect::<Result<Vec<_>>>()?,
        )
        .into_iter()
        .map(|key| Key { key })
        .collect(),
    )
  }
  #[rune::function]
  fn create_edges(&mut self, values: Vec<(Key, Vec<String>, rune::Value, Key)>)
    -> Result<Vec<Key>>
  {
    Ok(
      self
        .builder
        .create_edges(
          values
            .into_iter()
            .map(|(s, l, p, d)| {
              let p: graphcore::ValueMap = crate::to_gc_value(p)?.try_into()?;
              Ok((s.key, l, p, d.key))
            })
            .collect::<Result<Vec<_>>>()?,
        )
        .into_iter()
        .map(|key| Key { key })
        .collect(),
    )
  }
  /// Generate an opencypher query
  #[rune::function]
  fn into_oc_query(self) -> Result<(String, rune::Value)>
  {
    let (q, b) = self.builder.into_oc_query()?;
    Ok((q, crate::to_ru_value(b.into())?))
  }
}

/// Create rune module for gqb
pub fn gqb_module() -> Result<rune::Module>
{
  let mut m = rune::Module::with_crate("gqb")?;
  m.ty::<Builder>()?;
  m.function_meta(Builder::new)?;
  m.function_meta(Builder::create_nodes)?;
  m.function_meta(Builder::create_edges)?;
  m.function_meta(Builder::into_oc_query)?;
  Ok(m)
}

#[cfg(test)]
mod tests
{
  use crate::tests;

  #[test]
  fn test_query_builder()
  {
    let tester = tests::Tester::new(|rune_context| {
      rune_context.install(super::gqb_module().unwrap()).unwrap()
    });
    let (n, b) = tester
      .eval::<(String, rune::Value)>(
        r#"
      let builder = gqb::Builder::new();
      let nodes_ids = builder.create_nodes([(["n"], #{ "a": 1 }), (["m"], #{})])?;
      builder.create_edges([(nodes_ids[0], ["n"], #{ "a": 2 }, nodes_ids[1])])?;
      builder.into_oc_query()
      "#,
      )
      .unwrap();
    assert_eq!(n, "CREATE (v0:n $b0), (v1:m $b1), (v0)-[v2:n $b2]->(v1)");
    assert_eq!(
      crate::to_gc_value(b).unwrap(),
      graphcore::value_map!("$b0" => graphcore::value_map!("a" => 1), "$b2"=> graphcore::value_map!("a" => 2), "$b1" => graphcore::ValueMap::default()).into()
    )
  }
}
