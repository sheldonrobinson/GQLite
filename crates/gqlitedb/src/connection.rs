use crate::prelude::*;
use graph::ValueTryIntoRef;

trait ConnectionTrait
{
  fn execute_query(
    &self,
    query: String,
    parameters: crate::graph::ValueObject,
  ) -> crate::Result<crate::graph::Value>;
}

struct ConnectionImpl<TStore: store::Store>
{
  store: TStore,
  function_manager: crate::functions::Manager,
}

impl<TStore: store::Store> ConnectionTrait for ConnectionImpl<TStore>
{
  fn execute_query(
    &self,
    query: String,
    parameters: crate::graph::ValueObject,
  ) -> crate::Result<crate::graph::Value>
  {
    let q: String = query.into();
    let q = crate::parser::parse(q.as_str())?;
    let q = compiler::compile(&self.function_manager, q)?;
    return interpreter::evaluators::eval_program(&self.store, &q, parameters);
  }
}

impl<TStore: store::Store> ConnectionImpl<TStore>
{
  fn boxed(self) -> Box<Self>
  {
    Box::new(self)
  }
}

pub struct Connection
{
  connection: Box<dyn ConnectionTrait>,
}

impl Connection
{
  #[cfg(feature = "redb")]
  pub fn open<P: AsRef<std::path::Path>>(
    path: P,
    options: crate::graph::ValueObject,
  ) -> crate::Result<Connection>
  {
    if let Some(backend) = options.get("backend")
    {
      let backend: &String = backend.try_into_ref()?;
      match backend.as_str()
      {
        "redb" => Self::open_redb(path),
        _ => Err(
          StoreError::UnknownBackend {
            backend: backend.to_owned(),
          }
          .into(),
        ),
      }
    }
    else
    {
      Self::open_redb(path)
    }
  }
  #[cfg(feature = "redb")]
  fn open_redb<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Connection>
  {
    Ok(Connection {
      connection: ConnectionImpl {
        store: crate::store::redb::Store::new(path)?,
        function_manager: crate::functions::Manager::new(),
      }
      .boxed(),
    })
  }
  #[cfg(not(feature = "redb"))]
  fn open_redb<P: AsRef<std::path::Path>>(_: P) -> crate::Result<Connection>
  {
    Err(error::StoreError::UnavailableBackend { backend: "redb" }.into())
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
    self.connection.execute_query(query.into(), parameters)
  }
}
