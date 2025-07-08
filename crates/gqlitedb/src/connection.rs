use crate::prelude::*;
use value::ValueTryIntoRef;

trait ConnectionTrait
{
  fn execute_query(
    &self,
    query: String,
    parameters: crate::value::ValueMap,
  ) -> crate::Result<crate::value::Value>;
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
    parameters: crate::value::ValueMap,
  ) -> crate::Result<crate::value::Value>
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

/// Connection is the interface to the database, and allow to execute new queries.
/// New connection are created with [Connection::open] and queried with [Connection::execute_query].
/// As shown in the example bellow:
///
/// ```rust
/// # use gqlitedb::{Connection, Value};
/// # fn example() -> gqlitedb::Result<()> {
/// let connection = Connection::open("filename.db", gqlitedb::map!("backend" => "redb"))?;
/// let value = connection.execute_query("MATCH (a) RETURN a", Default::default())?;
/// match value
/// {
///   Value::Array(arr) =>
///   {
///     arr.iter().for_each(|row| match row
///     {
///       Value::Array(arr) =>
///       {
///         println!("{:?}", arr);
///       }
///       _ =>
///       {
///         panic!("Unexpected: {}", row);
///       }
///     });
///   },
///   _ => {
///     panic!("Query result should be an array, got {}!", value);
///   }
/// }
/// # Ok(()) }
/// ```

pub struct Connection
{
  connection: Box<dyn ConnectionTrait>,
}

impl Connection
{
  /// Open a `path` that contains a `GQLite` database. The `options` parameter can
  /// be used to select the backend, and configure the backend.
  ///
  /// Supported parameters:
  /// - `backend` can be `redb` or `sqlite`
  ///
  /// If the `backend` is not specified, the `open` function will attempt to guess it
  /// for existing databases. For new database, depending on availability, it will
  /// create a `sqlite` database, or a `redb` database.
  ///
  /// Example of use:
  ///
  /// ```rust
  /// # use gqlitedb::Connection;
  /// # fn example() -> gqlitedb::Result<()> {
  /// let connection = Connection::open("filename.db", gqlitedb::map!("backend" => "redb"))?;
  /// # Ok(()) }
  /// ```  
  #[cfg(any(feature = "redb", feature = "sqlite"))]
  pub fn open<P: AsRef<std::path::Path>>(
    path: P,
    options: crate::value::ValueMap,
  ) -> crate::Result<Connection>
  {
    if let Some(backend) = options.get("backend")
    {
      let backend: &String = backend.try_into_ref()?;
      match backend.as_str()
      {
        "sqlite" => Self::open_sqlite(path),
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
      Self::open_sqlite(path.as_ref().to_owned()).or_else(|sq_e| {
        Self::open_redb(path).map_err(|rb_e| {
          StoreError::OpeningError {
            errors: error::vec_to_error::<ErrorType>(&vec![sq_e, rb_e]),
          }
          .into()
        })
      })
    }
  }
  #[cfg(feature = "sqlite")]
  fn open_sqlite<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Connection>
  {
    Ok(Connection {
      connection: ConnectionImpl {
        store: crate::store::sqlite::Store::new(path)?,
        function_manager: crate::functions::Manager::new(),
      }
      .boxed(),
    })
  }
  #[cfg(not(feature = "sqlite"))]
  fn open_sqlite<P: AsRef<std::path::Path>>(_: P) -> crate::Result<Connection>
  {
    Err(error::ConnectionError::UnavailableBackend { backend: "sqlite" }.into())
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
  #[cfg(feature = "_pgql")]
  pub fn create() -> crate::Result<Connection>
  {
    Ok(Connection {
      store: crate::store::Store::new()?,
    })
  }
  /// Execute the `query` (using OpenCypher), given the query `parameters` (sometimes
  /// also referred as binding).
  ///
  /// Example:
  ///
  /// ```rust
  /// # use gqlitedb::{Connection, Value};
  /// # fn example() -> gqlitedb::Result<()> {
  /// # let connection = gqlitedb::Connection::open("filename.db", gqlitedb::map!("backend" => "redb"))?;
  /// let result = connection.execute_query("MATCH (a { name: $name }) RETURN a", gqlitedb::map!("name" => "Joe"))?;
  /// # Ok(()) }
  /// ```
  pub fn execute_query(
    &self,
    query: impl Into<String>,
    parameters: crate::value::ValueMap,
  ) -> crate::Result<crate::value::Value>
  {
    self.connection.execute_query(query.into(), parameters)
  }
}
