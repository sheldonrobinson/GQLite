import __c_gqlite as capi
import json

class Error(Exception):
  def __init__(self, msg):
    self.msg = msg

api_context = capi.lib.gqlite_api_context_create()

class Connection(object):
  def __init__(self, filename = None) -> None:
    self.empty_value = self.__call_capi(capi.lib.gqlite_value_create)
    if filename != None:
      self.handle = self.__call_capi(capi.lib.gqlite_connection_create_from_file, filename.encode('utf-8'), capi.ffi.NULL  )
    else:
      raise Error("No connection backend was selected.")
  def execute_oc_query(self, query, bindings = None):
    ret = self.__call_capi(capi.lib.gqlite_connection_query, self.handle, query.encode('utf-8'), capi.ffi.NULL)
    if self.__call_capi(capi.lib.gqlite_value_is_valid, ret):
      val = json.loads(capi.ffi.string(self.__call_capi(capi.lib.gqlite_value_to_json, ret)))
    else:
      return None
    self.__call_capi(capi.lib.gqlite_value_destroy, ret)
    return val

  def __call_capi(self, name, *args):
    ret = name(api_context, *args)
    if capi.lib.gqlite_api_context_has_error(api_context):
      err = capi.ffi.string(capi.lib.gqlite_api_context_get_message(api_context)).decode('utf-8')
      capi.lib.gqlite_api_context_clear_error(api_context)
      raise Error(err)
    return ret

def connect(*args):
  return Connection(*args)