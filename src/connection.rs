use std::borrow::Borrow;

use crate::interpreter;

pub struct Connection
{
  store: crate::store::Store,
}

impl Connection
{
  pub fn open<P: AsRef<std::path::Path>>(
    path: P,
    _options: crate::graph::ValueObject,
  ) -> crate::Result<Connection>
  {
    Ok(Connection {
      store: crate::store::Store::new(path)?,
    })
  }
  pub fn execute_query(
    &self,
    query: impl Into<String>,
    _bindings: crate::graph::ValueObject,
  ) -> crate::Result<crate::graph::Value>
  {
    let q: String = query.into();
    let q = crate::parser::parse(q.as_str())?;
    let q = interpreter::compiler::compile(q)?;
    return interpreter::evaluators::eval_program(self.store.borrow(), q);
  }
}
