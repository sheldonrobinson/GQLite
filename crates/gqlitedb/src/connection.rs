use crate::prelude::*;
use value::ValueTryIntoRef;

trait ConnectionTrait: Sync + Send
{
  fn execute_query(&self, query: String, parameters: value::ValueMap) -> Result<value::Value>;
}

struct ConnectionImpl<TStore>
where
  TStore: store::Store + Sync + Send,
{
  store: TStore,
  function_manager: functions::Manager,
}

impl<TStore> ConnectionTrait for ConnectionImpl<TStore>
where
  TStore: store::Store + Sync + Send,
{
  fn execute_query(&self, query: String, parameters: value::ValueMap) -> Result<value::Value>
  {
    let query_txt: String = query.into();
    let queries = parser::parse(query_txt.as_str())?;
    let mut results = Vec::<value::Value>::default();
    for query in queries
    {
      let program = compiler::compile(&self.function_manager, query)?;
      let v = interpreter::evaluators::eval_program(&self.store, &program, &parameters)?;
      if !v.is_null()
      {
        results.push(v);
      }
    }
    match results.len()
    {
      0 => Ok(value::Value::Null),
      1 => Ok(results.into_iter().next().unwrap()),
      _ =>
      {
        let mut map = value::ValueMap::new();
        map.insert("type".into(), "results".into());
        map.insert("results".into(), results.into());
        Ok(map.into())
      }
    }
  }
}

impl<TStore: store::Store> ConnectionImpl<TStore>
where
  TStore: store::Store + Sync + Send,
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

ccutils::assert_impl_all!(Connection: Sync, Send);

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
  pub fn open<P: AsRef<std::path::Path>>(path: P, options: value::ValueMap) -> Result<Connection>
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
  fn open_sqlite<P: AsRef<std::path::Path>>(path: P) -> Result<Connection>
  {
    Ok(Connection {
      connection: ConnectionImpl {
        store: store::sqlite::Store::new(path)?,
        function_manager: functions::Manager::new(),
      }
      .boxed(),
    })
  }
  #[cfg(not(feature = "sqlite"))]
  fn open_sqlite<P: AsRef<std::path::Path>>(_: P) -> Result<Connection>
  {
    Err(error::ConnectionError::UnavailableBackend { backend: "sqlite" }.into())
  }
  #[cfg(feature = "redb")]
  fn open_redb<P: AsRef<std::path::Path>>(path: P) -> Result<Connection>
  {
    Ok(Connection {
      connection: ConnectionImpl {
        store: store::redb::Store::new(path)?,
        function_manager: functions::Manager::new(),
      }
      .boxed(),
    })
  }
  #[cfg(not(feature = "redb"))]
  fn open_redb<P: AsRef<std::path::Path>>(_: P) -> Result<Connection>
  {
    Err(error::StoreError::UnavailableBackend { backend: "redb" }.into())
  }
  #[cfg(feature = "_pgql")]
  pub fn create() -> Result<Connection>
  {
    Ok(Connection {
      store: store::Store::new()?,
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
    parameters: value::ValueMap,
  ) -> Result<value::Value>
  {
    self.connection.execute_query(query.into(), parameters)
  }
}
