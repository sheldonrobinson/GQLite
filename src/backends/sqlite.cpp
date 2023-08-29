#include "sqlite.h"

#include <map>
#include <sqlite3.h>
#include <variant>

#include "sqlite_queries.h"

#include "../oc/algebra/abstract_node_visitor.h"
#include "../gqlite_p.h"
#include "../logging.h"

using namespace gqlite::backends;

namespace gqlite::backends
{
  struct sqlite_data
  {
    sqlite3* handle;
    std::unordered_map<int, std::string> id_to_label;
    std::unordered_map<std::string, int> label_to_id;

    void throwLastError();
    void graph_create(const std::string& _name);
    bool graph_has(const std::string& _name);
    bool table_has(const std::string& _name);

    value execute_sql(const std::string& _query, const std::map<int, value>& _bindings = {});
    uint64_t last_row_id();
    int id_for_label(const std::string& _string);
    std::string label_for_id(int _id);
  };
}

struct sqlite::data : public sqlite_data
{};

void sqlite_data::throwLastError()
{
  throw gqlite::exception(std::string(sqlite3_errmsg(handle)));
}

gqlite::value sqlite_data::execute_sql(const std::string& _query, const std::map<int, value>& _bindings)
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
          std::string js;
          if(value.get_type() == value_type::string)
          {
            js = value.to_string();
          } else {
            js = value.to_json();
          }
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

void sqlite_data::graph_create(const std::string& _name)
{
  execute_sql(sqlite_queries::graph_create(_name));
}

bool sqlite_data::graph_has(const std::string& _name)
{
  value r= execute_sql(sqlite_queries::graph_has(_name));
  std::vector<value> v = r.to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  v = v.begin()->to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  return v.begin()->to_bool();
}

bool sqlite_data::table_has(const std::string& _name)
{
  value r = execute_sql(sqlite_queries::table_has(), {{1, _name}});
  std::vector<value> v = r.to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result for table_has.");
  v = v.begin()->to_vector();
  check_condition(v.size() == 1, "Should have gotten only one column for table_has.");
  return v.begin()->to_bool();
}


uint64_t sqlite_data::last_row_id()
{
  return sqlite3_last_insert_rowid(handle);
}

int sqlite_data::id_for_label(const std::string& _string)
{
  auto it = label_to_id.find(_string);
  if(it == label_to_id.end())
  {
    value r = execute_sql(sqlite_queries::label_get_from_name(), {{1, _string}});
    std::vector<value> v = r.to_vector();
    if(v.size() == 0)
    {
      execute_sql(sqlite_queries::label_get_from_id(), {{1, _string}});
      int id = last_row_id();
      id_to_label[id] = _string;
      label_to_id[_string] = id;
      return id;
    } else {
      v = v.begin()->to_vector();
      check_condition(v.size() == 1, "Should have gotten only one column for label_get_from_id.");
      return v.begin()->to_integer();
    }
  } else {
    return it->second;
  }
}

std::string sqlite_data::label_for_id(int _id)
{
  auto it = id_to_label.find(_id);
  if(it == id_to_label.end())
  {
    value r = execute_sql(sqlite_queries::label_get_from_id(), {{1, _id}});
    std::vector<value> v = r.to_vector();
    if(v.size() == 0)
    {
      throw gqlite::exception("Internal error: unknown label id {}", _id);
    } else {
      v = v.begin()->to_vector();
      check_condition(v.size() == 1, "Should have gotten only one column for label_get_from_id.");
      return v.begin()->to_string();
    }
  } else {
    return it->second;
  }
}

namespace gqlite::backends::sqlite_oc_executor
{
  namespace algebra = gqlite::oc::algebra;

  struct node_ref
  {
    int id;
    value cache;
  };
  struct edge_ref
  {
    int id;
    value cache;
  };
  using node_ref_sp = std::shared_ptr<node_ref>;
  using edge_ref_sp = std::shared_ptr<edge_ref>;
  using element_ref = std::variant<node_ref_sp, edge_ref_sp>;
  struct element_ref_vector
  {
    std::vector<element_ref> refs;
    value cache;
  };
  using element_ref_vector_sp = std::shared_ptr<element_ref_vector>;
  using exec_value = std::variant<element_ref, element_ref_vector_sp, gqlite::value>;

