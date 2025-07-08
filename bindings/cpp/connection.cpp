#include "gqlite.h"

#include "exception.h"
#include "gqlite-c.h"
#include "value.h"

using namespace gqlite;

namespace
{
  void check_errors(gqlite_api_context_t _api_context)
  {
    if(gqlite_api_context_has_error(_api_context))
    {
      std::string msg{gqlite_api_context_get_message(_api_context)};
      gqlite_api_context_clear_error(_api_context);
      throw_exception(msg);
    }
  }

  struct gqlite_value_wrapper
  {
    gqlite_value_wrapper(gqlite_api_context_t _api_context, const value& _value)
        : api_context(_api_context)
    {
      gqlite_value = gqlite_value_from_json(_api_context, _value.to_json().c_str());
      check_errors(api_context);
    }
    gqlite_value_wrapper(gqlite_api_context_t _api_context, const gqlite_value_t& _gqlite_value)
        : api_context(_api_context), gqlite_value(_gqlite_value)
    {
    }
    ~gqlite_value_wrapper() { gqlite_value_destroy(api_context, gqlite_value); }
    value to_value() const
    {
      value val = value::from_json(gqlite_value_to_json(api_context, gqlite_value));
      check_errors(api_context);
      return val;
    }
    gqlite_api_context_t api_context{nullptr};
    gqlite_value_t gqlite_value{nullptr};
  };
} // namespace

struct connection::data
{
  gqlite_api_context_t api_context{nullptr};
  gqlite_connection_t connection{nullptr};

  ~data()
  {
    gqlite_connection_destroy(api_context, connection);
    gqlite_api_context_destroy(api_context);
  }

  void check_valid()
  {
    if(not api_context or not connection)
    {
      throw_exception("Invalid connection.");
    }
  }
};

connection::connection() : d(new data) {}

connection::connection(const connection& _rhs) : d{_rhs.d} {}
connection& connection::operator=(const connection& _rhs)
{
  d = _rhs.d;
  return *this;
}
connection::~connection() {}

connection connection::create_from_sqlite(void*, const value&)
{
  throw_exception("Creating a GQLite connection from a pointer is not possible anymore.");
}

connection connection::create_from_sqlite_file(const std::string& _filename, const value& _options)
{
  value_map options = _options.get_type() == value_type::map ? _options.to_map() : value_map();
  options["backend"] = "sqlite";
  return create_from_file(_filename, options);
}

connection connection::create_from_file(const std::string& _filename, const value& _options)
{
  connection c;
  c.d->api_context = gqlite_api_context_create();
  gqlite_value_wrapper options{c.d->api_context, _options};
  c.d->connection
    = gqlite_connection_create_from_file(c.d->api_context, _filename.c_str(), options.gqlite_value);
  check_errors(c.d->api_context);
  return c;
}

value connection::execute_oc_query(const std::string& _query, const value_map& _bindings)
{
  gqlite_value_wrapper bindings{d->api_context, _bindings};
  gqlite_value_wrapper val{
    d->api_context,
    gqlite_connection_query(d->api_context, d->connection, _query.c_str(), bindings.gqlite_value)};
  check_errors(d->api_context);
  return val.to_value();
}
