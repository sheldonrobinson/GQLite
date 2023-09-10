#include "sqlite.h"

#include <list>
#include <map>
#include <sqlite3.h>
#include <variant>

#include "functions.h"
#include "sqlite_queries.h"

#include "../oc/algebra/abstract_node_visitor.h"
#include "../logging.h"
#include "../string.h"

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
  value_vector q_rs;
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
    value_vector rows;
    int code = sqlite3_step(ps);
    while(code == SQLITE_ROW)
    {
      value_vector row;
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
  value_vector v = r.to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  v = v.begin()->to_vector();
  check_condition(v.size() == 1, "Should have gotten only one result");
  return v.begin()->to_bool();
}

bool sqlite_data::table_has(const std::string& _name)
{
  value r = execute_sql(sqlite_queries::table_has(), {{1, _name}});
  value_vector v = r.to_vector();
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
    value_vector v = r.to_vector();
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
    value_vector v = r.to_vector();
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

  struct execution_context
  {
    sqlite_data* data;
    std::string graph_name = "default";
  };

  struct match_context
  {
    execution_context* ec;
    int count = 0;
    std::string sql_variables;
    std::string sql_tables;
    std::string sql_conditions;
    std::map<int, value> bindings;
    std::vector<std::pair<std::string, std::string>> edge_sql_var_to_oc_var;
    int label_count = 0;
    struct var_info
    {
      std::string sql_var;
      std::size_t sql_column;
      bool is_node;
    };
    std::map<std::string, var_info> oc_var_to_sql_var;
    /**
     * Generate the SQL join operation needed for matchin labels
     */
    std::string generate_labels_match(const std::vector<std::string>& _labels, const std::string& _node_variable)
    {
      std::string cond;
      for(const std::string& label : _labels)
      {
        sql_tables += format_string(" JOIN gqlite_{}_labels AS tb_lab{}", ec->graph_name, label_count);
        if(not cond.empty()) cond += " AND ";
        cond += format_string(" {} = tb_lab{}.node_id AND ?{} = tb_lab{}.label ",
                      _node_variable, label_count, to_string_fixed_width(bindings.size() + 1, 3), label_count);
        bindings[bindings.size() + 1] = ec->data->id_for_label(label);
        ++label_count;
      }
      return cond;
    }
  };

  struct filter_visitor : public gqlite::oc::algebra::abstract_node_visitor<std::string>
  {
    match_context* mc;
    std::string get_sql_variable(const std::string& _oc_variable)
    {
      auto it = mc->oc_var_to_sql_var.find(_oc_variable);
      if(it == mc->oc_var_to_sql_var.end())
      {
        throw gqlite::exception("Unknown variable '{}'", _oc_variable);
      }
      return it->second.sql_var;
    }
    std::string visit(algebra::graph_node_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented graph_node");
    }
    std::string visit(algebra::graph_edge_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented edge_node");
    }
    std::string visit(algebra::named_expression_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented named_expression");
    }
    std::string visit(algebra::create_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented create");
    }
    std::string visit(algebra::match_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented match");
    }
    std::string visit(algebra::statements_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented statements");
    }
    std::string visit(algebra::return_statement_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented return");
    }
    std::string visit(algebra::has_labels_csp _node) override
    {
      return mc->generate_labels_match(_node->get_labels(), get_sql_variable(_node->get_left()));
    }
    std::string visit(algebra::value_csp _node) override
    {
      int idx = mc->bindings.size() + 1;
      mc->bindings[idx] = _node->get_value();
      return "?" + to_string_fixed_width(idx, 3);
    }
    std::string visit(algebra::map_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented map");
    }
    std::string visit(algebra::array_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented array");
    }
    std::string visit(algebra::variable_csp _node) override
    {
      return get_sql_variable(_node->get_identifier());
    }
    std::string visit(algebra::member_access_csp _node) override
    {
      return format_string("json_extract({}, '$.{}')", start(_node->get_left()), string::join(_node->get_path(), "."));
    }
    std::string visit(algebra::function_call_csp _node) override
    {
      throw gqlite::exception("sqlite not implemented function_call");
    }
#define FILTER_VISITOR_BINARY_OP(_AL_, _OP_)                                                          \
    std::string visit(algebra::_AL_ ## _csp _node) override                                           \
    {                                                                                                 \
      return format_string("({} " _OP_ " {})", start(_node->get_left()), start(_node->get_right()));  \
    }
    FILTER_VISITOR_BINARY_OP(logical_and, "&&")
    FILTER_VISITOR_BINARY_OP(logical_or, "||")
    FILTER_VISITOR_BINARY_OP(relational_equal, "=")
    FILTER_VISITOR_BINARY_OP(relational_different, "!=")
    FILTER_VISITOR_BINARY_OP(relational_inferior, "<")
    FILTER_VISITOR_BINARY_OP(relational_superior, ">")
    FILTER_VISITOR_BINARY_OP(relational_inferior_equal, "<")
    FILTER_VISITOR_BINARY_OP(relational_superior_equal, ">")
    FILTER_VISITOR_BINARY_OP(relational_in, "IN")
    FILTER_VISITOR_BINARY_OP(relational_not_in, "NOT IN")
    FILTER_VISITOR_BINARY_OP(addition, "+")
    FILTER_VISITOR_BINARY_OP(substraction, "-")
    FILTER_VISITOR_BINARY_OP(multiplication, "*")
    FILTER_VISITOR_BINARY_OP(division, "/")
#define FILTER_VISITOR_UNARY_OP(_AL_, _OP_)                             \
    std::string visit(algebra::_AL_ ## _csp _node) override             \
    {                                                                   \
      return format_string("(" _OP_ " {})", start(_node->get_value())); \
    }
    FILTER_VISITOR_UNARY_OP(logical_negation, "!")
    FILTER_VISITOR_UNARY_OP(negation, "-")
  };

  /**
   * @internal
   * This visitor is used to execute the queries
   */
  struct visitor : public gqlite::oc::algebra::abstract_node_visitor<exec_value>
  {
    execution_context ec;
    std::unordered_map<std::string, exec_value> variables;
    /**
     * @return a value representing the node/edge from @p _value
     */
    gqlite::value get_value(const element_ref& _value)
    {
      struct value_getter
      {
        execution_context* v;
        gqlite::value operator()(const node_ref_sp& _node_ref)
        {
          if(_node_ref->cache.get_type() == value_type::invalid)
          {
            // Retrieve properties
            gqlite::value properties;
            {
              // Query
              value_vector properties_val_list_vector = v->data->execute_sql(sqlite_queries::node_get_properties(v->graph_name), {{1, _node_ref->id}}).to_vector();
              check_condition(properties_val_list_vector.size() == 1, "When getting a node, should have received only one node");
              value_vector properties_row = properties_val_list_vector.front().to_vector();
              check_condition(properties_row.size() == 1, "When getting a node, properties get should only have given one column");
              properties = gqlite::value::from_json(properties_row.front().to_string());
            }

            // Retrieve labels
            value_vector labels;
            {
              // Query
              value_vector labels_val_list_vector = v->data->execute_sql(sqlite_queries::node_get_labels(v->graph_name), {{1, _node_ref->id}}).to_vector();
              for(const gqlite::value& label_row_value : labels_val_list_vector)
              {
                value_vector label_row = label_row_value.to_vector();
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
            // Retrieve label/properties
            std::string label;
            gqlite::value properties;
            {
              // Query
              value_vector properties_val_list_vector = v->data->execute_sql(sqlite_queries::edge_get_label_properties(v->graph_name), {{1, _edge_ref->id}}).to_vector();
              check_condition(properties_val_list_vector.size() == 1, "When getting an edge, should have received only one edge");
              value_vector properties_row = properties_val_list_vector.front().to_vector();
              check_condition(properties_row.size() == 2, "When getting an edge, properties get should only have given two column");
              label = v->data->label_for_id(properties_row[0].to_integer());
              properties = gqlite::value::from_json(properties_row[1].to_string());
            }
            // Generate the cache
            _edge_ref->cache = gqlite::value{{
              {"type", gqlite::value("edge")}, {"label", gqlite::value(label)}, {"id", gqlite::value(_edge_ref->id)}, {"properties", gqlite::value(properties)}
            }};
          }
          return _edge_ref->cache;
        }
      };
      return std::visit(value_getter{&ec}, _value);
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
      } else if(std::holds_alternative<element_ref_vector_sp>(_value))
      {
        element_ref_vector_sp erv = std::get<element_ref_vector_sp>(_value);
        if(erv->refs.size() == 1)
        {
          element_ref er = erv->refs.front();
          if(std::holds_alternative<node_ref_sp>(er))
          {
            return std::get<node_ref_sp>(er);
          }
        } else {
          throw gqlite::exception("Expected a single item.");
        }
      }
      throw gqlite::exception("Expected a reference to a node.");
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
            value_vector values;
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
    bool has_variable(const std::string& _variable)
    {
      return not _variable.empty() and variables.find(_variable) != variables.end();
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
    /**
     * Generate a filter expression for a map
     */
    std::string generate_filter(const algebra::map_csp& _map, const std::string& _path, filter_visitor* _filter_visitor)
    {
      std::string r;
      for(const auto& [k, v] : _map->get_map())
      {
        std::string path = _path + "." + k;
        if(v->get_type() == algebra::node_type::map)
        {
          r += generate_filter(std::static_pointer_cast<const algebra::map>(_map), path, _filter_visitor);
        } else {
          r += format_string(" AND json_extract({}') = {} ", path, _filter_visitor->start(v));
        }
      }
      return r;
    }
    // Unused nodes
#define UNUSED_NODES(_AL_)                                                          \
    exec_value visit(algebra::_AL_ ## _csp _node) override                          \
    {                                                                               \
      throw gqlite::exception("sqlite execution visitor not implemented " # _AL_);  \
    }
    UNUSED_NODES(logical_and)
    UNUSED_NODES(logical_or)
    UNUSED_NODES(relational_equal)
    UNUSED_NODES(relational_different)
    UNUSED_NODES(relational_inferior)
    UNUSED_NODES(relational_superior)
    UNUSED_NODES(relational_inferior_equal)
    UNUSED_NODES(relational_superior_equal)
    UNUSED_NODES(relational_in)
    UNUSED_NODES(relational_not_in)
    UNUSED_NODES(addition)
    UNUSED_NODES(substraction)
    UNUSED_NODES(multiplication)
    UNUSED_NODES(division)
    UNUSED_NODES(logical_negation)
    UNUSED_NODES(negation)
    UNUSED_NODES(has_labels)
    UNUSED_NODES(graph_node)
    UNUSED_NODES(graph_edge)
    UNUSED_NODES(named_expression)
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
      if(has_variable(_node->get_variable()))
      {
        throw exception("Variable {} is already bound.", _node->get_variable());
      }
      value props;
      if(_node->get_properties())
      {
        props = get_properties(_node->get_properties()->get_map());
      } else {
        props = value_map();
      }
      std::string json_properties = props.to_json();
      ec.data->execute_sql(sqlite_queries::node_create(ec.graph_name), {{1, json_properties}});
      int row_id = ec.data->last_row_id();
      for(const std::string& label : _node->get_labels())
      {
        ec.data->execute_sql(sqlite_queries::node_map_to_label(ec.graph_name), {{1, ec.data->id_for_label(label)}, {2, row_id}});
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
      if(has_variable(_edge->get_variable()))
      {
        throw exception("Variable {} is already bound.", _edge->get_variable());
      }
      node_ref_sp source = has_node(_edge->get_source()) ? get_node_ref(variables[_edge->get_source()->get_variable()]) : create_node(_edge->get_source());
      node_ref_sp destination = has_node(_edge->get_destination()) ? get_node_ref(variables[_edge->get_destination()->get_variable()]) : create_node(_edge->get_destination());
      std::string label = _edge->get_labels().empty() ? std::string() : _edge->get_labels().front();
      int label_id = ec.data->id_for_label(label);
      
      value props;
      if(_edge->get_properties())
      {
        props = get_properties(_edge->get_properties()->get_map());
      } else {
        props = value_map();
      }
      ec.data->execute_sql(sqlite_queries::edge_create(ec.graph_name), {{1, label_id}, {2, props.to_json()}, {3, source->id}, {4, destination->id}});
      int row_id = ec.data->last_row_id();
      edge_ref_sp nr = std::make_shared<edge_ref>(edge_ref{
          row_id,
          value{
            {{"type", value("edge")}, {"label", value(label)}, {"id", value(row_id)}, {"properties", props}}
          }
        });
      if(not _edge->get_variable().empty())
      {
        variables[_edge->get_variable()] = nr;
      }
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
    void generate_var_match(match_context* _mc, const std::string& _oc_var, const std::string& _sql_var, bool _is_node)
    {
      if(not _oc_var.empty())
      {
        auto it = _mc->oc_var_to_sql_var.find(_oc_var);
        if(it == _mc->oc_var_to_sql_var.end())
        {
          if(has_variable(_oc_var))
          {
            throw exception("{} is already defined.", _oc_var);
          }
          if(not _mc->sql_variables.empty()) _mc->sql_variables += ", ";
          _mc->sql_variables += _sql_var;
          _mc->oc_var_to_sql_var[_oc_var] = {_sql_var, _mc->oc_var_to_sql_var.size(), _is_node};
        } else {
          if(it->second.is_node != _is_node) throw exception("{} is redefined as a variable of a different type", _oc_var);
          _mc->sql_conditions += format_string(" AND {} = {} ", _sql_var, it->second.sql_var);
        }
      }
    }
    exec_value visit(algebra::match_csp _node) override
    {
      match_context mc;
      mc.ec = &ec;
      filter_visitor fil_vis;
      fil_vis.mc = &mc;
      // 1) Go through the patterns
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        if(mc.count != 0)
        {
          mc.sql_tables += " JOIN ";
        }
        pattern.visit<void>([this, &mc, &fil_vis](const algebra::graph_node_csp _node)
        {
          std::string count_s = std::to_string(mc.count);
          std::string sql_var = format_string("tb{}.id", count_s);
          mc.sql_tables += format_string("gqlite_{}_nodes AS tb{}", ec.graph_name, count_s);
          mc.sql_conditions += mc.generate_labels_match(_node->get_labels(), format_string("tb{}.id", count_s));
          if(_node->get_properties())
          {
            mc.sql_conditions += generate_filter(_node->get_properties(), "tb" + count_s + ".properties, '$", &fil_vis);
          }
          generate_var_match(&mc, _node->get_variable(), sql_var, true);
        },
        [this, &mc, &fil_vis](const algebra::graph_edge_csp _edge)
        {
          std::string count_s = std::to_string(mc.count);
          std::string sql_source_var = format_string("tb{}.left", count_s);
          std::string sql_edge_var = format_string("tb{}.id", count_s);
          std::string sql_destination_var = format_string("tb{}.right", count_s);
          if(_edge->get_directivity() == algebra::edge_directivity::undirected)
          {
            mc.sql_tables += format_string("gqlite_{}_edges_undirected AS tb{}", ec.graph_name, count_s);
          } else {
            mc.sql_tables += format_string("gqlite_{}_edges AS tb{}", ec.graph_name, count_s);
          }
          mc.sql_conditions += mc.generate_labels_match(_edge->get_source()->get_labels(), "tb" + count_s + ".left");
          mc.sql_conditions += mc.generate_labels_match(_edge->get_destination()->get_labels(), "tb" + count_s + ".right");
          if(not _edge->get_labels().empty())
          {
            mc.sql_conditions += " AND (FALSE ";
            for(const std::string& label : _edge->get_labels())
            {
              mc.sql_conditions += format_string(" OR tb{}.label = {}", count_s, ec.data->id_for_label(label));
            }
            mc.sql_conditions += ")";
          }
          if(_edge->get_properties())
          {
            mc.sql_conditions += generate_filter(_edge->get_properties(), format_string("tb{}.properties, '$", count_s), &fil_vis);
          }
          if(_edge->get_source()->get_properties())
          {
            throw exception("Node properties not implemented, need join.");
            // sql_conditions += generate_filter(_edge->get_source()->get_properties(), "tb" + count_s + ".properties, '$", &fil_vis);
          }
          if(_edge->get_destination()->get_properties())
          {
            throw exception("Node properties not implemented, need join.");
            // sql_conditions += generate_filter(_edge->get_destination()->get_properties(), "tb" + count_s + ".properties, '$", &fil_vis);
          }
          // Ensure identical oc variablle are joined in SQL
          generate_var_match(&mc, _edge->get_source()->get_variable(), sql_source_var, true);
          generate_var_match(&mc, _edge->get_variable(), sql_edge_var, false);
          generate_var_match(&mc, _edge->get_destination()->get_variable(), sql_destination_var, true);
          // ensure edge isomorphism, i.e., if the variable names are different, the edges must be different
          for(const std::pair<std::string, std::string>& sql_var_to_oc_var : mc.edge_sql_var_to_oc_var)
          {
            if(sql_var_to_oc_var.second != _edge->get_variable() and not _edge->get_variable().empty())
            {
              mc.sql_conditions += format_string(" AND {} != {}", sql_edge_var, sql_var_to_oc_var.first);
            }
          }
          mc.edge_sql_var_to_oc_var.push_back({sql_edge_var, _edge->get_variable()});
          // Increase variable counter
        });
        ++mc.count;
      }
      // 2) Handle where
      if(_node->get_where())
      {
        mc.sql_conditions += " AND " + fil_vis.start(_node->get_where());
      }
      // 3) Assemble SQL query for execution
      if(not mc.sql_conditions.empty())
      {
        mc.sql_conditions = (mc.count == 1 ? " WHERE TRUE " : " ON TRUE ") + mc.sql_conditions;
      }
      std::string sql_query = "SELECT DISTINCT " + mc.sql_variables + " FROM " + mc.sql_tables + mc.sql_conditions;
      // std::cout << sql_query << std::endl;
      gqlite::value r = ec.data->execute_sql(sql_query, mc.bindings);

      // 4) Store the results.
      // 4a) initialise ervs, which will contain the results
      std::vector<element_ref_vector_sp> ervs;
      ervs.reserve(mc.oc_var_to_sql_var.size());
      for(int i = 0; i < mc.oc_var_to_sql_var.size(); ++i)
      {
        ervs.push_back(std::make_shared<element_ref_vector>());
      }

      // 4b) loop through the SQL results, which are rows, while ervs are columns 
      for(const gqlite::value& row_value : r.to_vector())
      {
        value_vector row = row_value.to_vector();
        check_condition(row.size() == mc.oc_var_to_sql_var.size(), "Wrong number of column return by SQL Query.");
        for(const auto& [k, vi] : mc.oc_var_to_sql_var)
        {
          int id = row[vi.sql_column].to_integer();
          if(vi.is_node)
          {
            ervs[vi.sql_column]->refs.push_back(std::make_shared<node_ref>(id));
          } else {
            ervs[vi.sql_column]->refs.push_back(std::make_shared<edge_ref>(id));
          }
        }
      }
      // 4c) assigning to variables
      for(const auto& [k, vi] : mc.oc_var_to_sql_var)
      {
        check_condition(not has_variable(k), format_string("{} is already defined.", k));
        variables[k] = ervs[vi.sql_column];
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
    exec_value visit(algebra::array_csp _node) override
    {
      std::vector<value> values;
      for(const algebra::node_csp& v : _node->get_array())
      {
        values.push_back(get_value(start(v)));
      }
      return values;
    }
    exec_value visit(algebra::map_csp _node) override
    {
      return get_properties(_node->get_map());
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
          value_vector values;
          for(const element_ref& v : _v->refs)
          {
            values.push_back(operator()(v));
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
                value_vector values;
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
    exec_value visit(algebra::function_call_csp _node) override
    {
      std::vector<value> args;
      for(const algebra::node_csp& arg : _node->get_arguments())
      {
        args.push_back(get_value(start(arg)));
      }
      return functions::call(_node->get_name(), args);
    }
    exec_value visit(algebra::return_statement_csp rs) override
    {
      value_vector labels;
      std::vector<value_vector> results_columns;
      std::size_t rows = 0;
      for(const algebra::named_expression_csp& rv : rs->get_expressions())
      {
        labels.push_back(rv->get_name());
        value column_value = get_value(accept(rv->get_expression()));
        value_vector column = (column_value.get_type() == value_type::vector) ? column_value.to_vector() : value_vector{column_value};
        results_columns.push_back(column);
        rows = std::max(rows, column.size());
      }
      value_vector results_rows;
      results_rows.push_back(labels);
      for(int i = 0; i < rows; ++i)
      {
        value_vector row;
        for(const value_vector& col : results_columns)
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
  d->execute_sql("BEGIN");
  try
  {
    sqlite_oc_executor::visitor executor;
    executor.ec.data = d;
    value val = executor.get_value(executor.start(_node));
    d->execute_sql("COMMIT");
    return val;
  } catch(const exception& _ex)
  {
    d->execute_sql("ROLLBACK");
    throw _ex;
  }
}

gqlite::value sqlite::get_debug_stats() const
{
  gqlite::value result = d->execute_sql(sqlite_queries::get_debug_stats("default"));
  value_vector rows = result.to_vector();
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
