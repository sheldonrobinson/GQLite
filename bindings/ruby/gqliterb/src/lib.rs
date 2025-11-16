use std::fmt::Display;

use gqlitedb::TimeStamp;
use magnus::{
  function, method,
  prelude::*,
  r_array,
  r_hash::{self, ForEach},
  scan_args,
  value::Lazy,
  Error, ExceptionClass, Integer, IntoValue, RModule, Ruby, Symbol,
};

static MODULE: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("GQLite").unwrap());

static ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
  ruby
    .get_inner(&MODULE)
    .define_error("Error", ruby.exception_standard_error())
    .unwrap()
});

fn from_rvalue(ruby: &Ruby, value: magnus::Value) -> Result<gqlitedb::Value, Error>
{
  if value.is_nil()
  {
    Ok(gqlitedb::Value::Null)
  }
  else if value.is_kind_of(ruby.class_integer())
  {
    Ok(i64::try_convert(value)?.into())
  }
  else if value.is_kind_of(ruby.class_true_class())
  {
    Ok(true.into())
  }
  else if value.is_kind_of(ruby.class_false_class())
  {
    Ok(false.into())
  }
  else if value.is_kind_of(ruby.class_float())
  {
    Ok(f64::try_convert(value)?.into())
  }
  else if value.is_kind_of(ruby.class_string())
  {
    Ok(String::try_convert(value)?.into())
  }
  else if value.is_kind_of(ruby.class_time())
  {
    let time = magnus::Time::try_convert(value)?;
    let timespec = time.timespec()?;
    Ok(
      map_err(
        ruby,
        TimeStamp::from_unix_timestamp(
          timespec.tv_sec,
          timespec.tv_nsec as i32,
          time.utc_offset() as i32,
        ),
      )?
      .into(),
    )
  }
  else if value.is_kind_of(ruby.class_hash())
  {
    Ok(from_rhash(ruby, r_hash::RHash::try_convert(value)?)?.into())
  }
  else if value.is_kind_of(ruby.class_array())
  {
    Ok(from_rarray(ruby, r_array::RArray::try_convert(value)?)?.into())
  }
  else
  {
    Err(Error::new(
      ruby.get_inner(&ERROR),
      format!(
        "Cannot convert a value of type '{}' to a GQLite value.",
        value.class()
      ),
    ))
  }
}

fn from_rarray(ruby: &Ruby, array: r_array::RArray) -> Result<Vec<gqlitedb::Value>, Error>
{
  array
    .into_iter()
    .map(|value| from_rvalue(ruby, value))
    .collect()
}

fn from_rhash(ruby: &Ruby, hash: r_hash::RHash) -> Result<gqlitedb::ValueMap, Error>
{
  let mut vmap = gqlitedb::ValueMap::new();
  hash.foreach(|key: magnus::Value, value: magnus::Value| {
    let key = if key.is_kind_of(ruby.class_symbol())
    {
      Symbol::try_convert(key)?.name()?.into()
    }
    else
    {
      String::try_convert(key)?
    };

    vmap.insert(key, from_rvalue(ruby, value)?);
    Ok(ForEach::Continue)
  })?;

  Ok(vmap)
}

fn integer_from_u128(ruby: &Ruby, i: u128) -> Result<Integer, Error>
{
  if i <= u64::MAX as u128
  {
    Ok(ruby.integer_from_u64(i as u64))
  }
  else
  {
    ruby.module_kernel().funcall("Integer", (i.to_string(),))
  }
}

fn node_to_rhash(ruby: &Ruby, node: gqlitedb::Node) -> Result<magnus::Value, Error>
{
  let r_hash = ruby.hash_new();
  let (key, labels, properties) = node.unpack();
  r_hash.aset("type", "node")?;
  r_hash.aset("key", integer_from_u128(ruby, key.into())?)?;
  r_hash.aset("labels", labels)?;
  r_hash.aset("properties", to_rhash(ruby, properties)?)?;
  Ok(r_hash.into_value_with(ruby))
}

fn edge_to_rhash(ruby: &Ruby, edge: gqlitedb::Edge) -> Result<magnus::Value, Error>
{
  let r_hash = ruby.hash_new();
  let (key, labels, properties) = edge.unpack();
  r_hash.aset("type", "edge")?;
  r_hash.aset("key", integer_from_u128(ruby, key.into())?)?;
  r_hash.aset("labels", labels)?;
  r_hash.aset("properties", to_rhash(ruby, properties)?)?;
  Ok(r_hash.into_value_with(ruby))
}

fn path_to_rhash(ruby: &Ruby, path: gqlitedb::Path) -> Result<magnus::Value, Error>
{
  let r_hash = ruby.hash_new();
  let (key, source, labels, properties, destination) = path.unpack();
  r_hash.aset("type", "path")?;
  r_hash.aset("key", integer_from_u128(ruby, key.into())?)?;
  r_hash.aset("labels", labels)?;
  r_hash.aset("properties", to_rhash(ruby, properties)?)?;
  r_hash.aset("source", node_to_rhash(ruby, source)?)?;
  r_hash.aset("destination", node_to_rhash(ruby, destination)?)?;
  Ok(r_hash.into_value_with(ruby))
}

