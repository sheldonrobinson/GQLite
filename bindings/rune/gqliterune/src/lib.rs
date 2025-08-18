//! ![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite Rune
//! ===========================================================================
//!
//! Implementation of GQL (Graph Query Language), embeddable in applications.
//! This provides binding for the [Rune](https://rune-rs.github.io/) scripting language
//!
//! Add to your crate, using `cargo add gqliterune`.

#![warn(missing_docs)]
#![deny(warnings)]

use std::collections::HashMap;

#[cfg(feature = "gqb")]
mod gqb;

#[cfg(feature = "gqb")]
pub use gqb::gqb_module;

use rune::{
  support::{Error, Result},
  Any, FromValue, TypeHash,
};

#[derive(Any)]
#[rune(item = ::gqlite)]
struct Connection
{
  connection: gqlitedb::Connection,
}

fn to_gc_value(value: rune::Value) -> Result<gqlitedb::Value>
{
  match value.type_hash()
  {
    <()>::HASH => Ok(gqlitedb::Value::Null),
    bool::HASH => Ok(value.as_bool()?.into()),
    String::HASH => Ok(value.into_string()?.to_string().into()),
    Vec::<rune::Value>::HASH => Ok(
      Vec::<rune::Value>::from_value(value)?
        .into_iter()
        .map(|v| to_gc_value(v))
        .collect::<Result<Vec<_>>>()?
        .into(),
    ),
    HashMap::<String, rune::Value>::HASH => Ok(
      HashMap::<String, rune::Value>::from_value(value)?
        .into_iter()
        .map(|(k, v)| Ok((k, to_gc_value(v)?)))
        .collect::<Result<gqlitedb::ValueMap>>()?
        .into(),
    ),
    _ =>
    {
      if let Ok(i) = value.as_signed()
      {
        Ok(i.into())
      }
      else if let Ok(f) = value.as_float()
      {
        Ok(f.into())
      }
      else
      {
        Err(Error::msg(format!(
          "Cannot convert {:?} to GQLite value.",
          value
        )))
      }
    }
  }
}

fn to_ru_value(value: gqlitedb::Value) -> Result<rune::Value, rune::runtime::RuntimeError>
{
  match value
  {
    gqlitedb::Value::Null => Ok(rune::Value::empty()),
    gqlitedb::Value::Boolean(b) => rune::to_value(b),
    gqlitedb::Value::Integer(i) => rune::to_value(i),
    gqlitedb::Value::Float(f) => rune::to_value(f),
    gqlitedb::Value::String(s) => rune::to_value(s),
    gqlitedb::Value::Array(a) => rune::to_value(
      a.into_iter()
        .map(|x| to_ru_value(x))
        .collect::<Result<Vec<_>, _>>()?,
    ),
    gqlitedb::Value::Map(m) => rune::to_value(
      m.into_iter()
        .map(|(k, v)| Ok((k, to_ru_value(v)?)))
        .collect::<Result<HashMap<_, _>, rune::runtime::RuntimeError>>()?,
    ),
    gqlitedb::Value::Node(n) => to_ru_value(n.into_value_map().into()),
    gqlitedb::Value::Edge(e) => to_ru_value(e.into_value_map().into()),
    gqlitedb::Value::Path(p) => to_ru_value(p.into_value_map().into()),
  }
}

impl Connection
{
  fn to_value_map(value: rune::Value) -> Result<gqlitedb::ValueMap>
  {
    Ok(match to_gc_value(value)?
    {
      gqlitedb::Value::Null => Default::default(),
      gqlitedb::Value::Map(m) => m,
      o => Err(Error::msg(format!("Invalid argument {:?}", o)))?,
    })
  }
  #[rune::function(path = Self::create)]
  pub fn create(options: rune::Value) -> Result<Connection>
  {
    let options = Self::to_value_map(options)?;
    Ok(Connection {
      connection: gqlitedb::Connection::create(options)?,
    })
  }
  #[rune::function]
  pub fn execute_oc_query(&self, query: String, bindings: rune::Value) -> Result<rune::Value>
  {
    let bindings = Self::to_value_map(bindings)?;
    let r = self.connection.execute_query(query, bindings)?;
    Ok(to_ru_value(r)?)
  }
}

