#include "gqlite-c.h"
#include "gqlite.h"

#include "logging.h"

#define BEGIN_CHECK                       \
  if(not _context->error_message.empty()) \
  {                                       \
    gqlite_error("Error message '{}' was not cleared from context!", _context->error_message);  \
  }                                       \
  try                                     \
  {
#define END_CHECK                         \
  } catch(const gqlite::exception& _ex)   \
  {                                       \
    _context->error_message = _ex.what(); \
  }

extern "C"
{

  struct gqlite_api_context
  {
    std::string error_message;
    ~gqlite_api_context()
    {
      release_memory();
    }
    char* str = nullptr;
    char* set_string(const std::string& _string)
    {
      release_memory();
      str = new char[_string.size() + 1];
      std::copy(_string.begin(), _string.end() + 1, str);
      return str;
    }
    void release_memory()
    {
      delete[] str;
      str = nullptr;
    }
  };

  gqlite_api_context_t gqlite_api_context_create()
  {
    return new gqlite_api_context;
  }
  void gqlite_api_context_destroy(gqlite_api_context_t _context)
  {
    delete _context;
  }
  const char* gqlite_api_context_get_message(gqlite_api_context_t _context)
  {
    return _context->error_message.c_str();
  }
  bool gqlite_api_context_has_error(gqlite_api_context_t _context)
  {
    return not _context->error_message.empty();
  }
  void gqlite_api_context_clear_error(gqlite_api_context_t _context)
  {
    _context->error_message.clear();
  }

  struct gqlite_database
  {
    gqlite::database db;
  };
  struct gqlite_value
  {
    gqlite::value value;
  };

  gqlite_database_t gqlite_database_create_from_sqlite(gqlite_api_context_t _context, void* _handle)
  {
    BEGIN_CHECK
    return new gqlite_database{gqlite::database::create_from_sqlite(_handle)};
    END_CHECK
  }
  gqlite_database_t gqlite_database_create_from_sqlite_file(gqlite_api_context_t _context, const char* _filename)
  {
    BEGIN_CHECK
    return new gqlite_database{gqlite::database::create_from_sqlite_file(_filename)};
    END_CHECK
  }

  void gqlite_database_destroy(gqlite_api_context_t _context, gqlite_database_t _database)
  {
    BEGIN_CHECK
    delete _database;
    END_CHECK
  }

  gqlite_value_t gqlite_database_oc_query(gqlite_api_context_t _context, gqlite_database_t _database, const char* _query, gqlite_value_t _bindings)
  {
    BEGIN_CHECK
    std::unordered_map<std::string, gqlite::value> bindings;
    if(_bindings)
    {
      bindings = _bindings->value.to_map();
    }
    return new gqlite_value{ _database->db.execute_oc_query(_query, bindings) };
    END_CHECK
  }

  gqlite_value_t gqlite_value_create(gqlite_api_context_t)
  {
    return new gqlite_value;
  }
  void gqlite_value_destroy(gqlite_api_context_t, gqlite_value_t _value)
  {
    delete _value;
  }

}
