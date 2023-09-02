#include "sqlite.h"

#include <list>
#include <map>
#include <sqlite3.h>
#include <variant>

#include "sqlite_queries.h"

#include "../oc/algebra/abstract_node_visitor.h"
#include "../logging.h"

using namespace gqlite::backends;

namespace gqlite::backends
{
  struct sqlite_data
  {
    sqlite3* handle;
    std::unordered_map<int, std::string> id_to_label = {{0, std::string()}};
    std::unordered_map<std::string, int> label_to_id = {{std::string(), 0}};

    void throwLastError(const std::string& _query);
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

void sqlite_data::throwLastError(const std::string& _query)
{
  throw gqlite::exception("Error {}, while executing {}", sqlite3_errmsg(handle), _query);
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
      throwLastError(_query);
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
      throwLastError(_query);
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
      execute_sql(sqlite_queries::label_insert(), {{1, _string}});
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
  struct empty {};
  using node_ref_sp = std::shared_ptr<node_ref>;
  using edge_ref_sp = std::shared_ptr<edge_ref>;
  using element_ref = std::variant<node_ref_sp, edge_ref_sp>;
  struct element_ref_vector
  {
    std::vector<element_ref> refs;
    value cache;
  };
  using element_ref_vector_sp = std::shared_ptr<element_ref_vector>;
  using exec_value = std::variant<element_ref, element_ref_vector_sp, gqlite::value, empty>;

  /**
   * @internal
   * This visitor is used to execute the queries
   */
  struct visitor : public gqlite::oc::algebra::abstract_node_visitor<exec_value>
  {
    sqlite_data* data;
    std::string graph_name = "default";
    std::unordered_map<std::string, exec_value> variables;
    /**
     * @return a value representing the node/edge from @p _value
     */
    gqlite::value get_value(const element_ref& _value)
    {
      struct value_getter
      {
        visitor* v;
        gqlite::value operator()(const node_ref_sp& _node_ref)
        {
          if(_node_ref->cache.get_type() == value_type::invalid)
          {
            // Retrieve properties
            gqlite::value properties;
            {
              // Query
              std::vector<gqlite::value> properties_val_list_vector = v->data->execute_sql(sqlite_queries::node_get_properties(v->graph_name), {{1, _node_ref->id}}).to_vector();
              check_condition(properties_val_list_vector.size() == 1, "When getting a node, should have received only one node");
              std::vector<gqlite::value> properties_row = properties_val_list_vector.front().to_vector();
              check_condition(properties_row.size() == 1, "When getting a node, properties get should only have given one column");
              properties = gqlite::value::from_json(properties_row.front().to_string());
            }

            // Retrieve labels
            std::vector<gqlite::value> labels;
            {
              // Query
              std::vector<gqlite::value> labels_val_list_vector = v->data->execute_sql(sqlite_queries::node_get_labels(v->graph_name), {{1, _node_ref->id}}).to_vector();
              for(const gqlite::value& label_row_value : labels_val_list_vector)
              {
                std::vector<gqlite::value> label_row = label_row_value.to_vector();
                check_condition(label_row.size() == 1, "When getting a node, labels get should only have given one column");
                labels.push_back(v->data->label_for_id(label_row.front().to_integer()));
              }
            }
            // Generate the cache
            _node_ref->cache = gqlite::value{{
              {"type", gqlite::value("node")}, {"labels", gqlite::value(labels)}, {"id", gqlite::value(_node_ref->id)}, {"properties", gqlite::value(properties)}
            }};
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
    /**
     * @return a node ref or an exception if not a node ref
     */
    node_ref_sp get_node_ref(const exec_value& _value)
    {
      if(std::holds_alternative<element_ref>(_value))
      {
        element_ref er = std::get<element_ref>(_value);
        if(std::holds_alternative<node_ref_sp>(er))
        {
          return std::get<node_ref_sp>(er);
        }
      }
      throw gqlite::exception("Expected a reference to a node");
    }
    /**
     * @return a value representing the executed value from @p _value
     */
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
        gqlite::value operator()(const empty&)
        {
          return gqlite::value();
        }
      };
      return std::visit(value_getter{this}, _value);
    }
    /**
     * Get the variable stored in \ref variables or throw an exception.
     */
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
    value_map get_properties(const element_ref& _value)
    {
      return get_value(_value).to_map()["properties"].to_map();
    }
    value get_property(const value& _value, const std::vector<std::string>& _path)
    {
      value cval = _value;
      for(const std::string& pn : _path)
      {
        cval = cval.to_map()[pn];
      }
      return cval;
    }
    // Unused nodes
    exec_value visit(algebra::graph_node_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented graph_node");
    }
    exec_value visit(algebra::graph_edge_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented edge_node");
    }
    exec_value visit(algebra::named_expression_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented named_expression");
    }
    // Node/Edge creation
    bool has_node(algebra::graph_node_csp _node)
    {
      return not _node->get_variable().empty() and variables.find(_node->get_variable()) != variables.end();
    }
    gqlite::value get_properties(const std::unordered_map<std::string, algebra::node_csp>& _properties)
    {
      value_map props;
      for(auto const& [k,v] : _properties)
      {
        props[k] = get_value(start(v));
      }
      return props;
    }
    node_ref_sp create_node(algebra::graph_node_csp _node)
    {
      if(has_node(_node))
      {
        throw gqlite::exception("Variable {} is already bound.", _node->get_variable());
      }
      gqlite::value props = get_properties(_node->get_properties());
      std::string json_properties = props.to_json();
      data->execute_sql(sqlite_queries::node_create(graph_name), {{1, json_properties}});
      int row_id = data->last_row_id();
      for(const std::string& label : _node->get_labels())
      {
        data->execute_sql(sqlite_queries::node_map_to_label(graph_name), {{1, data->id_for_label(label)}, {2, row_id}});
      }
      node_ref_sp nr = std::make_shared<node_ref>(node_ref{
          row_id,
          gqlite::value{
          {
            {"type", gqlite::value("node")}, {"labels", gqlite::value(_node->get_labels())}, {"id", gqlite::value(row_id)}, {"properties", props}            
          }}
        });
      if(not _node->get_variable().empty())
      {
        variables[_node->get_variable()] = nr;
      }
      return nr;
    }
    void create_edge(const algebra::graph_edge_csp _edge)
    {
      node_ref_sp source = has_node(_edge->get_source()) ? get_node_ref(variables[_edge->get_source()->get_variable()]) : create_node(_edge->get_source());
      node_ref_sp destination = has_node(_edge->get_destination()) ? get_node_ref(variables[_edge->get_destination()->get_variable()]) : create_node(_edge->get_destination());
      int label_id = data->id_for_label(_edge->get_label());
      
      data->execute_sql(sqlite_queries::edge_create(graph_name), {{1, label_id}, {2, get_properties(_edge->get_properties()).to_json()}, {3, source->id}, {4, destination->id}});
    }
    exec_value visit(algebra::create_csp _node) override
    {
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        pattern.visit<void>([this](const algebra::graph_node_csp _node)
        {
          create_node(_node);
        },
        [this](const algebra::graph_edge_csp _edge)
        {
          create_edge(_edge);
        });
      }
      return empty{};
    }
    // Match
    exec_value visit(algebra::match_csp _node) override
    {
      int count = 0;
      int count_variables = 0;
      std::string sql_variables;
      std::string sql_tables;
      std::string sql_conditions;

      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        if(count != 0)
        {
          sql_variables += ", ";
          sql_tables += " JOIN ";
        }
        pattern.visit<void>([this, &sql_variables, &sql_tables, &sql_conditions, count, &count_variables](const algebra::graph_node_csp _node)
        {
          sql_variables += "tb" + std::to_string(count) + ".id";
          sql_tables += "gqlite_" + graph_name + "_nodes AS tb" + std::to_string(count);
          ++count_variables;
        },
        [this, &sql_variables, &sql_tables, &sql_conditions, count, &count_variables](const algebra::graph_edge_csp _edge)
        {
          sql_variables += "tb" + std::to_string(count) + ".left, ";
          sql_variables += "tb" + std::to_string(count) + ".id, ";
          sql_variables += "tb" + std::to_string(count) + ".right";
          sql_tables += "gqlite_" + graph_name + "_edges AS tb" + std::to_string(count);
          count_variables += 3;
        });
        ++count;
      }
      if(not sql_conditions.empty())
      {
        sql_conditions = (count == 1 ? " WHERE " : " ON ") + sql_conditions;
      }
      gqlite::value r = data->execute_sql("SELECT " + sql_variables + " FROM " + sql_tables + sql_conditions);
      std::vector<element_ref_vector_sp> ervs;
      ervs.reserve(count_variables);
      for(int i = 0; i < count_variables; ++i)
      {
        ervs.push_back(std::make_shared<element_ref_vector>());
      }

      for(const gqlite::value& row_value : r.to_vector())
      {
        int idx = 0;
        std::vector<gqlite::value> row = row_value.to_vector();
        check_condition(row.size() == count_variables, "Wrong number of column return by SQL Query.");

        for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
        {
          pattern.visit<void>([&row, &ervs, &idx](const algebra::graph_node_csp _node)
          {
            ervs[idx]->refs.push_back(std::make_shared<node_ref>(row[idx].to_integer()));
            ++idx;
          },
          [&row, &ervs, &idx](const algebra::graph_edge_csp _edge)
          {
            ervs[idx]->refs.push_back(std::make_shared<node_ref>(row[idx].to_integer()));
            ++idx;
            ervs[idx]->refs.push_back(std::make_shared<edge_ref>(row[idx].to_integer()));
            ++idx;
            ervs[idx]->refs.push_back(std::make_shared<node_ref>(row[idx].to_integer()));
            ++idx;
          });
        }
      }
      int idx = 0;
      std::function<void(const std::string&)> ervs_to_variables = [&idx, &ervs, this](const std::string& _varname)
      {
        if(not _varname.empty())
        {
          variables[_varname] = ervs[idx];
        }
        ++idx;
      };
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        pattern.visit<void>([ervs_to_variables](const algebra::graph_node_csp _node)
          {
            ervs_to_variables(_node->get_variable());
          },
          [ervs_to_variables](const algebra::graph_edge_csp _edge)
          {
            ervs_to_variables(_edge->get_source()->get_variable());
            ervs_to_variables(_edge->get_variable());
            ervs_to_variables(_edge->get_destination()->get_variable());
          });
      }
      return empty{};
    }
    exec_value visit(algebra::value_csp _node) override
    {
      return _node->get_value();
    }
    exec_value visit(algebra::variable_csp _node) override
    {
      return get_variable(_node->get_identifier());
    }
    exec_value visit(algebra::member_access_csp _node) override
    {
      exec_value value = start(_node->get_left());

      struct member_access
      {
        visitor* self;
        algebra::member_access_csp node;
        gqlite::value operator()(const element_ref& _v)
        {
          return self->get_property(self->get_properties(_v), node->get_path());
        }
        gqlite::value operator()(const element_ref_vector_sp& _v)
        {
          std::vector<gqlite::value> values;
          for(const element_ref& v : _v->refs)
          {
            values.push_back(operator()(_v));
          }
          return gqlite::value(values);
        }
        gqlite::value operator()(const gqlite::value& _v, bool _allow_array = true)
        {
          switch(_v.get_type())
          {
            case gqlite::value_type::map:
            {
              return self->get_property(_v.to_map(), node->get_path());
            }
            case gqlite::value_type::vector:
            {
              if(_allow_array)
              {
                std::vector<gqlite::value> values;
                for(const gqlite::value& v : _v.to_vector())
                {
                  values.push_back(operator()(_v, false));
                }
                return values;
              }
              [[fallthrough]];
            }
            default:
              throw gqlite::exception("Invalid value type, expected a map, got {}.", _v.to_json());
          }
        }
        gqlite::value operator()(const empty&)
        {
          return gqlite::value();
        }
      };
      return std::visit(member_access{this, _node}, value);
    }
    exec_value visit(algebra::return_statement_csp rs) override
    {
      std::vector<value> labels;
      std::vector<std::vector<value>> results_columns;
      std::size_t rows = 1;
      for(const algebra::named_expression_csp& rv : rs->get_expressions())
      {
        labels.push_back(rv->get_name());
        value column_value = get_value(accept(rv->get_expression()));
        std::vector<value> column = (column_value.get_type() == value_type::vector) ? column_value.to_vector() : std::vector<value>{column_value};
        results_columns.push_back(column);
        rows = std::max(rows, column.size());
      }
      std::vector<value> results_rows;
      results_rows.push_back(labels);
      for(int i = 0; i < rows; ++i)
      {
        std::vector<value> row;
        for(const std::vector<value>& col : results_columns)
        {
          if(i < col.size())
          {
            row.push_back(col[i]);
          } else {
            row.push_back(value());
          }
        }
        results_rows.push_back(row);
      }
      return value(results_rows);
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
      return gqlite::value();
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

gqlite::value sqlite::execute_oc_query(oc::algebra::node_csp _node, const value_map& _bindings)
{
  sqlite_oc_executor::visitor executor;
  executor.data = d;
  return executor.get_value(executor.start(_node));
}

gqlite::value sqlite::get_debug_stats() const
{
  gqlite::value result = d->execute_sql(sqlite_queries::get_debug_stats("default"));
  std::vector<gqlite::value> rows = result.to_vector();
  check_condition(rows.size() == 7, "Invalid number of debug stats.");
  value_map stats;
  stats["nodes_count"] = rows[0].to_vector()[0].to_integer();
  stats["edges_count"] = rows[1].to_vector()[0].to_integer();
  stats["labels_assignment_count"] = rows[2].to_vector()[0].to_integer();
  stats["properties_count"] = rows[3].to_vector()[0].to_integer() + rows[4].to_vector()[0].to_integer();
  stats["labels_count"] = rows[5].to_vector()[0].to_integer();
  stats["labels_assignment_nodes_count"] = rows[6].to_vector()[0].to_integer();
  return stats;
}
