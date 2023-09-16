#include "sqlite.h"

#include <list>
#include <map>
#include <sqlite3.h>
#include <variant>

#include "functions.h"
#include "sqlite_queries.h"

#include "../oc/algebra/default_node_visitor.h"
#include "../logging.h"
#include "../string.h"
#include "../table.h"

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
#if 0
  std::cout << "Executing query: '" << _query << "' with bindings:" << std::endl;
  for(auto const& [k,v] : _bindings)
  {
    std::cout << " [" << k << "] = " << v.to_json() << std::endl;
  }
#endif
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
        case value_type::integer:
          sqlite3_bind_int(ps, key, value.to_integer());
          break;
        case value_type::number:
          sqlite3_bind_double(ps, key, value.to_double());
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
  errors::check_condition(v.size() == 1, "Should have gotten only one result");
  v = v.begin()->to_vector();
  errors::check_condition(v.size() == 1, "Should have gotten only one result");
  return v.begin()->to_bool();
}

bool sqlite_data::table_has(const std::string& _name)
{
  value r = execute_sql(sqlite_queries::table_has(), {{1, _name}});
  value_vector v = r.to_vector();
  errors::check_condition(v.size() == 1, "Should have gotten only one result for table_has.");
  v = v.begin()->to_vector();
  errors::check_condition(v.size() == 1, "Should have gotten only one column for table_has.");
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
      errors::check_condition(v.size() == 1, "Should have gotten only one column for label_get_from_id.");
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
      errors::check_condition(v.size() == 1, "Should have gotten only one column for label_get_from_id.");
      return v.begin()->to_string();
    }
  } else {
    return it->second;
  }
}

namespace gqlite::backends::sqlite_oc_executor
{
  namespace algebra = gqlite::oc::algebra;

  /**
   * @internal
   * Represent a reference to a node
   */
  struct node_ref
  {
    int id; ///< database id for the node
    value cache; ///< cache of the labels/properties
  };

  /**
   * @internal
   * Represent a relationship between nodes
   */
  struct edge_ref
  {
    int id; ///< database id for the edge
    value cache; ///< cache of the label/properties
  };
  struct empty {};
  using node_ref_sp = std::shared_ptr<node_ref>;
  using edge_ref_sp = std::shared_ptr<edge_ref>;
  using exec_value = std::variant<empty, node_ref_sp, edge_ref_sp, gqlite::value>;
  using exec_value_table = table<exec_value>;

  /**
   * Global context for the execution of a query.
   */
  struct execution_context
  {
    sqlite_data* data;
    std::string graph_name = "default";

