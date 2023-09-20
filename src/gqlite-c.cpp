#include "gqlite-c.h"
#include "gqlite.h"
#include "gqlite_p.h"

#include "logging.h"

#define BEGIN_CHECK                       \
  if(not _context->error_message.empty()) \
  {                                       \
    gqlite_error("Error message '{}' was not cleared from context!", _context->error_message);  \
  }                                       \
  try                                     \
  {
#define END_CHECK                                 \
  } catch(const gqlite::exception& _ex)           \
  {                                               \
    _context->error_message = _ex.what();         \
    if(_context->error_message.empty())           \
    {                                             \
      _context->error_message = "Unknown error";  \
    }                                             \
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

  struct gqlite_connection
  {
    gqlite::connection db;
  };
  struct gqlite_value
  {
    gqlite::value value;
  };

  gqlite_connection_t gqlite_connection_create_from_sqlite(gqlite_api_context_t _context, void* _handle, gqlite_value_t _options)
  {
    BEGIN_CHECK
    gqlite::value options;
    if(_options) options = _options->value;
    return new gqlite_connection{gqlite::connection::create_from_sqlite(_handle, options)};
    END_CHECK
    return nullptr;
  }
  gqlite_connection_t gqlite_connection_create_from_sqlite_file(gqlite_api_context_t _context, const char* _filename, gqlite_value_t _options)
  {
    BEGIN_CHECK
    gqlite::value options;
    if(_options) options = _options->value;
    return new gqlite_connection{gqlite::connection::create_from_sqlite_file(_filename, options)};
    END_CHECK
    return nullptr;
  }

  void gqlite_connection_destroy(gqlite_api_context_t _context, gqlite_connection_t _connection)
  {
    BEGIN_CHECK
    delete _connection;
    END_CHECK
  }

  gqlite_value_t gqlite_connection_oc_query(gqlite_api_context_t _context, gqlite_connection_t _connection, const char* _query, gqlite_value_t _bindings)
  {
    BEGIN_CHECK
    gqlite::value_map bindings;
    if(_bindings)
    {
      bindings = _bindings->value.to_map();
    }
    return new gqlite_value{ _connection->db.execute_oc_query(_query, bindings) };
    END_CHECK
    return nullptr;
  }

  gqlite_value_t gqlite_value_create(gqlite_api_context_t)
  {
    return new gqlite_value;
  }
  void gqlite_value_destroy(gqlite_api_context_t, gqlite_value_t _value)
  {
    delete _value;
  }
  const char* gqlite_value_to_json(gqlite_api_context_t _context, gqlite_value_t _value)
  {
    BEGIN_CHECK
    return _context->set_string(_value->value.to_json());
    END_CHECK
    return nullptr;
  }
  gqlite_value_t gqlite_value_from_json(gqlite_api_context_t _context, const char* _json)
  {
    BEGIN_CHECK
    return new gqlite_value{gqlite::value::from_json(_json)};
    END_CHECK
    return nullptr;
  }

  bool gqlite_value_is_valid(gqlite_api_context_t, gqlite_value_t _value)
  {
    return _value and _value->value.get_type() != gqlite::value_type::invalid;
  }

  gqlite_value_t gqlite_private_test_stats(gqlite_connection_t _connection)
  {
    return new gqlite_value { _connection->db.get_debug_stats() };
  }

}