fn to_rvalue(ruby: &Ruby, val: gqlitedb::Value) -> Result<magnus::Value, Error>
{
  match val
  {
    gqlitedb::Value::Array(arr) => Ok(to_rarray(ruby, arr)?.into_value_with(ruby)),
    gqlitedb::Value::Boolean(b) => Ok(b.into_value_with(ruby)),
    gqlitedb::Value::Key(k) => Ok(integer_from_u128(ruby, k.into())?.into_value_with(ruby)),
    gqlitedb::Value::Integer(i) => Ok(i.into_value_with(ruby)),
    gqlitedb::Value::Float(f) => Ok(f.into_value_with(ruby)),
    gqlitedb::Value::String(s) => Ok(s.into_value_with(ruby)),
    gqlitedb::Value::TimeStamp(s) => Ok(to_rdatetime(ruby, s)?.into_value_with(ruby)),
    gqlitedb::Value::Map(m) => Ok(to_rhash(ruby, m)?.into_value_with(ruby)),
    gqlitedb::Value::Null => Ok(ruby.qnil().into_value_with(ruby)),
    gqlitedb::Value::Edge(e) => Ok(edge_to_rhash(ruby, e)?),
    gqlitedb::Value::Node(n) => Ok(node_to_rhash(ruby, n)?),
    gqlitedb::Value::Path(p) => Ok(path_to_rhash(ruby, p)?),
  }
}

fn to_rdatetime(ruby: &Ruby, ts: gqlitedb::TimeStamp) -> Result<magnus::Time, Error>
{
  let (tv_sec, tv_nsec) = ts.unix_timestamp();
  ruby.time_timespec_new(
    magnus::time::Timespec {
      tv_sec,
      tv_nsec: tv_nsec as i64,
    },
    map_err(ruby, magnus::time::Offset::from_secs(ts.offset_seconds()))?,
  )
}

fn to_rhash(ruby: &Ruby, map: gqlitedb::ValueMap) -> Result<r_hash::RHash, Error>
{
  let r_hash = ruby.hash_new();
  for (key, value) in map.into_iter()
  {
    r_hash.aset(key, to_rvalue(ruby, value)?)?;
  }
  Ok(r_hash)
}

fn to_rarray(ruby: &Ruby, arr: Vec<gqlitedb::Value>) -> Result<r_array::RArray, Error>
{
  let r_arr = ruby.ary_new_capa(arr.len());

  for value in arr.into_iter()
  {
    r_arr.push(to_rvalue(ruby, value)?)?;
  }

  Ok(r_arr)
}

fn map_err<T, E>(ruby: &Ruby, result: Result<T, E>) -> Result<T, Error>
where
  E: Display,
{
  result.map_err(|e| Error::new(ruby.get_inner(&ERROR), format!("{}", e)))
}

#[magnus::wrap(class = "GQLite::Connection")]
struct Connection
{
  dbhandle: std::sync::RwLock<Option<gqlitedb::Connection>>,
}

impl Connection
{
  fn new(ruby: &Ruby, args: &[magnus::Value]) -> Result<Self, Error>
  {
    let args = scan_args::scan_args::<(), (), (), (), _, ()>(args)?;

    let options = from_rhash(ruby, args.keywords)?;

    let filename: String = map_err(
      ruby,
      options
        .get("filename")
        .ok_or_else(|| Error::new(ruby.get_inner(&ERROR), "Missing filename."))?
        .to_owned()
        .try_into(),
    )?;
    let dbhandle = map_err(
      ruby,
      gqlitedb::Connection::builder()
        .options(options)
        .path(filename)
        .create(),
    )?;
    Ok(Self {
      dbhandle: std::sync::RwLock::new(Some(dbhandle)),
    })
  }
  fn execute_oc_query(
    ruby: &Ruby,
    rb_self: &Self,
    args: &[magnus::Value],
  ) -> Result<magnus::Value, Error>
  {
    match &*rb_self.dbhandle.read().unwrap()
    {
      Some(connection) =>
      {
        let args = scan_args::scan_args::<_, (), (), (), _, ()>(args)?;
        let (query,): (String,) = args.required;

        let kw = scan_args::get_kwargs::<_, (), (Option<magnus::Value>,), ()>(
          args.keywords,
          &[],
          &["bindings"],
        )?;
        let (bindings,) = kw.optional;

        let bindings = bindings
          .map(|bindings| {
            if bindings.is_nil()
            {
              Ok(Default::default())
            }
            else
            {
              from_rhash(ruby, r_hash::RHash::try_convert(bindings)?)
            }
          })
          .transpose()?
          .unwrap_or_default();
        let result = map_err(ruby, connection.execute_oc_query(query, bindings))?;

        to_rvalue(ruby, result.into_value())
      }
      None => Err(Error::new(
        ruby.get_inner(&ERROR),
        "Connection is closed.".to_string(),
      )),
    }
  }
  fn close(&self)
  {
    *self.dbhandle.write().unwrap() = None;
  }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error>
{
  Lazy::force(&ERROR, ruby);

  let module = ruby.get_inner(&MODULE);
  let class = module.define_class("Connection", ruby.class_object())?;
  class.define_singleton_method("new", function!(Connection::new, -1))?;
  class.define_method(
    "execute_oc_query",
    method!(Connection::execute_oc_query, -1),
  )?;
  class.define_method("close", method!(Connection::close, 0))?;
  Ok(())
}