    /**
     * @return a value representing the executed value from @p _value
     */
    gqlite::value get_value(const exec_value& _value)
    {
      struct value_getter
      {
        execution_context* exec_c;
        gqlite::value operator()(const node_ref_sp& _node_ref)
        {
          if(_node_ref->cache.get_type() == value_type::invalid)
          {
            // Retrieve properties
            gqlite::value properties;
            {
              // Query
              value_vector properties_val_list_vector = exec_c->data->execute_sql(sqlite_queries::node_get_properties(exec_c->graph_name), {{1, _node_ref->id}}).to_vector();
              errors::check_condition(properties_val_list_vector.size() == 1, "When getting a node, should have received only one node");
              value_vector properties_row = properties_val_list_vector.front().to_vector();
              errors::check_condition(properties_row.size() == 1, "When getting a node, properties get should only have given one column");
              properties = gqlite::value::from_json(properties_row.front().to_string());
            }

            // Retrieve labels
            value_vector labels;
            {
              // Query
              value_vector labels_val_list_vector = exec_c->data->execute_sql(sqlite_queries::node_get_labels(exec_c->graph_name), {{1, _node_ref->id}}).to_vector();
              for(const gqlite::value& label_row_value : labels_val_list_vector)
              {
                value_vector label_row = label_row_value.to_vector();
                errors::check_condition(label_row.size() == 1, "When getting a node, labels get should only have given one column");
                labels.push_back(exec_c->data->label_for_id(label_row.front().to_integer()));
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
              value_vector properties_val_list_vector = exec_c->data->execute_sql(sqlite_queries::edge_get_label_properties(exec_c->graph_name), {{1, _edge_ref->id}}).to_vector();
              errors::check_condition(properties_val_list_vector.size() == 1, "When getting an edge, should have received only one edge");
              value_vector properties_row = properties_val_list_vector.front().to_vector();
              errors::check_condition(properties_row.size() == 2, "When getting an edge, properties get should only have given two column");
              label = exec_c->data->label_for_id(properties_row[0].to_integer());
              properties = gqlite::value::from_json(properties_row[1].to_string());
            }
            // Generate the cache
            _edge_ref->cache = gqlite::value{{
              {"type", gqlite::value("edge")}, {"label", gqlite::value(label)}, {"id", gqlite::value(_edge_ref->id)}, {"properties", gqlite::value(properties)}
            }};
          }
          return _edge_ref->cache;
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
     * @return a value representing the executed value from @p _value to be used in sql query.
     * A.k.a. nodes/edges id or the value
     */
    gqlite::value get_sql_value(const exec_value& _value)
    {
      struct value_getter
      {
        execution_context* exec_c;
        gqlite::value operator()(const node_ref_sp& _node_ref)
        {
          return _node_ref->id;
        }
        gqlite::value operator()(const edge_ref_sp& _edge_ref)
        {
          return _edge_ref->id;
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
  };

  /**
   * Context for the evaluation of one row.
   */
  struct evaluation_context
  {
    exec_value_table table;
    std::vector<exec_value> current_row;
    std::vector<std::string> new_vars; ///< i.e. the new variables introduced in this match

    void prepare_current_row(const exec_value_table& _table, std::size_t _i, bool _first_statement)
    {
      if(current_row.size() < table.get_columns_count())
      {
        current_row.resize(table.get_columns_count());
      }
      if(_first_statement)
      {
        std::fill_n(current_row.begin(), current_row.size(), empty{});
      } else {
        exec_value_table::row_view row_init = _table.get_row(_i);
        std::copy(row_init.begin(), row_init.end(), current_row.begin());
        std::fill_n(current_row.begin() + row_init.size(), current_row.size() - row_init.size(), empty{});
      }
    }
    bool has_variable(const std::string& _variable)
    {
      return table.has_column(_variable);
    }
    bool is_variable_set(const std::string& _variable)
    {
      return table.has_column(_variable) and not std::holds_alternative<empty>(current_row[table.get_column_index(_variable)]);
    }
    bool is_new_variable(const std::string& _variable)
    {
      return std::find(new_vars.begin(), new_vars.end(), _variable) != new_vars.end();
    }
    std::size_t get_new_variable_index(const std::string& _variable)
    {
      return std::find(new_vars.begin(), new_vars.end(), _variable) - new_vars.begin();
    }
    void define_variable(const std::string& _variable)
    {
      if(_variable.empty()) return;
      if(table.has_column(_variable))
      {
        throw exception("Variable {} is already bound.", _variable);
      }
      table.add_column(_variable);
      new_vars.push_back(_variable);
    }
    void define_variable_if_needed(const std::string& _variable)
    {
      if(_variable.empty()) return;
      if(not table.has_column(_variable))
      {
        table.add_column(_variable);
        new_vars.push_back(_variable);
      }
    }
    exec_value get_variable(const std::string& _variable)
    {
      if(table.has_column(_variable))
      {
        return current_row[table.get_column_index(_variable)];
      } else {
        throw exception("Variable {} is not defined.", _variable);
      }
    }
    void set_variable(const std::string& _variable, const exec_value& _ev)
    {
      current_row[table.get_column_index(_variable)] = _ev;
    }

    /**
     * @return a node ref or an exception if not a node ref
     */
    node_ref_sp get_node_ref(const exec_value& _value)
    {
      if(std::holds_alternative<node_ref_sp>(_value))
      {
        return std::get<node_ref_sp>(_value);
      }
      throw exception("Expected a reference to a node.");
    }
    void define_variables(const std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>>& _patterns)
    {
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _patterns)
      {
        pattern.visit<void>([this](const algebra::graph_node_csp _node)
        {
          define_variable(_node->get_variable());
        }, [this](const algebra::graph_edge_csp _edge)
        {
          define_variable_if_needed(_edge->get_source()->get_variable());
          define_variable(_edge->get_variable());
          define_variable_if_needed(_edge->get_destination()->get_variable());
        });
      }
    }
  };

  /**
   * Context for the generation of the sql match query
   */
  struct match_context
  {
    execution_context* exec_c;
    evaluation_context* eval_c;
    int count = 0;
    std::string sql_variables;
    std::size_t sql_variables_count = 0;
    std::string sql_tables;
    std::string sql_conditions;
    std::map<int, value> bindings;
    std::vector<std::pair<std::string, std::string>> edge_sql_var_to_oc_var;
    int label_count = 0;
    struct var_info
    {
      std::string sql_var;
      std::string sql_properties_var;
      std::string sql_label_var;
      std::size_t sql_column;
      bool is_node;
    };
    std::map<std::string, var_info> oc_var_to_sql_var;
    /**
     * Generate the SQL join operation needed for matchin labels
     */
    std::string generate_labels_match(const std::vector<std::string>& _labels, const std::string& _node_variable, bool _start_with_and)
    {
      std::string cond;
      for(const std::string& label : _labels)
      {
        sql_tables += format_string(" JOIN gqlite_{}_labels AS tb_lab{}", exec_c->graph_name, label_count);
        if(not cond.empty() or _start_with_and) cond += " AND ";
        std::size_t idx = bind_value(exec_c->data->id_for_label(label));
        cond += format_string(" {} = tb_lab{}.node_id AND ?{} = tb_lab{}.label ",
                      _node_variable, label_count, to_string_fixed_width(idx, 3), label_count);
        ++label_count;
      }
      return cond;
    }

    var_info& get_variable_info(const std::string& _oc_variable)
    {
      auto it = oc_var_to_sql_var.find(_oc_variable);
      if(it == oc_var_to_sql_var.end())
      {
        throw gqlite::exception("Unknown variable '{}'", _oc_variable);
      }
      return it->second;
    }

    std::string get_sql_variable(const std::string& _oc_variable)
    {
      return get_variable_info(_oc_variable).sql_var;
    }
    std::string get_sql_properties_variable(const std::string& _oc_variable)
    {
      var_info& info = get_variable_info(_oc_variable);
      std::string properties_var = info.sql_properties_var;
      if(properties_var.empty())
      {
        properties_var = retrieve_sql_properties(info.sql_var);
        info.sql_properties_var = properties_var;
      }
      return properties_var;
    }
    std::string retrieve_sql_properties(const std::string& _sql_var)
    {
      std::string count_s = std::to_string(count++);
      std::string properties_var = format_string("tbp{}.properties", count_s);
      sql_tables += format_string(" JOIN gqlite_{}_nodes AS tbp{} ", exec_c->graph_name, count_s);
      sql_conditions += format_string(" AND tbp{}.id = {}", count_s, _sql_var);
      return properties_var;
    }
    std::size_t bind_value(const value& _value)
    {
      std::size_t binding_id = bindings.size() + 1;
      bindings[binding_id] = _value;
      return binding_id;
    }

  };

  /**
   * Visitor to build a sql query
   */
  struct sql_filter_visitor : public gqlite::oc::algebra::default_node_visitor<std::string>
  {
    match_context* mc;
    std::string visit_default(algebra::node_csp _node) override
    {
      throw gqlite::exception("Unimplemented statement node {} in sql_filter_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    std::string visit(algebra::variable_csp _node) override
    {
      return mc->get_sql_variable(_node->get_identifier());
    }
    std::string visit(algebra::has_labels_csp _node) override
    {
      return mc->generate_labels_match(_node->get_labels(), mc->get_sql_variable(_node->get_left()), false);
    }
    std::string visit(algebra::value_csp _node) override
    {
      std::size_t idx = mc->bind_value(_node->get_value());
      return "?" + to_string_fixed_width(idx, 3);
    }
    std::string visit(algebra::member_access_csp _node) override
    {
      return format_string("json_extract({}, '$.{}')", mc->get_sql_properties_variable(_node->get_left()), string::join(_node->get_path(), "."));
    }
    std::string visit(algebra::function_call_csp _node) override
    {
      if(_node->get_name() == "type")
      {
        errors::check_arguments_size("type", _node->get_arguments(), 1);
        algebra::node_csp arg0 = _node->get_arguments().front();
        errors::check_argument_type("type", arg0->get_type(), algebra::node_type::variable);
        algebra::variable_csp arg0_var = std::static_pointer_cast<const algebra::variable>(arg0);
        match_context::var_info& info = mc->get_variable_info(arg0_var->get_identifier());
        if(info.is_node)
        {
          throw gqlite::exception("type expect an edge for variable {}", arg0_var->get_identifier());
        }
        return gqlite::format_string("(SELECT label FROM gqlite_labels WHERE id = {})", info.sql_label_var);
      } else {
        throw gqlite::exception("unknown function {}", _node->get_name());
      }
    }
#define FILTER_VISITOR_BINARY_OP(_AL_, _OP_)                                                          \
    std::string visit(algebra::_AL_ ## _csp _node) override                                           \
    {                                                                                                 \
      return format_string("({} " _OP_ " {})", start(_node->get_left()), start(_node->get_right()));  \
    }
    FILTER_VISITOR_BINARY_OP(logical_and, "AND")
    FILTER_VISITOR_BINARY_OP(logical_or, "OR")
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
    std::string visit(algebra::is_not_null_csp _node) override
    {
      return format_string("({} IS NOT NULL)", start(_node->get_value()));
    }
    std::string visit(algebra::is_null_csp _node) override
    {
      return format_string("({} IS NULL)", start(_node->get_value()));
    }
  };
  /**
   * Visitor to evaluate expressions.
   */
  struct evaluator_visitor : public gqlite::oc::algebra::default_node_visitor<exec_value>
  {
    execution_context* exec_c;
    evaluation_context* eval_c;
    exec_value visit_default(algebra::node_csp _node) override
    {
      throw gqlite::exception("Unimplemented statement node {} in evaluator_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    value get_properties(const std::unordered_map<std::string, algebra::node_csp>& _properties)
    {
      value_map props;
      for(auto const& [k,v] : _properties)
      {
        props[k] = exec_c->get_value(start(v));
      }
      return props;
    }
    value_map get_properties(const exec_value& _value)
    {
      return exec_c->get_value(_value).to_map()["properties"].to_map();
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
    exec_value visit(algebra::value_csp _node) override
    {
      return _node->get_value();
    }
    exec_value visit(algebra::variable_csp _node) override
    {
      return eval_c->get_variable(_node->get_identifier());
    }
    exec_value visit(algebra::array_csp _node) override
    {
      std::vector<value> values;
      for(const algebra::node_csp& v : _node->get_array())
      {
        values.push_back(exec_c->get_value(start(v)));
      }
      return values;
    }
    exec_value visit(algebra::map_csp _node) override
    {
      return get_properties(_node->get_map());
    }
    exec_value visit(algebra::member_access_csp _node) override
    {
      exec_value value = eval_c->get_variable(_node->get_left());

      struct member_access
      {
        evaluator_visitor* self;
        algebra::member_access_csp node;
        gqlite::value operator()(const node_ref_sp& _v)
        {
          return self->get_property(self->get_properties(_v), node->get_path());
        }
        gqlite::value operator()(const edge_ref_sp& _v)
        {
          return self->get_property(self->get_properties(_v), node->get_path());
        }
        gqlite::value operator()(const gqlite::value& _v, bool _allow_array = true)
        {
          switch(_v.get_type())
          {
            case gqlite::value_type::map:
            {
              return self->get_property(_v.to_map(), node->get_path());
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
    bool is_numeric(value_type _vt)
    {
      return _vt == value_type::integer or _vt == value_type::number;
    }
    exec_value visit(algebra::addition_csp _node) override
    {
      exec_value left_ev = start(_node->get_left());
      exec_value right_ev = start(_node->get_right());
      errors::check_condition(std::holds_alternative<value>(left_ev), "Binary operations must be done on null values.");
      errors::check_condition(std::holds_alternative<value>(right_ev), "Binary operations must be done on null values.");
      value left_val = std::get<value>(left_ev);
      value right_val = std::get<value>(right_ev);
      if(left_val.get_type() == value_type::string and right_val.get_type() == value_type::string)
      {
        return left_val.to_string() + right_val.to_string();
      } else if(is_numeric(left_val.get_type()) and is_numeric(right_val.get_type()))
      {
        if(left_val.get_type() == value_type::integer and right_val.get_type() == value_type::integer)
        {
          return left_val.to_integer() + right_val.to_integer();
        } else {
          return left_val.to_double() + right_val.to_double();
        }
      } else {
        throw exception("Cannot add {} with {}.", left_val.to_json(), right_val.to_json());
      }
    }
    exec_value visit(algebra::is_not_null_csp _node) override
    {
      exec_value left_ev = start(_node->get_value());
      return not std::holds_alternative<empty>(left_ev)
            and (not std::holds_alternative<value>(left_ev) or std::get<value>(left_ev).get_type() != value_type::invalid);
    }
    exec_value visit(algebra::is_null_csp _node) override
    {
      exec_value left_ev = start(_node->get_value());
      return std::holds_alternative<empty>(left_ev)
            or (std::holds_alternative<value>(left_ev) and std::get<value>(left_ev).get_type() == value_type::invalid);;
    }
    exec_value visit(algebra::function_call_csp _node) override
    {
      std::vector<value> args;
      for(const algebra::node_csp& arg : _node->get_arguments())
      {
        args.push_back(exec_c->get_value(start(arg)));
      }
      return functions::call(_node->get_name(), args);
    }
  };
  /**
   * Visitor to execute statements: CREATE, MATCHES, RETURN...
   */
  struct statement_visitor : public gqlite::oc::algebra::default_node_visitor<value>
  {
    execution_context exec_c;
    exec_value_table table;
    bool first_statement = true;
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Common functions
    value visit_default(algebra::node_csp _node) override
    {
      throw gqlite::exception("Unimplemented statement node {} in statement_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Creation functions
    bool has_node_ref(evaluator_visitor* _ev, algebra::graph_node_csp _node)
    {
      return not _node->get_variable().empty() and _ev->eval_c->is_variable_set(_node->get_variable());
    }
    node_ref_sp create_node(evaluator_visitor* _ev, algebra::graph_node_csp _node)
    {
      value props;
      if(_node->get_properties())
      {
        props = _ev->get_properties(_node->get_properties()->get_map());
      } else {
        props = value_map();
      }
      std::string json_properties = props.to_json();
      exec_c.data->execute_sql(sqlite_queries::node_create(exec_c.graph_name), {{1, json_properties}});
      int row_id = exec_c.data->last_row_id();
      for(const std::string& label : _node->get_labels())
      {
        exec_c.data->execute_sql(sqlite_queries::node_add_label(exec_c.graph_name), {{1, exec_c.data->id_for_label(label)}, {2, row_id}});
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
        _ev->eval_c->set_variable(_node->get_variable(), nr);
      }
      
      return nr;
    }
    void create_edge(evaluator_visitor* _ev, algebra::graph_edge_csp _edge)
    {
      node_ref_sp source = has_node_ref(_ev, _edge->get_source()) ? _ev->eval_c->get_node_ref(_ev->eval_c->get_variable(_edge->get_source()->get_variable())) : create_node(_ev, _edge->get_source());
      node_ref_sp destination = has_node_ref(_ev, _edge->get_destination()) ? _ev->eval_c->get_node_ref(_ev->eval_c->get_variable(_edge->get_destination()->get_variable())) : create_node(_ev, _edge->get_destination());
      std::string label = _edge->get_labels().empty() ? std::string() : _edge->get_labels().front();
      int label_id = exec_c.data->id_for_label(label);
      
      value props;
      if(_edge->get_properties())
      {
        props = _ev->get_properties(_edge->get_properties()->get_map());
      } else {
        props = value_map();
      }
      exec_c.data->execute_sql(sqlite_queries::edge_create(exec_c.graph_name), {{1, label_id}, {2, props.to_json()}, {3, source->id}, {4, destination->id}});
      int row_id = exec_c.data->last_row_id();
      edge_ref_sp nr = std::make_shared<edge_ref>(edge_ref{
          row_id,
          value{
            {{"type", value("edge")}, {"label", value(label)}, {"id", value(row_id)}, {"properties", props}}
          }
        });
      if(not _edge->get_variable().empty())
      {
        _ev->eval_c->set_variable(_edge->get_variable(), nr);
      }
    }
    /**
     * Execute the create statement
     */
    value visit(algebra::create_csp _node) override
    {
      evaluation_context eval_c;
      eval_c.table.add_columns(table.get_columns_names());
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;
      std::size_t max_iter = first_statement ? 1 : table.get_rows_count();
      // Set variables
      eval_c.define_variables(_node->get_patterns());
      // Create nodes
      for(std::size_t i = 0; i < max_iter; ++i)
      {
        eval_c.prepare_current_row(table, i, first_statement);
        for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
        {
          pattern.visit<void>([this, &eval_v](const algebra::graph_node_csp _node)
          {
            create_node(&eval_v, _node);
          },
          [this, &eval_v](const algebra::graph_edge_csp _edge)
          {
            create_edge(&eval_v, _edge);
          });
        }
        eval_c.table.add_row(eval_c.current_row);
      }
      table = eval_c.table;
      return value{};
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Match
    /**
     * Generate a filter expression for a map
     */
    std::string generate_filter(const algebra::map_csp& _map, const std::string& _path, sql_filter_visitor* _filter_visitor)
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
    void generate_var_match(match_context* _mc, const std::string& _oc_var, const std::string& _sql_var, const std::string& _sql_properties_var, const std::string& _label_var, bool _is_node)
    {
      if(not _oc_var.empty())
      {
        auto it = _mc->oc_var_to_sql_var.find(_oc_var);
        if(it == _mc->oc_var_to_sql_var.end())
        {
          // Check if it is a new variable and need to be extracted in the SQL statement
          errors::check_condition(_mc->eval_c->is_new_variable(_oc_var), "only new variable should be defined.");
          if(not _mc->sql_variables.empty()) _mc->sql_variables += ", ";
          _mc->sql_variables += _sql_var;
          _mc->oc_var_to_sql_var[_oc_var] = {_sql_var, _sql_properties_var, _label_var, _mc->sql_variables_count, _is_node};
          ++_mc->sql_variables_count;
        } else {
          // Generate match
          if(it->second.is_node != _is_node) throw exception("{} is redefined as a variable of a different type", _oc_var);
          _mc->sql_conditions += format_string(" AND {} = {} ", _sql_var, it->second.sql_var);
        }
      }
    }
    value visit(algebra::match_csp _node) override
    {
      evaluation_context eval_c;
      eval_c.table.add_columns(table.get_columns_names());
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;

      match_context mc;
      mc.exec_c = &exec_c;
      mc.eval_c = &eval_c;
      sql_filter_visitor fil_vis;
      fil_vis.mc = &mc;

      std::size_t max_iter = first_statement ? 1 : table.get_rows_count();
      // Set variables
      eval_c.define_variables(_node->get_patterns());

      for(std::size_t i = 0; i < max_iter; ++i)
      {
        eval_c.prepare_current_row(table, i, first_statement);

        // 0) Bind the values
        {
          std::size_t column_index = 0;
          for(const std::string& column : table.get_columns_names())
          {
            // It is a value, bind it.
            int idx = mc.bind_value(mc.exec_c->get_sql_value(mc.eval_c->get_variable(column)));
            std::string binding = format_string("?{}", to_string_fixed_width(idx, 3));
            mc.oc_var_to_sql_var[column] = {binding, std::string(), std::string(), std::size_t(-1), std::holds_alternative<node_ref_sp>(eval_c.current_row[column_index])};
          }
          ++column_index;
        }

        // 1) Go through the patterns
        for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
        {
          if(mc.count != 0)
          {
            mc.sql_tables += " JOIN ";
          }
          pattern.visit<void>([this, &mc, &fil_vis](const algebra::graph_node_csp _node)
          {
            std::string count_s = std::to_string(mc.count++);
            std::string sql_var = format_string("tb{}.id", count_s);
            std::string sql_properties_var = format_string("tb{}.properties", count_s);
            mc.sql_tables += format_string("gqlite_{}_nodes AS tb{}", exec_c.graph_name, count_s);
            mc.sql_conditions += mc.generate_labels_match(_node->get_labels(), format_string("tb{}.id", count_s), true);
            if(_node->get_properties())
            {
              mc.sql_conditions += generate_filter(_node->get_properties(), sql_properties_var + ", '$", &fil_vis);
            }
            generate_var_match(&mc, _node->get_variable(), sql_var, sql_properties_var, std::string(), true);
          },
          [this, &mc, &fil_vis](const algebra::graph_edge_csp _edge)
          {
            std::string count_s = std::to_string(mc.count++);
            std::string sql_source_var = format_string("tb{}.left", count_s);
            std::string sql_edge_var = format_string("tb{}.id", count_s);
            std::string sql_properties_edge_var = format_string("tb{}.properties", count_s);
            std::string sql_label_edge_var = format_string("tb{}.label", count_s);
            std::string sql_source_properties_var;
            std::string sql_destination_properties_var;
            std::string sql_destination_var = format_string("tb{}.right", count_s);
            if(_edge->get_directivity() == algebra::edge_directivity::undirected)
            {
              mc.sql_tables += format_string("gqlite_{}_edges_undirected AS tb{}", exec_c.graph_name, count_s);
            } else {
              mc.sql_tables += format_string("gqlite_{}_edges AS tb{}", exec_c.graph_name, count_s);
            }
            mc.sql_conditions += mc.generate_labels_match(_edge->get_source()->get_labels(), "tb" + count_s + ".left", true);
            mc.sql_conditions += mc.generate_labels_match(_edge->get_destination()->get_labels(), "tb" + count_s + ".right", true);
            if(not _edge->get_labels().empty())
            {
              mc.sql_conditions += " AND (FALSE ";
              for(const std::string& label : _edge->get_labels())
              {
                mc.sql_conditions += format_string(" OR {} = {}", sql_label_edge_var
                , exec_c.data->id_for_label(label));
              }
              mc.sql_conditions += ")";
            }
            if(_edge->get_properties())
            {
              mc.sql_conditions += generate_filter(_edge->get_properties(), sql_properties_edge_var + ", '$", &fil_vis);
            }
            if(_edge->get_source()->get_properties())
            {
              sql_source_properties_var = mc.retrieve_sql_properties(sql_source_var);
              mc.sql_conditions += generate_filter(_edge->get_source()->get_properties(), sql_source_properties_var + ", '$", &fil_vis);
            }
            if(_edge->get_destination()->get_properties())
            {
              sql_destination_properties_var = mc.retrieve_sql_properties(sql_destination_var);
              mc.sql_conditions += generate_filter(_edge->get_destination()->get_properties(), sql_destination_properties_var + ", '$", &fil_vis);
            }
            // Ensure identical oc variablle are joined in SQL
            generate_var_match(&mc, _edge->get_source()->get_variable(), sql_source_var, sql_source_properties_var, std::string(), true);
            generate_var_match(&mc, _edge->get_variable(), sql_edge_var, sql_properties_edge_var, sql_label_edge_var, false);
            generate_var_match(&mc, _edge->get_destination()->get_variable(), sql_destination_var, sql_destination_properties_var, std::string(), true);
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
        std::string sql_query = "SELECT " + mc.sql_variables + " FROM " + mc.sql_tables + mc.sql_conditions;
        gqlite::value r = exec_c.data->execute_sql(sql_query, mc.bindings);

        // 4) Store the results
        std::vector<exec_value> values;
        values.resize(eval_c.table.get_columns_count());
        std::size_t start_new_values = 0;
        if(not first_statement)
        {
          exec_value_table::row_view row_init = table.get_row(i);
          std::copy(row_init.begin(), row_init.end(), values.begin());
          start_new_values = row_init.size();
        }
        for(const gqlite::value& row_value : r.to_vector())
        {
          value_vector row = row_value.to_vector();
          errors::check_condition(row.size() == mc.eval_c->new_vars.size(), "Wrong number of column return by SQL Query.");
          for(const std::string& k : mc.eval_c->new_vars)
          {
            match_context::var_info vi = mc.oc_var_to_sql_var[k];
            int id = row[vi.sql_column].to_integer();
            if(vi.is_node)
            {
              values[vi.sql_column + start_new_values] = std::make_shared<node_ref>(id);
            } else {
              values[vi.sql_column + start_new_values] = std::make_shared<edge_ref>(id);
            }
          }
          eval_c.table.add_row(values);
        }
      }
      table = eval_c.table;
      return value{};
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // delete
    value visit(algebra::delete_statement_csp _ds) override
    {
      evaluation_context eval_c;
      eval_c.table = table;
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;

      struct deleter
      {
        statement_visitor* self;
        bool detach;
        void operator()(const node_ref_sp& _node)
        {
          if(detach)
          {
            self->exec_c.data->execute_sql(sqlite_queries::edge_delete_by_node(self->exec_c.graph_name), {{1, _node->id}});
          } else {
            value result = self->exec_c.data->execute_sql(sqlite_queries::edge_count_by_node(self->exec_c.graph_name), {{1, _node->id}});
            value_vector rows = result.to_vector();
            errors::check_condition(rows.size() == 1, "Invalid number of rows for counting edges got {} expected 1.", rows.size());
            value_vector row = rows.front().to_vector();
            errors::check_condition(row.size() == 1, "Invalid number of columns for counting edges got {} expected 1.", rows.size());
            int count = row.front().to_integer();
            errors::check_condition(count == 0, "Cannot delete node with {} relationships.", count);
          }
          self->exec_c.data->execute_sql(sqlite_queries::node_delete(self->exec_c.graph_name), {{1, _node->id}});
        }
        void operator()(const edge_ref_sp& _edge)
        {
          self->exec_c.data->execute_sql(sqlite_queries::edge_delete(self->exec_c.graph_name), {{1, _edge->id}});
        }
        void operator()(const value& _value)
        {
          throw exception("Cannot delete a value.");
        }
        void operator()(const empty&)
        {
          throw exception("Cannot delete an empty value.");
        }
      };

      for(int i = 0; i < table.get_rows_count(); ++i)
      {
        value_vector row;
        eval_c.prepare_current_row(table, i, false);
        for(const algebra::node_csp& rv : _ds->get_expressions())
        {
          exec_value ev =eval_v.start(rv);
          std::visit(deleter{this, _ds->get_detach()}, ev);
        }
      }
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // set
    value visit(algebra::set_csp _set) override
    {
      evaluation_context eval_c;
      eval_c.table = table;
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;

      struct property_setter_adder_base
      {
        statement_visitor* self;
        std::vector<std::string> path;
        exec_value new_value;
        void execute(const std::string& _query, int _node_id)
        {
          std::string path_string = "$";
          for(const std::string& pe : path)
          {
            path_string += "." + pe;
          }
          self->exec_c.data->execute_sql(_query, {{1, _node_id}, {2, path_string}, {3, self->exec_c.get_value(new_value)}});
        }
        void operator()(const value& _value)
        {
          throw exception("Only node/edge can be set.");
        }
        void operator()(const empty&)
        {
          throw exception("Try to set a null value.");
        }

      };

      struct property_setter : property_setter_adder_base
      {
        using property_setter_adder_base::operator();
        void operator()(const node_ref_sp& _node)
        {
          _node->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::node_set_property(self->exec_c.graph_name), _node->id);
        }
        void operator()(const edge_ref_sp& _edge)
        {
          _edge->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::edge_set_property(self->exec_c.graph_name), _edge->id);
        }
      };
      struct property_adder : property_setter_adder_base
      {
        using property_setter_adder_base::operator();
        void operator()(const node_ref_sp& _node)
        {
          _node->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::node_add_properties(self->exec_c.graph_name), _node->id);
        }
        void operator()(const edge_ref_sp& _edge)
        {
          _edge->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::node_add_properties(self->exec_c.graph_name), _edge->id);
        }
      };

      struct set_visitor : public gqlite::oc::algebra::default_node_visitor<void>
      {
        statement_visitor* self;
        evaluation_context* eval_c;
        evaluator_visitor* eval_v;
        void visit_default(algebra::node_csp _node) override
        {
          throw gqlite::exception("Unimplemented statement node {} in set_visitor", oc::algebra::node_type_name(_node->get_type()));
        }
        void visit(algebra::set_property_csp _property)
        {
          exec_value target = eval_c->get_variable(_property->get_target()->get_left());
          exec_value ev = eval_v->start(_property->get_expression());
          std::visit(property_setter{self, _property->get_target()->get_path(), ev}, target);
          
        }
        void visit(algebra::add_property_csp _property)
        {
          exec_value target = eval_c->get_variable(_property->get_target()->get_left());
          exec_value ev = eval_v->start(_property->get_expression());
          std::visit(property_adder{self, _property->get_target()->get_path(), ev}, target);
          
        }
        void visit(algebra::edit_labels_csp _property)
        {
          exec_value target = eval_c->get_variable(_property->get_target());
          errors::check_condition(std::holds_alternative<node_ref_sp>(target), "Can only add labels to nodes.");
          node_ref_sp target_nd = std::get<node_ref_sp>(target);
          for(const std::string& label : _property->get_labels())
          {
            self->exec_c.data->execute_sql(sqlite_queries::node_add_label(self->exec_c.graph_name), {{2, target_nd->id}, {1, self->exec_c.data->id_for_label(label)}});
          }
        }
      };

      set_visitor set_v;
      set_v.self = this;
      set_v.eval_c = &eval_c;
      set_v.eval_v = &eval_v;
      for(int i = 0; i < table.get_rows_count(); ++i)
      {
        value_vector row;
        eval_c.prepare_current_row(table, i, false);
        for(const algebra::node_csp& node : _set->get_nodes())
        {
          set_v.start(node);
        }
      }
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // remove
    value visit(algebra::remove_csp _set) override
    {
      evaluation_context eval_c;
      eval_c.table = table;
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;

      struct property_remover
      {
        statement_visitor* self;
        std::vector<std::string> path;
        void execute(const std::string& _query, int _node_id)
        {
          std::string path_string = "$";
          for(const std::string& pe : path)
          {
            path_string += "." + pe;
          }
          self->exec_c.data->execute_sql(_query, {{1, _node_id}, {2, path_string}});
        }
        void operator()(const value& _value)
        {
          throw exception("Only node/edge can be set.");
        }
        void operator()(const empty&)
        {
          throw exception("Try to set a null value.");
        }
        void operator()(const node_ref_sp& _node)
        {
          _node->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::node_remove_property(self->exec_c.graph_name), _node->id);
        }
        void operator()(const edge_ref_sp& _edge)
        {
          _edge->cache = gqlite::value(); // Invalidate cache
          execute(sqlite_queries::edge_remove_property(self->exec_c.graph_name), _edge->id);
        }
      };

      struct remove_visitor : public gqlite::oc::algebra::default_node_visitor<void>
      {
        statement_visitor* self;
        evaluation_context* eval_c;
        evaluator_visitor* eval_v;
        void visit_default(algebra::node_csp _node) override
        {
          throw gqlite::exception("Unimplemented statement node {} in set_visitor", oc::algebra::node_type_name(_node->get_type()));
        }
        void visit(algebra::remove_property_csp _property)
        {
          exec_value target = eval_c->get_variable(_property->get_target()->get_left());
          std::visit(property_remover{self, _property->get_target()->get_path()}, target);
          
        }
        void visit(algebra::edit_labels_csp _property)
        {
          exec_value target = eval_c->get_variable(_property->get_target());
          errors::check_condition(std::holds_alternative<node_ref_sp>(target), "Can only add labels to nodes.");
          node_ref_sp target_nd = std::get<node_ref_sp>(target);
          for(const std::string& label : _property->get_labels())
          {
            self->exec_c.data->execute_sql(sqlite_queries::node_remove_label(self->exec_c.graph_name), {{2, target_nd->id}, {1, self->exec_c.data->id_for_label(label)}});
          }
        }

      };

      remove_visitor set_v;
      set_v.self = this;
      set_v.eval_c = &eval_c;
      set_v.eval_v = &eval_v;
      for(int i = 0; i < table.get_rows_count(); ++i)
      {
        value_vector row;
        eval_c.prepare_current_row(table, i, false);
        for(const algebra::node_csp& node : _set->get_nodes())
        {
          set_v.start(node);
        }
      }
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // with
    value visit(algebra::with_csp w) override
    {
      if(w->get_all() and w->get_expressions().empty())
      {
        return value{};
      }
      if(w->get_all() and not w->get_expressions().empty())
      {
        throw exception("Unimplemented WITH *, expressions");
      }
      evaluation_context eval_c;
      eval_c.table = table;
      evaluator_visitor eval_v;
      eval_v.exec_c = &exec_c;
      eval_v.eval_c = &eval_c;

      exec_value_table out_table;
      value_vector labels;
      for(const algebra::named_expression_csp& rv : w->get_expressions())
      {
        out_table.add_column(rv->get_name());
      }
      std::size_t max_iter = first_statement ? 1 : table.get_rows_count();
      for(int i = 0; i < max_iter; ++i)
      {
        std::vector<exec_value> row;
        eval_c.prepare_current_row(table, i, first_statement);
        for(const algebra::named_expression_csp& rv : w->get_expressions())
        {
          row.push_back(eval_v.start(rv->get_expression()));
        }
        out_table.add_row(row);
      }
      table = out_table;
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Return
    value visit(algebra::return_statement_csp rs) override
    {
      if(rs->get_all())
      {
        errors::check_condition(rs->get_expressions().empty(), "Unimplemented RETURN *, expressions");
        value_vector labels;
        for(const std::string&  c : table.get_columns_names()) { labels.push_back(c); }
        value_vector results_rows;
        results_rows.push_back(labels);
        for(int i = 0; i < table.get_rows_count(); ++i)
        {
          value_vector row;
          for(const exec_value& ev : table.get_row(i))
          {
            row.push_back(exec_c.get_value(ev));
          }
          results_rows.push_back(row);
        }
        return results_rows;

      } else {
        evaluation_context eval_c;
        eval_c.table = table;
        evaluator_visitor eval_v;
        eval_v.exec_c = &exec_c;
        eval_v.eval_c = &eval_c;

        value_vector labels;
        for(const algebra::named_expression_csp& rv : rs->get_expressions())
        {
          labels.push_back(rv->get_name());
        }
        value_vector results_rows;
        results_rows.push_back(labels);
        for(int i = 0; i < table.get_rows_count(); ++i)
        {
          value_vector row;
          eval_c.prepare_current_row(table, i, false);
          for(const algebra::named_expression_csp& rv : rs->get_expressions())
          {
            exec_value ev = eval_v.start(rv->get_expression());
            row.push_back(exec_c.get_value(ev));
          }
          results_rows.push_back(row);
        }
        return value(results_rows);
      }
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Statements
    value visit(algebra::statements_csp _node) override
    {
      for(algebra::node_csp node : _node->get_nodes())
      {
        value val = start(node);
        first_statement = false;
        if(node->get_type() == algebra::node_type::return_statement)
        {
          return val;
        }
      }
      return value{};
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
    sqlite_oc_executor::statement_visitor executor;
    executor.exec_c.data = d;
    value val = executor.start(_node);
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
  errors::check_condition(rows.size() == 7, "Invalid number of debug stats got {} expected 7.", rows.size());
  value_map stats;
  stats["nodes_count"] = rows[0].to_vector()[0].to_integer();
  stats["edges_count"] = rows[1].to_vector()[0].to_integer();
  stats["labels_assignment_count"] = rows[2].to_vector()[0].to_integer();
  stats["properties_count"] = rows[3].to_vector()[0].to_integer() + rows[4].to_vector()[0].to_integer();
  stats["labels_count"] = rows[5].to_vector()[0].to_integer();
  stats["labels_assignment_nodes_count"] = rows[6].to_vector()[0].to_integer();
  return stats;
}
