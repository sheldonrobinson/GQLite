use crate::interpreter;

pub struct Connection
{
  store: crate::store::Store,
  function_manager: crate::functions::Manager,
}

impl Connection
{
  #[cfg(feature = "redb")]
  pub fn open<P: AsRef<std::path::Path>>(
    path: P,
    _options: crate::graph::ValueObject,
  ) -> crate::Result<Connection>
  {
    Ok(Connection {
      store: crate::store::Store::new(path)?,
      function_manager: crate::functions::Manager::new(),
    })
  }
  #[cfg(feature = "pgql")]
  pub fn create() -> crate::Result<Connection>
  {
    Ok(Connection {
      store: crate::store::Store::new()?,
    })
  }
  pub fn execute_query(
    &self,
    query: impl Into<String>,
    parameters: crate::graph::ValueObject,
  ) -> crate::Result<crate::graph::Value>
  {
    let q: String = query.into();
    let q = crate::parser::parse(q.as_str())?;
    let q = interpreter::compiler::compile(&self.function_manager, q)?;
    return interpreter::evaluators::eval_program(&self.store, q, parameters);
  }
}
