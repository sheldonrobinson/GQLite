
mod gqlite
{
  use std::ffi::CStr;
  use std::ffi::CString;
  use std::os::raw::c_char;

  #[repr(C)]
  struct gqlite_api_context {
      _data: [u8; 0],
      _marker:
      core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
  }
  
  #[repr(C)]
  struct gqlite_connection {
    _data: [u8; 0],
      _marker:
          core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
  }

  #[repr(C)]
  struct gqlite_value {
      _data: [u8; 0],
      _marker:
          core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
  }

  #[link(name = "gqlite")]
  extern "C" {
    // Context API
    fn gqlite_api_context_create() -> *mut gqlite_api_context;
    fn gqlite_api_context_destroy(context: *mut gqlite_api_context);
    fn gqlite_api_context_get_message(context: *mut gqlite_api_context) -> *const c_char;
    fn gqlite_api_context_has_error(context: *mut gqlite_api_context) -> bool;
    fn gqlite_api_context_clear_error(context: *mut gqlite_api_context);

    // Query API
    fn gqlite_connection_create_from_sqlite_file(context: *mut gqlite_api_context, filename: *const c_char, options: *mut gqlite_value) -> *mut gqlite_connection;
    fn gqlite_connection_destroy(context: *mut gqlite_api_context, connection: *mut gqlite_connection);

    fn gqlite_connection_oc_query(context: *mut gqlite_api_context, connection: *mut gqlite_connection, json: *const c_char, value: *mut gqlite_value) -> *mut gqlite_value;

    // Value API
    fn gqlite_value_create(context: *mut gqlite_api_context) -> *mut gqlite_value;
    fn gqlite_value_destroy(context: *mut gqlite_api_context, value: *mut gqlite_value);

    fn gqlite_value_to_json(context: *mut gqlite_api_context, value: *mut gqlite_value) -> *const c_char;
    fn gqlite_value_from_json(context: *mut gqlite_api_context, json: *const c_char) -> *mut gqlite_value;
    fn gqlite_value_is_valid(context: *mut gqlite_api_context, value: *mut gqlite_value) -> bool;
  }

  pub struct Connection
  {
    context: *mut gqlite_api_context,
    connection: *mut gqlite_connection
  }

  #[derive(Debug)]
  pub struct Error {
      details: String
  }

  impl Error {
    fn new(msg: impl Into<String>) -> Error {
      Error{details: msg.into()}
    }
    pub fn description(&self) -> &str {
      &self.details
    }
  }

  impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f,"{}",self.details)
    }
  }

  impl std::error::Error for Error {
    fn description(&self) -> &str {
      &self.details
    }
  }

  impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::new(err.to_string())
    }
  }

  impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Error::new(err.to_string())
    }
  }

  fn check_error(_context: *mut gqlite_api_context) -> Result<(),Error>
  {
    unsafe {
      if gqlite_api_context_has_error(_context)
      {
        let err = Error::new(CStr::from_ptr(gqlite_api_context_get_message(_context)).to_str()?);
        gqlite_api_context_clear_error(_context);
        Err(err)
      } else {
        Ok(())
      }
    }
  }

  impl Connection
  {
    pub fn new(filename: String) -> Result<Connection, Error>
    {
      unsafe {
        let filename_cstr = CString::new(filename)?;
        let ctx = gqlite_api_context_create();
        let opts: *mut gqlite_value = std::ptr::null_mut();
        let conn = gqlite_connection_create_from_sqlite_file(ctx, filename_cstr.as_ptr(), opts);
        check_error(ctx)?;
        Ok(Connection {
          context: ctx,
          connection: conn
        })
      }
    }
    pub fn execute_oc_query(&self, query: impl Into<String>, bindings: Option<String>) -> Result<String, Error>
    {
      unsafe {
        let query_cstr = CString::new(query.into())?;
        let bindings_cstr = CString::new("{}")?;
        let bindings_value = gqlite_value_from_json(self.context, bindings_cstr.as_ptr());
        check_error(self.context)?;
        let result = gqlite_connection_oc_query(self.context, self.connection, query_cstr.as_ptr(), bindings_value);
        check_error(self.context)?;
        let result_str = CStr::from_ptr(gqlite_value_to_json(self.context, result)).to_str()?;
        check_error(self.context)?;
        gqlite_value_destroy(self.context, bindings_value);
        check_error(self.context)?;
        Ok(result_str.into())
      }
    }
  }
}

#[cfg(test)]
mod gqlite_tests {
  use gqlite;
  fn test_create_db() -> Result<gqlite::Connection, gqlite::Error>
  {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = env::temp_dir();
    gqlite::Connection::new(format!("{}/gqlite_testdb_{}", dir.display(), nanos))
  }
  #[test]
  fn test_simple_queries() {
    let conn = test_create_db().unwrap();
    assert_eq!(conn.execute_oc_query("CREATE (n)", None).unwrap(), "null");
    assert_eq!(conn.execute_oc_query("MATCH (n) RETURN n", None).unwrap(), "[[\"n\"],[{\"type\":\"node\",\"labels\":[],\"properties\":{},\"id\":1}]]");
    let failed = conn.execute_oc_query("MATCH (n", None);
    assert!(failed.is_err());
    assert_eq!(failed.unwrap_err().description(), "CompileTime: UnexpectedSyntax: Expected token ) got end of file at (1, 8).");
  }
}