  struct visitor : public gqlite::oc::algebra::abstract_node_visitor<exec_value>
  {
    sqlite_data* data;
    std::string graph_name = "default";
    std::unordered_map<std::string, exec_value> variables;
    gqlite::value get_value(const element_ref& _value)
    {
      struct value_getter
      {
        visitor* v;
        gqlite::value operator()(const node_ref_sp& _node_ref)
        {
          if(_node_ref->cache.get_type() == value_type::invalid)
          {
            throw gqlite::exception("wip: value getter for node ref");
          }
          return _node_ref->cache;
        }
        gqlite::value operator()(const edge_ref_sp& _edge_ref)
        {
          if(_edge_ref->cache.get_type() == value_type::invalid)
          {
            throw gqlite::exception("wip: value getter for edge ref");
          }
          return _edge_ref->cache;
        }
      };
      return std::visit(value_getter{this}, _value);
    }
    gqlite::value get_value(const exec_value& _value)
    {
      struct value_getter
      {
        visitor* v;
        gqlite::value operator()(const element_ref& _element)
        {
          return v->get_value(_element);
        }
        gqlite::value operator()(const element_ref_vector_sp& _vector)
        {
          if(_vector->cache.get_type() == value_type::invalid)
          {
            std::vector<gqlite::value> values;
            for(const element_ref& e_ref : _vector->refs)
            {
              values.push_back(v->get_value(e_ref));
            }
            _vector->cache = values;
          }
          return _vector->cache;
        }
        gqlite::value operator()(const gqlite::value& _value)
        {
          return _value;
        }
      };
      return std::visit(value_getter{this}, _value);
    }
    exec_value get_variable(const std::string& _name)
    {
      auto it = variables.find(_name);
      if(it == variables.end())
      {
        throw exception("Variable {} is not defined.", _name);
      } else {
        return it->second;
      }
    }
    exec_value visit(algebra::graph_node_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented graph_node");
    }
    exec_value visit(algebra::create_nodes_csp _node) override
    {
      for(algebra::graph_node_csp node : _node->get_nodes())
      {
        std::unordered_map<std::string, value> props;
        for(auto const& [k,v] : node->get_properties())
        {
          props[k] = get_value(start(v));
        }
        std::string json_properties = value(props).to_json();
        data->execute_sql(sqlite_queries::node_create(graph_name), {{1, json_properties}});
        int row_id = data->last_row_id();
        for(const std::string& label : node->get_labels())
        {
          data->execute_sql(sqlite_queries::node_map_to_label(graph_name), {{1, data->id_for_label(label)}, {2, row_id}});
        }
        variables[node->get_variable()] = std::make_shared<node_ref>(node_ref{
          row_id,
          gqlite::value{
          {
            {"type", gqlite::value("node")}, {"labels", gqlite::value(node->get_labels())}, {"id", gqlite::value(row_id)}, {"properties", props}            
          }}
        });
      }
      return gqlite::value();
    }
    exec_value visit(algebra::value_csp _node) override
    {
      return _node->get_value();
    }
    exec_value visit(algebra::return_statement_csp rs) override
    {
      std::vector<value> results;
      for(const std::string& rv : rs->get_variables())
      {
        results.push_back(get_value(get_variable(rv)));
      }
      return value(results);
    }
    exec_value visit(algebra::statements_csp _node) override
    {
      for(algebra::node_csp node : _node->get_nodes())
      {
        exec_value val = start(node);
        if(node->get_type() == algebra::node_type::return_statement)
        {
          return val;
        }
      }
      return exec_value();
    }
  };
}

sqlite::sqlite(void* _db) : d(new data)
{
  d->handle = reinterpret_cast<sqlite3*>(_db);
  if(not d->table_has("gqlite_labels"))
  {
    d->execute_sql(sqlite_queries::label_create_table());
  }
  if(not d->graph_has("default"))
  {
    d->graph_create("default");
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
  sqlite_oc_executor::visitor executor;
  executor.data = d;
  return executor.get_value(executor.start(_node));
}
