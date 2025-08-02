use std::path::Path;

use crate::prelude::*;
use value::ValueTryIntoRef;

/// Backend
pub enum Backend
{
  /// Select the first available backend.
  Automatic,
  /// SQLite backend.
  #[cfg(feature = "sqlite")]
  SQLite,
  /// Redb backend.
  #[cfg(feature = "redb")]
  Redb,
}

/// Builder with high-level API for creating connection.
pub struct ConnectionBuilder
{
  map: value::ValueMap,
}

impl ConnectionBuilder
{
  /// Merge options. This might overwrite value from the builder
  pub fn options(mut self, options: value::ValueMap) -> Self
  {
    for (k, v) in options.into_iter()
    {
      self.map.insert(k, v);
    }
    self
  }
  /// Set path
  pub fn path<P: AsRef<Path>>(mut self, p: P) -> Self
  {
    self.map.insert(
      "path".to_string(),
      p.as_ref().to_string_lossy().as_ref().into(),
    );
    self
  }
  /// Set backend
  pub fn backend(mut self, backend: Backend) -> Self
  {
    let key = "backend".into();
    match backend
    {
      Backend::Automatic =>
      {
        self.map.insert(key, "automatic".into());
      }
      #[cfg(feature = "sqlite")]
      Backend::SQLite =>
      {
        self.map.insert(key, "sqlite".into());
      }
      #[cfg(feature = "redb")]
      Backend::Redb =>
      {
        self.map.insert(key, "redb".into());
      }
    }
    self
  }
  /// Create the connection
  pub fn create(self) -> Result<Connection>
  {
    Connection::create(self.map)
  }
}

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
  /// Create a new connection to a `GQLite` database. The `options` parameter can
  /// be used to select the backend, and configure the backend.
  ///
  /// Supported parameters:
  /// - `path` a path to a file, if not present, an in-memory database is created
  /// - `backend` for instance `redb` or `sqlite` (the [Self::available_backends] function contains the list of compiled backends)
  ///
  /// If the `backend` is not specified, the `open` function will attempt to guess it
  /// for existing databases. For new database, depending on availability, it will
  /// create a `sqlite` database, or a `redb` database.
  ///
  /// Example of use, this will create an in-memory database:
  ///
  /// ```rust
  /// # use gqlitedb::Connection;
  /// # fn example() -> gqlitedb::Result<()> {
  /// let connection = Connection::create(gqlitedb::map!("backend" => "redb"))?;
  /// # Ok(()) }
  /// ```  
  pub fn create(options: value::ValueMap) -> Result<Connection>
  {
    let backend = options.get("backend").map_or_else(
      || Ok("automatic".to_string()),
      |x| x.try_into_ref().map(|x: &String| x.to_owned()),
    )?;
    match backend.as_str()
    {
      "automatic" =>
      {
        #[cfg(feature = "sqlite")]
        let sq_e = {
          let mut options = options.clone();
          options.insert("backend".into(), "sqlite".into());
          Self::create(options)
        };
        #[cfg(not(feature = "sqlite"))]
        let sq_e = Err(error::StoreError::UnavailableBackend { backend: "sqlite" }.into());
        let sq_r = match sq_e
        {
          Ok(sq) => Ok(sq),
          Err(sq_e) =>
          {
            #[cfg(feature = "redb")]
            let sq_r = {
              let mut options = options;
              options.insert("backend".into(), "redb".into());
              Self::create(options)
            };
            #[cfg(not(feature = "redb"))]
            let sq_r = Err(error::StoreError::UnavailableBackend { backend: "redb" }.into());

            sq_r.map_err(|rb_e| {
              StoreError::OpeningError {
                errors: error::vec_to_error::<ErrorType>(&vec![sq_e, rb_e]),
              }
              .into()
            })
          }
        };
        sq_r
      }
      #[cfg(feature = "sqlite")]
      "sqlite" =>
      {
        let store = if let Some(path) = options.get("path")
        {
          let path: &String = path.try_into_ref()?;
          store::sqlite::Store::open(path)?
        }
        else
        {
          store::sqlite::Store::in_memory()?
        };
        Ok(Connection {
          connection: ConnectionImpl {
            store,
            function_manager: functions::Manager::new(),
          }
          .boxed(),
        })
      }
      #[cfg(feature = "redb")]
      "redb" =>
      {
        let store = if let Some(path) = options.get("path")
        {
          let path: &String = path.try_into_ref()?;
          store::redb::Store::open(path)?
        }
        else
        {
          store::redb::Store::in_memory()?
        };
        Ok(Connection {
          connection: ConnectionImpl {
            store,
            function_manager: functions::Manager::new(),
          }
          .boxed(),
        })
      }
      _ => Err(StoreError::UnknownBackend { backend }.into()),
    }
  }
  /// Create a builder, with a high-level API to set the options.
  /// Example of use:
  /// ```
  /// let connection = Connection::builder().path("path/to/file").backend(Backend::SQLite).create()?;
  /// ```
  pub fn builder() -> ConnectionBuilder
  {
    ConnectionBuilder {
      map: Default::default(),
    }
  }
  /// List of available backends
  pub fn available_backends() -> Vec<String>
  {
    let mut backends = vec![];
    #[cfg(feature = "sqlite")]
    backends.push("sqlite".to_string());
    #[cfg(feature = "redb")]
    backends.push("redb".to_string());
    backends
  }

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
  #[deprecated = "Use create or builder instead."]
  pub fn open<P: AsRef<std::path::Path>>(path: P, options: value::ValueMap) -> Result<Connection>
  {
    Self::builder().options(options).path(path).create()
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
