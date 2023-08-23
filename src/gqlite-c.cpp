#include "gqlite-c.h"
#include "gqlite.h"

extern "C"
{
  struct gqlite_api_error
  {
    std::string error_message;
  };

  gqlite_api_error_t gqlite_api_error_create()
  {
    return new gqlite_api_error;
  }
  void gqlite_api_error_destroy(gqlite_api_error_t _error)
  {
    delete _error;
  }
  const char* gqlite_api_error_get_message(gqlite_api_error_t _error)
  {
    return _error->error_message.c_str();
  }
  bool gqlite_api_error_has_error(gqlite_api_error_t _error)
  {
    return not _error->error_message.empty();
  }

  struct gqlite_database
  {
    gqlite::database db;
  };
  struct gqlite_result
  {
    gqlite::result result;
  };
  struct gqlite_bindings
  {
    std::unordered_map<std::string, std::any> bindings;
  };

  gqlite_database_t gqlite_database_create_sqlite(gqlite_api_error_t, void* _handle)
  {
    return new gqlite_database{gqlite::database::create_from_sqlite(_handle)}; 
  }

  void gqlite_database_destroy(gqlite_api_error_t, gqlite_database_t _database)
  {
    delete _database;
  }

  gqlite_result_t gqlite_database_oc_query(gqlite_api_error_t, gqlite_database_t _database, const char* _query, gqlite_bindings_t _bindings)
  {
    std::unordered_map<std::string, std::any> bindings;
    if(_bindings)
    {
      bindings = _bindings->bindings;
    }
    return new gqlite_result{ _database->db.execute_oc_query(_query, bindings) };
  }

  gqlite_bindings_t gqlite_bindings_create(gqlite_api_error_t)
  {
    return new gqlite_bindings;
  }

}