/// Convert a gqlitedb connection to a rune Value.
pub fn connection_to_rune_value(connection: gqlitedb::Connection) -> Result<rune::Value>
{
  use rune::ToValue as _;
  Ok(Connection {
    connection
  }.to_value()?)
}

/// Create the rune module
pub fn gqlite_module() -> Result<rune::Module>
{
  let mut m = rune::Module::with_crate("gqlite")?;
  m.ty::<Connection>()?;
  m.function_meta(Connection::create)?;
  m.function_meta(Connection::execute_oc_query)?;

  Ok(m)
}

#[cfg(test)]
mod tests
{
  use rune::{
    alloc::clone::TryClone,
    support::Result,
    termcolor::{ColorChoice, StandardStream},
    Context, Diagnostics, FromValue, Hash, Options, Source, Sources, Vm,
  };
  use std::sync::Arc;

  use crate::to_gc_value;

  pub(crate) struct Tester
  {
    rune_context: Context,
    sources: Vec<Source>,
  }
  impl Tester
  {
    pub(crate) fn new(installer: impl FnOnce(&mut rune::Context)) -> Tester
    {
      let mut rune_context = Context::with_default_modules().unwrap();
      installer(&mut rune_context);
      let sources = Default::default();
      Tester {
        rune_context,
        sources,
      }
    }
    fn build(
      &self,
      script: Option<Source>,
    ) -> Result<(Arc<rune::runtime::RuntimeContext>, Arc<rune::runtime::Unit>)>
    {
      let mut options = Options::default();
      let mut sources = Sources::default();

      for source in self.sources.iter()
      {
        sources.insert(source.try_clone()?)?;
      }

      if let Some(script) = script
      {
        sources.insert(script)?;
        options.script(true);
      }

      let mut diagnostics = Diagnostics::new();

      let result = rune::prepare(&mut sources)
        .with_context(&self.rune_context)
        .with_diagnostics(&mut diagnostics)
        .with_options(&options)
        .build();

      if !diagnostics.is_empty()
      {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
      }

      let unit = result?;
      Ok((Arc::new(self.rune_context.runtime()?), Arc::new(unit)))
    }

    pub(crate) fn eval<T>(&self, source: impl AsRef<str>) -> Result<T>
    where
      T: FromValue,
    {
      let (runtime, unit) = self.build(Some(Source::memory(source)?))?;
      let mut vm = Vm::new(runtime, unit);
      let r = vm.execute(Hash::EMPTY, ())?.complete().into_result()?;
      Result::<T>::from_value(r)?
    }
  }

  #[test]
  fn test_connection()
  {
    let tester = Tester::new(|rune_context| {
      rune_context
        .install(crate::gqlite_module().unwrap())
        .unwrap()
    });
    let n: Vec<gqlitedb::Value> = tester
      .eval::<rune::Value>(
        r#"
      let connection = gqlite::Connection::create({})?;
      connection.execute_oc_query("CREATE (n)", {})?;
      connection.execute_oc_query("MATCH (n) RETURN n", {})
      "#,
      )
      .map(|x| to_gc_value(x).unwrap())
      .unwrap()
      .try_into()
      .unwrap();
    assert_eq!(n.len(), 2);
    assert_eq!(n[0], graphcore::array!("n"));
    let row_0: Vec<gqlitedb::Value> = n[1].clone().try_into().unwrap();
    let n_0 = row_0[0].clone().into_map();
    assert_eq!(*n_0.get("labels").unwrap(), graphcore::array!());
    assert_eq!(
      *n_0.get("properties").unwrap(),
      gqlitedb::Value::Map(gqlitedb::map!())
    );
  }
}
