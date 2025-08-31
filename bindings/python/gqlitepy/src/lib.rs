#![deny(warnings)]

use pyo3::{
  prelude::*,
  types::{PyDict, PyList, PyNone},
  IntoPyObjectExt,
};

pyo3::create_exception!(gqlitepy, Error, pyo3::exceptions::PyException);

pub fn new_error(py: Python<'_>, msg: &str) -> PyErr
{
  let exc_type = py.get_type::<Error>();
  let obj = exc_type.call1((msg,)).unwrap();
  obj.setattr("msg", msg).unwrap();
  PyErr::from_value(obj)
}

fn map_err<T>(py: Python, result: gqlitedb::Result<T>) -> PyResult<T>
{
  result.map_err(|e| new_error(py, &format!("{}", e)))
}

fn from_pany<'py>(py: Python<'py>, value: &Bound<'py, PyAny>) -> PyResult<gqlitedb::Value>
{
  if value.is_none()
  {
    Ok(gqlitedb::Value::Null)
  }
  else if let Ok(b) = value.extract::<bool>()
  {
    Ok(gqlitedb::Value::Boolean(b))
  }
  else if let Ok(i) = value.extract::<i64>()
  {
    Ok(gqlitedb::Value::Integer(i))
  }
  else if let Ok(f) = value.extract::<f64>()
  {
    Ok(gqlitedb::Value::Float(f))
  }
  else if let Ok(s) = value.extract::<String>()
  {
    Ok(gqlitedb::Value::String(s))
  }
  else if let Ok(list) = value.downcast::<PyList>()
  {
    Ok(from_plist(py, list)?.into())
  }
  else if let Ok(dict) = value.downcast::<PyDict>()
  {
    Ok(from_pdict(py, dict)?.into())
  }
  else
  {
    // Placeholder for Node/Edge/Path
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
      "Unsupported Python type for Value conversion",
    ))
  }
}

fn from_plist<'py>(py: Python<'py>, list: &Bound<'py, PyList>) -> PyResult<Vec<gqlitedb::Value>>
{
  list
    .iter()
    .map(|value| from_pany(py, &value))
    .collect::<Result<Vec<_>, _>>()
}

fn from_pdict<'py>(py: Python<'py>, hash: &Bound<'py, PyDict>) -> PyResult<gqlitedb::ValueMap>
{
  hash
    .iter()
    .map(|(k, v)| Ok((k.to_string(), from_pany(py, &v)?)))
    .collect::<PyResult<_>>()
}

fn node_to_pdict<'py>(py: Python<'py>, node: gqlitedb::Node) -> PyResult<Bound<'py, PyAny>>
{
  let p_dict = PyDict::new(py);
  let (key, labels, properties) = node.unpack();
  p_dict.set_item("type", "node")?;
  let key: u128 = key.into();
  p_dict.set_item("key", key.into_bound_py_any(py)?)?;
  p_dict.set_item("labels", labels)?;
  p_dict.set_item("properties", to_pdict(py, properties)?)?;
  Ok(p_dict.into_any())
}

fn edge_to_pdict<'py>(py: Python<'py>, edge: gqlitedb::Edge) -> PyResult<Bound<'py, PyAny>>
{
  let p_dict = PyDict::new(py);
  let (key, labels, properties) = edge.unpack();
  p_dict.set_item("type", "edge")?;
  let key: u128 = key.into();
  p_dict.set_item("key", key.into_bound_py_any(py)?)?;
  p_dict.set_item("labels", labels)?;
  p_dict.set_item("properties", to_pdict(py, properties)?)?;
  Ok(p_dict.into_any())
}

fn path_to_pdict<'py>(py: Python<'py>, path: gqlitedb::Path) -> PyResult<Bound<'py, PyAny>>
{
  let p_dict = PyDict::new(py);
  let (key, source, labels, properties, destination) = path.unpack();
  p_dict.set_item("type", "path")?;
  let key: u128 = key.into();
  p_dict.set_item("key", key.into_bound_py_any(py)?)?;
  p_dict.set_item("labels", labels)?;
  p_dict.set_item("properties", to_pdict(py, properties)?)?;
  p_dict.set_item("source", node_to_pdict(py, source)?)?;
  p_dict.set_item("destination", node_to_pdict(py, destination)?)?;
  Ok(p_dict.into_any())
}

fn to_pvalue<'py>(py: Python<'py>, val: gqlitedb::Value) -> PyResult<Bound<'py, PyAny>>
{
  match val
  {
    gqlitedb::Value::Array(arr) => Ok(to_plist(py, arr)?.into_any()),
    gqlitedb::Value::Boolean(b) => b.into_bound_py_any(py),
    gqlitedb::Value::Key(k) => k.uuid().into_bound_py_any(py),
    gqlitedb::Value::Integer(i) => i.into_bound_py_any(py),
    gqlitedb::Value::Float(f) => f.into_bound_py_any(py),
    gqlitedb::Value::String(s) => s.into_bound_py_any(py),
    gqlitedb::Value::Map(m) => Ok(to_pdict(py, m)?.into_any()),
    gqlitedb::Value::Null => Ok(PyNone::get(py).to_owned().into_any()),
    gqlitedb::Value::Edge(e) => Ok(edge_to_pdict(py, e)?),
    gqlitedb::Value::Node(n) => Ok(node_to_pdict(py, n)?),
    gqlitedb::Value::Path(p) => Ok(path_to_pdict(py, p)?),
  }
}

fn to_pdict<'py>(py: Python<'py>, map: gqlitedb::ValueMap) -> PyResult<Bound<'py, PyDict>>
{
  let p_dict = PyDict::new(py);
  for (key, value) in map.into_iter()
  {
    p_dict.set_item(key, to_pvalue(py, value)?)?;
  }
  Ok(p_dict)
}

fn to_plist<'py>(py: Python<'py>, arr: Vec<gqlitedb::Value>) -> PyResult<Bound<'py, PyList>>
{
  PyList::new(
    py,
    arr
      .into_iter()
      .map(|x| to_pvalue(py, x))
      .collect::<PyResult<Vec<_>>>()?,
  )
}

#[pyclass(frozen)]
struct Connection
{
  dbhandle: gqlitedb::Connection,
}

#[pymethods]
impl Connection
{
  #[new]
  #[pyo3(signature = (filename, backend=None))]
  fn new(py: Python<'_>, filename: String, backend: Option<String>) -> PyResult<Self>
  {
    let mut options = gqlitedb::ValueMap::new();
    if let Some(backend) = backend
    {
      options.insert("backend".into(), backend.into());
    }
    let dbhandle = map_err(
      py,
      gqlitedb::Connection::builder()
        .options(options)
        .path(filename)
        .create(),
    )?;
    Ok(Self { dbhandle })
  }
  #[pyo3(signature = (query, bindings=None))]
  fn execute_oc_query<'py>(
    &self,
    py: Python<'py>,
    query: String,
    bindings: Option<&Bound<'_, PyDict>>,
  ) -> PyResult<Bound<'py, PyAny>>
  {
    let bindings = bindings
      .map(|bindings| from_pdict(py, bindings))
      .transpose()?
      .unwrap_or_default();
    let result = map_err(py, self.dbhandle.execute_oc_query(query, bindings))?;

    to_pvalue(py, result.into_value())
  }
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn gqlite(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()>
{
  m.add("Error", py.get_type::<Error>())?;
  m.add_class::<Connection>()?;
  Ok(())
}
