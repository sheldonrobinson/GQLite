from cffi import FFI

import sys

def build(include_dirs, library_dirs, libraries):

  ffibuilder = FFI()
  ffibuilder.set_source("__c_gqlite", "#include <gqlite-c.h>", include_dirs = include_dirs, libraries=libraries, library_dirs=library_dirs)
  ffibuilder.cdef(
  """

  typedef struct gqlite_api_context* gqlite_api_context_t;
  typedef struct gqlite_connection* gqlite_connection_t;
  typedef struct gqlite_backend* gqlite_backend_t;
  typedef struct gqlite_value* gqlite_value_t;

  gqlite_api_context_t gqlite_api_context_create();
  void gqlite_api_context_destroy(gqlite_api_context_t);
  const char* gqlite_api_context_get_message(gqlite_api_context_t);
  bool gqlite_api_context_has_error(gqlite_api_context_t);
  void gqlite_api_context_clear_error(gqlite_api_context_t);
  gqlite_connection_t gqlite_connection_create_from_sqlite(gqlite_api_context_t, void*, gqlite_value_t _options);
  gqlite_connection_t gqlite_connection_create_from_sqlite_file(gqlite_api_context_t, const char*, gqlite_value_t _options);
  void gqlite_connection_destroy(gqlite_api_context_t, gqlite_connection_t);
  gqlite_value_t gqlite_connection_oc_query(gqlite_api_context_t, gqlite_connection_t, const char*, gqlite_value_t);
  gqlite_value_t gqlite_value_create(gqlite_api_context_t);
  void gqlite_value_destroy(gqlite_api_context_t, gqlite_value_t);
  const char* gqlite_value_to_json(gqlite_api_context_t, gqlite_value_t);
  bool gqlite_value_is_valid(gqlite_api_context_t, gqlite_value_t);

  """)

  return ffibuilder

def build_for_pip():
  return build(["src/"], ["build"], ["gqlite", "stdc++", "sqlite3"])

if __name__ == "__main__":
  build([sys.argv[1] + "/include"], ["."], ["gqlite"]).compile(verbose=True)