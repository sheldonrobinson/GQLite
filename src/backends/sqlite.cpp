#include "sqlite.h"

#include <map>
#include <sqlite3.h>

#include "../logging.h"

using namespace gqlite::backends;

struct sqlite::data
{
  sqlite3* handle;
  void throwLastError();
  void create_graph(const std::string& _name);
  bool has_graph(const std::string& _name);
  value execute_sql(const std::string& _query, const std::map<int, value>& _bindings = {});
};

void sqlite::data::throwLastError()
{
  throw gqlite::exception(std::string(sqlite3_errmsg(handle)));
}

gqlite::value sqlite::data::execute_sql(const std::string& _query, const std::map<int, value>& _bindings)
{
  sqlite3_stmt* ps;
  const char* ptr = _query.data();
  const char* ptr_end = _query.data() + _query.size();
  std::vector<value> q_rs;
  while(ptr != ptr_end)
  {
    sqlite3_stmt* ps;
    if(sqlite3_prepare_v2(handle, ptr, ptr_end - ptr, &ps, &ptr) != SQLITE_OK)
    {
      sqlite3_finalize(ps);
      throwLastError();
    }
    for(const auto& [key, value] : _bindings)
    {
      switch(value.get_type())
      {
        case value_type::invalid:
          sqlite3_bind_null(ps, key);
          break;
        case value_type::number:
          sqlite3_bind_int(ps, key, value.to_double());
          break;
        default:
        {
          std::string js = value.to_json();
          char* arr = new char[js.size()];
          std::copy(js.begin(), js.end(), arr);
          sqlite3_bind_text(ps, key, arr, js.size(), [](void* _ptr) { delete[] static_cast<char*>(_ptr); });
          break;
        }
      }
    }
    std::vector<value> rows;
    int code = sqlite3_step(ps);
    while(code == SQLITE_ROW)
    {
      std::vector<value> row;
      for(int i = 0; i < sqlite3_data_count(ps); ++i)
      {
        switch(sqlite3_column_type(ps, i))
        {
          case SQLITE_INTEGER:
            row.push_back(sqlite3_column_int(ps, i));
            break;
          case SQLITE_FLOAT:
            row.push_back(sqlite3_column_double(ps, i));
            break;
          case SQLITE_BLOB:
            sqlite3_finalize(ps);
            throw gqlite::exception("Blobs are not supported.");
          case SQLITE_NULL:
            row.push_back(value());
            break;
          case SQLITE3_TEXT:
            row.push_back(std::string(reinterpret_cast<const char*>(sqlite3_column_text(ps, i))));
            break;
        }
      }
      rows.push_back(row);
      code = sqlite3_step(ps);
    }
    // Finalize the execution of the statement
    sqlite3_finalize(ps);
    switch(code)
    {
    case SQLITE_DONE:
      q_rs.push_back(rows);
      break;
    default:
      throwLastError();
    }
  }
  if(q_rs.size() == 1)
  {
    return q_rs[0];
  } else {
    return q_rs;
  }
}

void sqlite::data::create_graph(const std::string& _name)
{
  execute_sql("CREATE TABLE gqlite_" + _name + "_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL)");
  execute_sql("CREATE TABLE gqlite_" + _name + "_edges(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL, left INTEGER, right INTEGER, FOREIGN KEY(left) REFERENCES " + _name + "_nodes(id), FOREIGN KEY(right) REFERENCES " + _name + "_nodes(id))");
  execute_sql("CREATE TABLE gqlite_" + _name + "_labels(label TEXT NOT NULL, node_id INTEGER)");
}

bool sqlite::data::has_graph(const std::string& _name)
{
  value r= execute_sql("SELECT count(*) FROM sqlite_master WHERE type='table' AND (name='gqlite_" + _name + "_nodes' or name='gqlite_" + _name + "_edges' or name='gqlite_" + _name + "_labels')");
  std::vector<value> v = r.to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  v = v.begin()->to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  return v.begin()->to_bool();
}

sqlite::sqlite(void* _db) : d(new data)
{
  d->handle = reinterpret_cast<sqlite3*>(_db);
  if(not d->has_graph("default"))
  {
    d->create_graph("default");
  }
}

sqlite::~sqlite()
{}

sqlite* sqlite::from_file(const std::string& _filename)
{
  sqlite3* handle;

  if(sqlite3_open(_filename.c_str(), &handle) != SQLITE_OK)
  {
    gqlite_error(sqlite3_errmsg(handle));
    sqlite3_close(handle);
    return nullptr;
  }
  return new sqlite(handle);
}

gqlite::value sqlite::execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, value>& _bindings)
{
  throw gqlite::exception("sqlite not implemented");
}
