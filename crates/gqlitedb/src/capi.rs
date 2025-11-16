use std::borrow::Borrow;

fn handle_error<T, E: std::borrow::Borrow<E> + ToString>(
  context: *mut GqliteApiContextT,
  result: Result<T, E>,
) -> Result<T, E>
{
  match &result
  {
    Ok(_) =>
    {}
    Err(e) =>
    {
      let mut context = unsafe { Box::from_raw(context) };
      context.string = std::ffi::CString::new(e.borrow().to_string()).unwrap();
      context.has_error = true;
      let _ = Box::into_raw(context);
    }
  }
  result
}

fn get_value(value: *mut GqliteValueT) -> crate::value::Value
{
  if value.is_null()
  {
    crate::value::Value::default()
  }
  else
  {
    let value = unsafe { Box::from_raw(value) };
    let v = value.value.clone();
    let _ = Box::into_raw(value);
    v
  }
}

fn check_error(context: *mut GqliteApiContextT)
{
  if unsafe { (*context).has_error }
  {
    let context = unsafe { Box::from_raw(context) };
    let stri = context.string.to_str().unwrap();
    println!("Error message '{stri:?}' was not cleared from context!");
    let _ = Box::into_raw(context);
  }
}

#[repr(C)]
pub struct GqliteApiContextT
{
  string: std::ffi::CString,
  has_error: bool,
}

#[repr(C)]
pub struct GqliteConnectionT
{
  connection: crate::Connection,
}

#[repr(C)]
pub struct GqliteValueT
{
  value: crate::value::Value,
}

#[no_mangle]
pub extern "C" fn gqlite_api_context_create() -> *mut GqliteApiContextT
{
  Box::into_raw(Box::new(GqliteApiContextT {
    string: std::ffi::CString::new("").unwrap(),
    has_error: false,
  }))
}

#[no_mangle]
pub extern "C" fn gqlite_api_context_destroy(context: *mut GqliteApiContextT)
{
  unsafe {
    let _ = Box::from_raw(context);
  }
}

#[no_mangle]
pub extern "C" fn gqlite_api_context_get_message(
  context: *mut GqliteApiContextT,
) -> *const std::ffi::c_char
{
  unsafe { (*context).string.as_ptr() }
}

#[no_mangle]
pub extern "C" fn gqlite_api_context_has_error(context: *mut GqliteApiContextT) -> bool
{
  unsafe { (*context).has_error }
}

#[no_mangle]
pub extern "C" fn gqlite_api_context_clear_error(context: *mut GqliteApiContextT)
{
  let mut context = unsafe { Box::from_raw(context) };
  context.has_error = false;
  let _ = Box::into_raw(context);
}

#[no_mangle]
pub extern "C" fn gqlite_connection_create_from_file(
  context: *mut GqliteApiContextT,
  filename: *const std::ffi::c_char,
  options: *mut GqliteValueT,
) -> *mut GqliteConnectionT
{
  check_error(context);
  let options = get_value(options);
  let path = unsafe { std::ffi::CStr::from_ptr(filename) };

  if let Ok(path) = handle_error(context, path.to_str())
  {
    if let Ok(c) = handle_error(
      context,
      crate::Connection::builder()
        .options(options.into_map())
        .path(path)
        .create(),
    )
    {
      return Box::into_raw(Box::new(GqliteConnectionT { connection: c }));
    }
  }
  std::ptr::null::<GqliteConnectionT>() as *mut GqliteConnectionT
}

#[no_mangle]
pub extern "C" fn gqlite_connection_create(
  context: *mut GqliteApiContextT,
  options: *mut GqliteValueT,
) -> *mut GqliteConnectionT
{
  check_error(context);
  let options = get_value(options);

  if let Ok(c) = handle_error(context, crate::Connection::create(options.into_map()))
  {
    return Box::into_raw(Box::new(GqliteConnectionT { connection: c }));
  }
  std::ptr::null::<GqliteConnectionT>() as *mut GqliteConnectionT
}

#[no_mangle]
pub extern "C" fn gqlite_connection_destroy(
  _context: *mut GqliteApiContextT,
  connection: *mut GqliteConnectionT,
)
{
  unsafe {
    let _ = Box::from_raw(connection);
  }
}

#[no_mangle]
pub extern "C" fn gqlite_connection_query(
  context: *mut GqliteApiContextT,
  connection: *mut GqliteConnectionT,
  query: *const std::ffi::c_char,
  bindings: *mut GqliteValueT,
) -> *mut GqliteValueT
{
  check_error(context);
  let query = unsafe { std::ffi::CStr::from_ptr(query) };
  if let Ok(query) = handle_error(context, query.to_str())
  {
    let conn = unsafe { Box::from_raw(connection) };
    let result = conn
      .connection
      .execute_oc_query(query, get_value(bindings).into_map());
    let _ = Box::into_raw(conn);
    if let Ok(v) = handle_error(context, result)
    {
      return Box::into_raw(Box::new(GqliteValueT {
        value: v.into_value(),
      }));
    }
  }
  std::ptr::null::<GqliteValueT>() as *mut GqliteValueT
}

#[no_mangle]
pub extern "C" fn gqlite_value_create(context: *mut GqliteApiContextT) -> *mut GqliteValueT
{
  check_error(context);
  Box::into_raw(Box::new(GqliteValueT {
    value: crate::value::Value::default(),
  }))
}

#[no_mangle]
pub extern "C" fn gqlite_value_destroy(context: *mut GqliteApiContextT, value: *mut GqliteValueT)
{
  check_error(context);
  unsafe {
    let _ = Box::from_raw(value);
  }
}

#[no_mangle]
pub extern "C" fn gqlite_value_to_json(
  context: *mut GqliteApiContextT,
  value: *mut GqliteValueT,
) -> *const std::ffi::c_char
{
  check_error(context);

  let value = unsafe { Box::from_raw(value) };

  if let Ok(v) = handle_error(context, serde_json::to_string(value.value.borrow()))
  {
    let mut context = unsafe { Box::from_raw(context) };
    context.string = std::ffi::CString::new(v).unwrap();
    context.has_error = false;
    let r = context.string.as_ptr();
    let _ = Box::into_raw(context);
    let _ = Box::into_raw(value);
    return r;
  }
  let _ = Box::into_raw(value);
  std::ptr::null()
}

#[no_mangle]
pub extern "C" fn gqlite_value_from_json(
  context: *mut GqliteApiContextT,
  json: *const std::ffi::c_char,
) -> *mut GqliteValueT
{
  check_error(context);

  let json = unsafe { std::ffi::CStr::from_ptr(json) };
  if let Ok(json) = handle_error(context, json.to_str())
  {
    if let Ok(v) = handle_error(context, serde_json::from_str::<crate::value::Value>(json))
    {
      return Box::into_raw(Box::new(GqliteValueT { value: v }));
    }
  }
  std::ptr::null::<GqliteValueT>() as *mut GqliteValueT
}

#[no_mangle]
pub extern "C" fn gqlite_value_is_valid(
  context: *mut GqliteApiContextT,
  value: *mut GqliteValueT,
) -> bool
{
  check_error(context);
  !matches!(
    unsafe { (*value).value.borrow() },
    crate::value::Value::Null
  )
}
