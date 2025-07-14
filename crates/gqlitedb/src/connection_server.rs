use crate::prelude::*;

/// Connection server allowing to share a connection between threads. This feature is experimental and is gated behind the "_connection_server".
pub struct ConnectionServer
{
  server:
    ccutils::servers::SingleServer<(String, value::ValueMap), Result<value::Value, ErrorType>>,
}

impl ConnectionServer
{
  /// Open a new connection. To the given path and options.
  pub fn open<P: AsRef<std::path::Path>>(
    path: P,
    options: crate::value::ValueMap,
  ) -> crate::Result<ConnectionServer>
  {
    let path = path.as_ref().to_owned();
    let server = ccutils::servers::SingleServer::new_fallible(
      |connection: connection::Connection, (query, bindings)| {
        let result = connection.execute_query(query, bindings);
        (connection, result)
      },
      move || connection::Connection::open(path, options),
      ccutils::servers::Options::default().stack_size(8 * 1024 * 1024),
    )?;
    Ok(ConnectionServer { server })
  }
  /// Execute a query.
  pub fn execute_query(
    &self,
    query: impl Into<String>,
    parameters: crate::value::ValueMap,
  ) -> crate::Result<crate::value::Value>
  {
    self
      .server
      .request_sync((query.into(), parameters))
      .unwrap()
  }
}
