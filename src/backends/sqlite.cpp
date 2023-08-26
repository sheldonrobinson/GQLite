#include "sqlite.h"

#include <sqlite3.h>

#include "../logging.h"

using namespace gqlite::backends;

struct sqlite::data
{
  sqlite3* handle;
  void create_graph(const std::string& _name);
  bool has_graph(const std::string& _name);
  void execute_sql(const std::string& _query);
};

void sqlite::data::create_graph(const std::string& _name)
{
  execute_sql("CREATE TABLE " + _name + "_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL)");
  execute_sql("CREATE TABLE " + _name + "_edges(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL, left INTEGER, right INTEGER, FOREIGN KEY(left) REFERENCES " + _name + "_nodes(id), FOREIGN KEY(right) REFERENCES " + _name + "_nodes(id))");
  execute_sql("CREATE TABLE " + _name + "_labels(label TEXT NOT NULL, node_id INTEGER)");
}

bool sqlite::data::has_graph(const std::string& _name)
{
  execute_sql("SELECT count(*) FROM sqlite_master WHERE type='table' AND (name='" + _name + "_nodes' or name='" + _name + "_edges' or name='" + _name + "_labels')");
  gqlite_fatal("wip");
}

sqlite::sqlite(void* _db) : d(new data)
{
  d->handle = reinterpret_cast<sqlite3*>(_db);
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

gqlite::result sqlite::execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, std::any>& _variant)
{
  return gqlite::result::from_error("sqlite not implemented");
}
