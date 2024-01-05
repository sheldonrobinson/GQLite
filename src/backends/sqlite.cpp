#include "sqlite.h"

#include <list>
#include <map>
#include <optional>
#include <variant>

#include <sqlite3.h>

#include "sqlite_queries.h"

#include "../oc/algebra/default_node_visitor.h"
#include "../oc/algebra/visitors/expression_analyser.h"
#include "../global.h"
#include "../logging.h"
#include "../string.h"

using namespace std::string_literals;
using namespace gqlite::backends;

namespace gqlite::backends
{
  void initialise_sqlite_ext(sqlite3* db);
  struct sqlite_data
  {
    sqlite3* handle;
    std::unordered_map<int64_t, std::string> id_to_label = {{0, std::string()}};
    std::unordered_map<std::string, int64_t> label_to_id = {{std::string(), 0}};

    std::unordered_map<std::string, std::function<value(const value_vector&)>> procedures;

    void throw_last_error(const std::string& _query);
    void graph_create(const std::string& _name);
    bool graph_has(const std::string& _name);
    bool table_has(const std::string& _name);

    value execute_sql(const std::string& _query, const std::map<int, value>& _bindings = {});
    void clean_up();
    uint64_t last_row_id();
    int64_t id_for_label(const std::string& _string);
    std::string label_for_id(int64_t _id);
  };

  bool is_number(oc::algebra::expression_type _et)
  {
    return _et == oc::algebra::expression_type::integer or _et == oc::algebra::expression_type::floating_point;
  }
}

struct sqlite::data : public sqlite_data
{};

void sqlite_data::throw_last_error(const std::string& _query)
{
  std::string err_msg = sqlite3_errmsg(handle);
  if(err_msg.starts_with("__gqlite_fun: "))
  {
    value_map vm = value::from_json(err_msg.substr(sizeof("__gqlite_fun: ")-1)).to_map();
    throw_exception(exception_stage::runtime, exception_code(vm["code"].to_integer()), vm["message"].to_string());
  } else {
    throw_exception(exception_stage::runtime, exception_code::internal_error, "Sqlite error '{}', while executing {}.", err_msg, _query);
  }
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
  const char* ptr = _query.data();
  const char* ptr_end = _query.data() + _query.size();
  value_vector q_rs;
  while(ptr != ptr_end)
  {
    sqlite3_stmt* ps;
    if(sqlite3_prepare_v2(handle, ptr, ptr_end - ptr, &ps, &ptr) != SQLITE_OK)
    {
      sqlite3_finalize(ps);
      throw_last_error(_query);
    }
    for(const auto& [key, value] : _bindings)
    {
      switch(value.get_type())
      {
        case value_type::invalid:
          sqlite3_bind_null(ps, key);
          break;
        case value_type::boolean:
          sqlite3_bind_int64(ps, key, value.to_bool() ? 1 : 0);
          break;
        case value_type::integer:
          sqlite3_bind_int64(ps, key, value.to_integer());
          break;
        case value_type::floating_point:
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
            row.push_back(int64_t(sqlite3_column_int64(ps, i)));
            break;
          case SQLITE_FLOAT:
            row.push_back(sqlite3_column_double(ps, i));
            break;
          case SQLITE_BLOB:
            sqlite3_finalize(ps);
            throw_exception(exception_stage::runtime, exception_code::internal_error, "Blobs are not supported.");
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
      throw_last_error(_query);
    }
  }
  if(q_rs.size() == 1)
  {
    return q_rs[0];
  } else {
    return q_rs;
  }
}

void sqlite_data::clean_up()
{
  value temp = execute_sql("SELECT name FROM sqlite_temp_master;");
  for(const value& tv : temp.to_vector())
  {
    execute_sql(format_string("DROP TABLE {}", tv.to_vector().front().to_string()));
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
  errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one result");
  v = v.begin()->to_vector();
  errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one result");
  return v.begin()->to_bool();
}

bool sqlite_data::table_has(const std::string& _name)
{
  value r = execute_sql(sqlite_queries::table_has(), {{1, _name}});
  value_vector v = r.to_vector();
  errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one result for table_has.");
  v = v.begin()->to_vector();
  errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one column for table_has.");
  return v.begin()->to_bool();
}


uint64_t sqlite_data::last_row_id()
{
  return sqlite3_last_insert_rowid(handle);
}

int64_t sqlite_data::id_for_label(const std::string& _string)
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
      errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one column for label_get_from_id.");
      return v.begin()->to_integer();
    }
  } else {
    return it->second;
  }
}

std::string sqlite_data::label_for_id(int64_t _id)
{
  auto it = id_to_label.find(_id);
  if(it == id_to_label.end())
  {
    value r = execute_sql(sqlite_queries::label_get_from_id(), {{1, _id}});
    value_vector v = r.to_vector();
    if(v.size() == 0)
    {
      throw_exception(exception_stage::runtime, exception_code::internal_error, "Unknown label id {}.", _id);
    } else {
      v = v.begin()->to_vector();
      errors::check_condition(v.size() == 1, exception_stage::runtime, exception_code::internal_error, "Should have gotten only one column for label_get_from_id.");
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
   * Global context for the execution of a query.
   */
  struct execution_context
  {
    sqlite_data* data;
    std::string graph_name = "default";
    int next_view_id = 0;
  };

  struct sql_variable_info
  {
    std::string sql_var_name; ///< name of the sql variable that holds the OC variable
    oc::algebra::expression_type type;
    std::string sql_properties_var_name = std::string(); ///< name of the sql variable that holds the the properties of a node or an edge
    std::string sql_labels_var_name = std::string(); ///< 
  };

  struct table
  {
    std::string view_name;
    std::unordered_map<std::string, sql_variable_info> variables;
  };
  enum class sql_join
  {
    inner, left
  };
  struct sql_query_builder
  {
    std::size_t uid = 0;
    std::size_t table_count = 0;
    std::string variables;
    std::string tables;
    std::string join_conditions;
    std::string where_conditions;
    std::string modifiers;
    std::map<int, value> bindings;
    std::size_t next_uid()
    {
      return uid++;
    }
    std::string bind_value(const value& _value)
    {
      std::size_t binding_id = bindings.size() + 1;
      bindings[binding_id] = _value;
      return format_string("?{}", to_string_fixed_width(binding_id, 3));
    }
    /**
     * Add a new variable with a generic name format.
     */
    std::string add_variable(const std::string& _expression, const char* _format = "var_{}")
    {
      std::string name = format_string(_format, next_uid());
      return add_variable_with_name(_expression, name);
    }
    /**
     * Add a new variable with the given name.
     */
    std::string add_variable_with_name(const std::string& _expression, const std::string& _name)
    {
      if(not variables.empty()) variables += ", ";
      variables += format_string("{} AS {}", _expression, _name);
      return _name;
    }
    std::string add_table(const std::string& _table, sql_join _join, const char* _table_name_format= "tbl_{}")
    {
      if(table_count == 0)
      {
        errors::check_condition(_join == sql_join::inner, exception_stage::runtime, exception_code::internal_error, "Joining with non-inner join on first table.");
        tables += " FROM ";
      } else {
        switch(_join)
        {
          case sql_join::inner:
            tables += " JOIN ";
            break;
          case sql_join::left:
            tables += " LEFT JOIN ";
            break;
        }
      }
      ++table_count;
      std::string table_name = format_string(_table_name_format, next_uid());
      tables += format_string(" {} {} ", _table, table_name);
      return table_name;
    }
    template<typename... _T_>
    void add_join_condition(const std::string& _condition, const _T_&... _args)
    {
      if(_condition.empty()) return;
      if(not join_conditions.empty()) join_conditions += " AND ";
      join_conditions += format_string(_condition, _args...);
    }
    template<typename... _T_>
    void add_where_condition(const std::string& _condition, const _T_&... _args)
    {
      if(_condition.empty()) return;
      where_conditions += where_conditions.empty() ? " WHERE " : " AND ";
      where_conditions += format_string(_condition, _args...);
    }
    void add_modifier(const std::string& _modifier)
    {
      modifiers += _modifier;
    }
    std::string assemble()
    {
      std::string q = "SELECT ";
      q += variables.empty() ? " null " : variables;
      q += tables;
      if(not join_conditions.empty())
      {
        if(table_count == 1)
        {
          q += " WHERE ";
        } else {
          q += " ON ";
        }
        q += join_conditions;
      }
      q += where_conditions;
      q += modifiers;
      return q;
    }
  };

  /**
   * Context for the generation of the sql match query
   */
  struct sql_evaluation_context
  {
    execution_context* exec_c;
    sql_query_builder query_builder;
    std::map<std::string, sql_variable_info> variables;
    std::string previous_table_ref;
  
    // std::string view_name;
    oc::algebra::visitors::expression_analyser expression_analyser;

    table prepare(const table& _table, bool _create_new_table)
    {
      expression_analyser.stage = exception_stage::runtime;
      expression_analyser.variables_f = [this](const std::string& _var_name)
      {
        return get_variable_info(_var_name).type;
      };
      table new_table;
      if(_create_new_table) new_table.view_name = "__view" + std::to_string(++exec_c->next_view_id);
      if(not _table.view_name.empty())
      {
        previous_table_ref = query_builder.add_table(_table.view_name, sql_join::inner, "pt_{}");

        for(auto const& [k, v] : _table.variables)
        {
          std::string sql_var = format_string("{}.{}", previous_table_ref, v.sql_var_name);
          variables[k] = { sql_var, v.type };
          if(_create_new_table)
          {
            std::string col_name = query_builder.add_variable(sql_var);
            new_table.variables[k] = { col_name, v.type };
          }
        }
      }
      return new_table;
    }
    sql_variable_info& get_variable_info(const std::string& _oc_variable)
    {
      auto it = variables.find(_oc_variable);
      if(it == variables.end())
      {
        errors::undefined_variable(exception_stage::runtime, _oc_variable);
      }
      return it->second;
    }
    std::string get_sql_variable(const std::string& _oc_variable)
    {
      return get_variable_info(_oc_variable).sql_var_name;
    }
    std::string get_sql_properties_variable(const std::string& _oc_variable)
    {
      sql_variable_info& info = get_variable_info(_oc_variable);
      std::string properties_var = info.sql_properties_var_name;
      if(properties_var.empty())
      {
        properties_var = retrieve_sql_properties(info.sql_var_name, info.type);
        info.sql_properties_var_name = properties_var;
      }
      return properties_var;
    }
    /**
     * Retrieve SQL properties
     */
    std::string retrieve_sql_properties(const std::string& _sql_var, algebra::expression_type _et)
    {
      std::string table_suffix = (_et == algebra::expression_type::node) ? "nodes" : "edges";
      std::string table_ref = query_builder.add_table(format_string("gqlite_{}_{}", exec_c->graph_name, table_suffix), sql_join::inner, "tbp{}");
      query_builder.add_join_condition("{}.id = {}", table_ref, _sql_var);
      std::string properties_var = format_string("{}.properties", table_ref);
      return properties_var;
    }
  };

  /**
   * Visitor to build a sql query
   */
  struct sql_expression_visitor : public gqlite::oc::algebra::default_node_visitor<std::string>
  {
    sql_evaluation_context* eval_c;
    sql_expression_visitor(sql_evaluation_context* _ec) : eval_c(_ec)
    {}
    std::string visit_default(algebra::node_csp _node) override
    {
      throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Unimplemented node '{}' in sql_expression_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    std::string visit(algebra::variable_csp _node) override
    {
      return eval_c->get_sql_variable(_node->get_identifier());
    }
    std::string visit(algebra::end_of_list_csp ) override
    {
      return eval_c->query_builder.bind_value(std::numeric_limits<int64_t>::max());
    }
    std::string visit(algebra::value_csp _node) override
    {
      return eval_c->query_builder.bind_value(_node->get_value());
    }
    std::string visit(algebra::has_labels_csp _node) override
    {
      oc::algebra::expression_type et = eval_c->get_variable_info(_node->get_left()).type;
      std::string label_ids;
      for(const std::string& l : _node->get_labels())
      {
        if(not label_ids.empty()) label_ids += ", ";
        label_ids += std::to_string(eval_c->exec_c->data->id_for_label(l));
      }
      switch(et)
      {
      case oc::algebra::expression_type::node:
        {
          return "(SELECT count(*) == {} FROM gqlite_{}_labels WHERE node_id={} AND label IN ({}) )"s
              % _node->get_labels().size(), eval_c->exec_c->graph_name, eval_c->get_sql_variable(_node->get_left()), label_ids;
        }
      case oc::algebra::expression_type::edge:
        {
          std::string label_id;
          return "(SELECT count(*) > 0 FROM gqlite_{}_edges WHERE id={} AND label IN ({}) )"s
              % eval_c->exec_c->graph_name, eval_c->get_sql_variable(_node->get_left()), label_ids;
        }
      default:
        errors::invalid_argument_type(exception_stage::runtime, "Can only check labels for nodes and edges.");
      }
    }
    std::string visit(algebra::map_csp _map) override
    {
      std::size_t var_uuid = eval_c->query_builder.next_uid();
      std::string map = ("(SELECT json_group_object(name_{}, val_{}) FROM ("s % var_uuid, var_uuid);
      bool first = true;
      for(const auto& [k, v] : _map->get_map())
      {
        if(not first)
        {
          map += " UNION ";
        }
        map += "SELECT " + eval_c->query_builder.bind_value(k);
        if(first)
        {
          map += " AS name_{}"s % var_uuid;
        }
        map += ", " + start(v);
        if(first)
        {
          map += " AS val_{}"s % var_uuid;
        }
        first = false;
      }
      return map + "))";
    }
    std::string visit(algebra::array_csp _array) override
    {
      std::size_t var_uuid = eval_c->query_builder.next_uid();
      std::string array = "(SELECT json_group_array(val_{}) FROM ("s % var_uuid;
      bool first = true;
      for(algebra::node_csp v : _array->get_array())
      {
        if(not first)
        {
          array += " UNION ";
        }
        array += "SELECT " + start(v);
        if(first)
        {
          array += " AS val_{}"s % var_uuid;
        }
        first = false;
      }
      return array + "))";
    }
    std::string visit(algebra::member_access_csp _node) override
    {
      std::string path_prefix;
      algebra::expression_type et = eval_c->expression_analyser.start(_node->get_left()).type;
      std::string expr;
      if(et == algebra::expression_type::node or et == algebra::expression_type::edge)
      {
        errors::check_condition(_node->get_left()->get_type() == algebra::node_type::variable, exception_stage::runtime, exception_code::internal_error, "Only variable are supported yet for getting properties for node/edge.");
        expr = eval_c->get_sql_properties_variable(std::static_pointer_cast<const algebra::variable>(_node->get_left())->get_identifier());
      } else {
        expr = start(_node->get_left());
      }
      return "json_extract({}, '${}.{}')"s % expr, path_prefix, string::join(_node->get_path(), ".");
    }
    std::string visit(algebra::indexed_access_csp _node) override
    {
      algebra::expression_type et = eval_c->expression_analyser.start(_node->get_left()).type;
      std::string expr;
      if(et == algebra::expression_type::node or et == algebra::expression_type::edge)
      {
        errors::check_condition(_node->get_left()->get_type() == algebra::node_type::variable, exception_stage::runtime, exception_code::internal_error, "Only variable are supported yet for getting properties for node/edge.");
        expr = eval_c->get_sql_properties_variable(std::static_pointer_cast<const algebra::variable>(_node->get_left())->get_identifier());
      } else {
        expr = start(_node->get_left());
      }
      if(_node->get_end())
      {
        return "gqlite_range_access({}, {}, {})"s % expr, start(_node->get_index()), start(_node->get_end());
      } else {
        return "gqlite_indexed_access({}, {})"s % expr, start(_node->get_index());
      }
    }
    std::string visit(algebra::function_call_csp _node) override
    {
      if(_node->get_name() == "coalesce")
      {
        std::string arguments;
        for(algebra::node_csp arg : _node->get_arguments())
        {
          if(not arguments.empty()) arguments += ", ";
          arguments += start(arg);
        }
        return "coalesce(" + arguments + ")";
      } else if(_node->get_name() == "labels")
      {
        errors::check_arguments_size("labels", _node->get_arguments(), 1);
        std::string arg = start(_node->get_arguments().front());
        return sqlite_queries::function_labels(eval_c->exec_c->graph_name, arg);
      } else if(_node->get_name() == "type")
      {
        errors::check_arguments_size("type", _node->get_arguments(), 1);
        std::string arg = start(_node->get_arguments().front());
        std::string edges_table_ref = eval_c->query_builder.add_table(format_string("gqlite_{}_edges", eval_c->exec_c->graph_name), sql_join::inner, "tb_edge_{}");
        std::string labels_table_ref = eval_c->query_builder.add_table("gqlite_labels", sql_join::inner, "tb_lab_{}");
        eval_c->query_builder.add_join_condition("{}.id = {} AND {}.label = {}.id", edges_table_ref, arg, edges_table_ref, labels_table_ref);
        return "{}.label"s % labels_table_ref;
      } else if(_node->get_name() == "range")
      {
        std::string args;
        for(algebra::node_csp arg : _node->get_arguments())
        {
          if(not args.empty()) args += ", ";
          args += start(arg);
        }
        return "gqlite_range({})" % args;
      } else if(_node->get_name() == "properties")
      {
        algebra::node_csp arg_0 = _node->get_arguments()[0];
        algebra::expression_type type_0 = eval_c->expression_analyser.start(arg_0).type;
        if(type_0 == algebra::expression_type::map)
        {
          return start(arg_0);
        } else {
          errors::check_condition(arg_0->get_type() == algebra::node_type::variable, exception_stage::runtime, exception_code::internal_error, "Only variable are supported yet for getting properties for node/edge.");
          return eval_c->get_sql_properties_variable(std::static_pointer_cast<const algebra::variable>(arg_0)->get_identifier());
        }
      } else if(_node->get_name() == "keys")
      {
        algebra::node_csp arg_0 = _node->get_arguments()[0];
        algebra::expression_type type_0 = eval_c->expression_analyser.start(arg_0).type;
        std::string map_expr;
        if(type_0 == algebra::expression_type::map)
        {
          map_expr = start(arg_0);
        } else {
          errors::check_condition(arg_0->get_type() == algebra::node_type::variable, exception_stage::runtime, exception_code::internal_error, "Only variable are supported yet for getting properties for node/edge.");
          map_expr = eval_c->get_sql_properties_variable(std::static_pointer_cast<const algebra::variable>(arg_0)->get_identifier());
        }
        return "(SELECT json_group_array(key) FROM json_each({}))" % map_expr;
      } else if(_node->get_name() == "toInteger")
      {
        return "CAST({} AS INTEGER)"s % start(_node->get_arguments()[0]);
      } else if(_node->get_name() == "size")
      {
        return "json_array_length({})"s % start(_node->get_arguments()[0]);
      } else if(_node->get_name() == "id")
      {
        return start(_node->get_arguments()[0]);
      } else if(_node->get_name() == "head")
      {
        return "json_extract({}, '$[0]')" % start(_node->get_arguments()[0]);
      } else if(_node->get_name() == "tail")
      {
        return "gqlite_tail({})" % start(_node->get_arguments()[0]);
      }
      throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Function call to {} is not implemented.", _node->get_name());
    }
#define FILTER_VISITOR_BINARY_OP(_AL_, _OP_)                                                          \
    std::string visit(algebra::_AL_ ## _csp _node) override                                           \
    {                                                                                                 \
      return "({} " _OP_ " {})"s % start(_node->get_left()), start(_node->get_right());               \
    }
    FILTER_VISITOR_BINARY_OP(logical_and, "AND")
    FILTER_VISITOR_BINARY_OP(logical_or, "OR")
    FILTER_VISITOR_BINARY_OP(logical_xor, "<>")
    FILTER_VISITOR_BINARY_OP(relational_equal, "=")
    FILTER_VISITOR_BINARY_OP(relational_different, "!=")
    FILTER_VISITOR_BINARY_OP(relational_inferior, "<")
    FILTER_VISITOR_BINARY_OP(relational_superior, ">")
    FILTER_VISITOR_BINARY_OP(relational_inferior_equal, "<=")
    FILTER_VISITOR_BINARY_OP(relational_superior_equal, ">=")
    FILTER_VISITOR_BINARY_OP(substraction, "-")
    FILTER_VISITOR_BINARY_OP(multiplication, "*")
    FILTER_VISITOR_BINARY_OP(division, "/")
    FILTER_VISITOR_BINARY_OP(modulo, "%")

    std::string visit(algebra::relational_in_csp _node) override
    {
      return "gqlite_contains({}, {})" % start(_node->get_right()), start(_node->get_left());
    }
    std::string visit(algebra::relational_not_in_csp _node) override
    {
      return "NOT gqlite_contains({}, {})" % start(_node->get_right()), start(_node->get_left());
    }

    std::string visit(algebra::addition_csp _node) override
    {
      algebra::expression_type left_type = eval_c->expression_analyser.start(_node->get_left()).type;
      algebra::expression_type right_type = eval_c->expression_analyser.start(_node->get_right()).type;

      if(left_type == algebra::expression_type::vector or right_type == algebra::expression_type::vector)
      {
        errors::check_argument_types(left_type, {algebra::expression_type::vector, algebra::expression_type::value}, exception_stage::runtime, "Invalid argument for array concatenation.");

        return "gqlite_concat({}, {})"s % start(_node->get_left()), start(_node->get_right());
      } else if(left_type == algebra::expression_type::string or right_type == algebra::expression_type::string)
      {
        errors::check_argument_types(left_type, {algebra::expression_type::string, algebra::expression_type::value}, exception_stage::runtime, "Invalid argument for string concatenation.");
        errors::check_argument_types(right_type, {algebra::expression_type::string, algebra::expression_type::value}, exception_stage::runtime, "Invalid argument for string concatenation.");

        return "({} || {})"s % start(_node->get_left()), start(_node->get_right());
      } else if(is_number(left_type) or is_number(right_type))
      {
        return "({} + {})"s % start(_node->get_left()), start(_node->get_right());
      } else {
        return "gqlite_addition({}, {})"s % start(_node->get_left()), start(_node->get_right());
      }
    }

#define FILTER_VISITOR_UNARY_OP(_AL_, _OP_)                             \
    std::string visit(algebra::_AL_ ## _csp _node) override             \
    {                                                                   \
      return format_string("(" _OP_ " {})", start(_node->get_value())); \
    }
    FILTER_VISITOR_UNARY_OP(logical_negation, "NOT")
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
  struct sql_match_evaluation_context : public sql_evaluation_context
  {
    std::vector<std::pair<std::string, std::string>> edge_sql_var_to_oc_var;
    /**
     * Generate the SQL join operation needed for matchin labels
     */
    std::string generate_labels_match(const std::vector<std::string>& _labels, const std::string& _node_variable)
    {
      std::string cond;
      for(const std::string& label : _labels)
      {
        std::string tb_lab = query_builder.add_table(format_string("gqlite_{}_labels", exec_c->graph_name), sql_join::inner, "tb_lab{}");
        std::string label_binding = query_builder.bind_value(exec_c->data->id_for_label(label));
        if(not cond.empty()) cond += " AND ";
        cond += format_string(" {} = {}.node_id AND {} = {}.label ", _node_variable, tb_lab, label_binding, tb_lab);
      }
      return cond;
    }
  };
  struct sql_match_expression_visitor : public sql_expression_visitor
  {
    sql_match_evaluation_context* eval_match_c;
    sql_match_expression_visitor(sql_match_evaluation_context* _ec) : sql_expression_visitor(_ec), eval_match_c(_ec)
    {}
    std::string visit_default(algebra::node_csp _node) override
    {
      throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Unimplemented node '{}' in sql_expression_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    std::string visit(algebra::has_labels_csp _node) override
    {
      return eval_match_c->generate_labels_match(_node->get_labels(), eval_c->get_sql_variable(_node->get_left()));
    }
  };
  /**
   * Visitor to execute statements: CREATE, MATCHES, RETURN...
   */
  struct statement_visitor : public gqlite::oc::algebra::default_node_visitor<value>
  {
    execution_context exec_c;
    table current_table;
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Common functions
    value visit_default(algebra::node_csp _node) override
    {
      throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Unimplemented statement node '{}' in statement_visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Creation functions
    value_vector create_node(algebra::graph_node_csp _node)
    {
      sql_evaluation_context mc;
      mc.exec_c = &exec_c;
      mc.prepare(current_table, false);
      sql_expression_visitor sev{&mc};
      mc.query_builder.add_variable("gqlite_next_uid('nodes')");

      std::string properties_str;
      if(_node->get_properties())
      {
        properties_str = sev.start(_node->get_properties());
      } else {
        properties_str = "'{}'";
      }
      mc.query_builder.add_variable(properties_str);
      std::string query = sqlite_queries::node_create(
        exec_c.graph_name, sev.eval_c->query_builder.assemble());
      value res = exec_c.data->execute_sql(query, sev.eval_c->query_builder.bindings);
      value_vector ret_v;
      value_vector res_v = res.to_vector();
      for(std::size_t i = 0; i < res_v.size(); ++i)
      {
        ret_v.push_back(res_v[i].to_vector()[0]);
      }
      // Add labels
      if(not _node->get_labels().empty())
      {
        value_vector label_ids;
        for(const std::string& label : _node->get_labels())
        {
          label_ids.push_back(exec_c.data->id_for_label(label));
        }
        std::map<int, value> bindings;
        bindings[1] = label_ids;
        bindings[2] = ret_v;
        exec_c.data->execute_sql(sqlite_queries::node_add_label(exec_c.graph_name), bindings);
      }
      return ret_v;
    }
    std::tuple<value, bool, std::string> create_node_if_needed(sql_expression_visitor* _sev, algebra::graph_node_csp _node, const std::map<std::string, value_vector>& _new_nodes, const char* _table_name_format/*, const std::string& _previous_table*/)
    {
      auto it_var = _sev->eval_c->variables.find(_node->get_variable());
      if(it_var == _sev->eval_c->variables.end())
      {
        auto it = _new_nodes.find(_node->get_variable());
        gqlite::value node_values;
        bool new_nodes;
        if(it == _new_nodes.end())
        {
          node_values = create_node(_node);
          new_nodes = true;
        } else {
          node_values = it->second;
          new_nodes = false;
        }
        std::string binding_idx = _sev->eval_c->query_builder.bind_value(node_values);
        std::string table_name_ref = _sev->eval_c->query_builder.add_table(format_string("json_each({})", binding_idx), sql_join::inner, _table_name_format);
        if(not _sev->eval_c->previous_table_ref.empty())
        {
          _sev->eval_c->query_builder.add_join_condition("{}.key + 1 = {}.rowid", table_name_ref, _sev->eval_c->previous_table_ref);
        }
        return std::make_tuple(node_values, new_nodes, format_string("{}.value", table_name_ref));
      } else {
        return std::make_tuple(value(), false, it_var->second.sql_var_name);
      }
    }
    std::tuple<value_vector, value_vector, value_vector> create_edge(algebra::graph_edge_csp _edge, const std::map<std::string, value_vector>& _new_nodes)
    {
      sql_evaluation_context mc;
      mc.exec_c = &exec_c;
      mc.prepare(current_table, false);
      sql_expression_visitor sev{&mc};
      mc.query_builder.add_variable("gqlite_next_uid('edges')");
      // Handle label
      std::string label = _edge->get_labels().empty() ? std::string() : _edge->get_labels().front();
      std::string label_binding_idx = sev.eval_c->query_builder.bind_value(exec_c.data->id_for_label(label));
      mc.query_builder.add_variable(label_binding_idx, "label_{}");

      // Handle properties
      std::string properties_str;
      if(_edge->get_properties())
      {
        properties_str = sev.start(_edge->get_properties());
      } else {
        properties_str = "'{}'";
      }
      mc.query_builder.add_variable(properties_str, "properties_{}");

      // Handle source
      auto const&[left_value, new_left_value, left_expr] = create_node_if_needed(&sev, _edge->get_source(), _new_nodes, "source_node_{}");
      mc.query_builder.add_variable(left_expr, "source_{}");

      // Check if destination == source
      auto const&[right_value, new_right_value, right_expr]
        = (_edge->get_source()->get_variable() == _edge->get_destination()->get_variable()) ?
            std::make_tuple(gqlite::value(), false, left_expr)
          : create_node_if_needed(&sev, _edge->get_destination(), _new_nodes, "destination_node_{}");
      mc.query_builder.add_variable(right_expr, "destination_{}");

      // Execute queries
      std::string query = sqlite_queries::edge_create(exec_c.graph_name, sev.eval_c->query_builder.assemble());
      value res = sev.eval_c->exec_c->data->execute_sql(query, sev.eval_c->query_builder.bindings);

      // Prepare return
      value_vector ret_v;
      value_vector res_v = res.to_vector();
      for(std::size_t i = 0; i < res_v.size(); ++i)
      {
        ret_v.push_back(res_v[i].to_vector()[0]);
      }

      // Check if nodes were created during the edge cretion
      value_vector left_value_ret; if(new_left_value) left_value_ret = left_value.to_vector();
      value_vector right_value_ret; if(new_right_value) right_value_ret = right_value.to_vector();
      return {left_value_ret, ret_v, right_value_ret};
    }
    /**
     * Execute the create statement
     */
    value visit(algebra::create_csp _node) override
    {
      // 1) Create new nodes/edgers
      std::map<std::string, value_vector> new_nodes, new_edges;
      bool new_non_anonymous_nodes = false;
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        pattern.visit<void>([this, &new_nodes, &new_non_anonymous_nodes](const algebra::graph_node_csp _node)
        {
          gqlite::value_vector nodes = create_node(_node);
          if(not _node->get_variable().starts_with("__gqlite_anon_"))
          {
            new_nodes[_node->get_variable()] = nodes;
            new_non_anonymous_nodes = new_non_anonymous_nodes or (not _node->get_variable().starts_with("__gqlite_anon_"));
          }
        },
        [this, &new_nodes, &new_edges, &new_non_anonymous_nodes](const algebra::graph_edge_csp _edge)
        {
          auto const& [left_nodes, edges, right_nodes] = create_edge(_edge, new_nodes);
          if(not left_nodes.empty())
          {
            new_nodes[_edge->get_source()->get_variable()] = left_nodes;
            new_non_anonymous_nodes = new_non_anonymous_nodes or (not _edge->get_source()->get_variable().starts_with("__gqlite_anon_"));
          }
          if(not right_nodes.empty())
          {
            new_nodes[_edge->get_destination()->get_variable()] = right_nodes;
            new_non_anonymous_nodes = new_non_anonymous_nodes or (not _edge->get_destination()->get_variable().starts_with("__gqlite_anon_"));
          }
          new_edges[_edge->get_variable()] = edges;
            new_non_anonymous_nodes = new_non_anonymous_nodes or (not _edge->get_variable().starts_with("__gqlite_anon_"));

        });
      }

      if(new_non_anonymous_nodes)
      {
        // 2) Create the resulting table
        sql_evaluation_context mc;
        mc.exec_c = &exec_c;
        table new_table = mc.prepare(current_table, true);
        sql_expression_visitor sev{&mc};
        std::string first_table_ref;
        // loop over new things
        for(const auto& [new_things, type, var_format]
           : std::initializer_list<std::tuple<std::map<std::string, gqlite::value_vector>, oc::algebra::expression_type, const char*>>
             {{new_nodes, oc::algebra::expression_type::node, "new_node_{}"}, {new_edges, oc::algebra::expression_type::edge, "new_edge_{}"}})
        {
          for(const auto& [k, v] : new_things)
          {
            if(not k.starts_with("__gqlite_anon_"))
            {
              std::string idx = mc.query_builder.bind_value(v);
              std::string table_ref = mc.query_builder.add_table(format_string("json_each({})", idx), sql_join::inner, var_format);
              if(first_table_ref.empty())
              {
                first_table_ref = table_ref;
              } else {
                mc.query_builder.add_join_condition("{}.key = {}.key", first_table_ref, table_ref);
              }
              mc.query_builder.add_variable_with_name(format_string(" {}.value ", table_ref), table_ref);
              new_table.variables[k] = {table_ref, type};
            }
          }
        }
        if(not mc.previous_table_ref.empty())
        {
          mc.query_builder.add_join_condition("{}.rowid = {}.key + 1", mc.previous_table_ref, first_table_ref);
        }
        exec_c.data->execute_sql("CREATE TEMPORARY TABLE " + new_table.view_name + " AS " + mc.query_builder.assemble(), mc.query_builder.bindings);
        current_table = new_table;
      }
      return value{};
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Match
    /**
     * Generate a filter expression for a map
     */
    std::string generate_filter(const algebra::node_csp& _map, const std::string& _path, sql_expression_visitor* _filter_visitor)
    {
      switch(_map->get_type())
      {
        case algebra::node_type::map:
        {
          
          std::string r;
          algebra::map_csp m = std::static_pointer_cast<const algebra::map>(_map);
          for(const auto& [k, v] : m->get_map())
          {
            std::string path = _path + "." + k;
            if(not r.empty())
            {
              r += " AND ";
            }
            if(v->get_type() == algebra::node_type::map)
            {
              r += generate_filter(v, path, _filter_visitor);
            } else {
              r += format_string(" json_extract({}') = {} ", path, _filter_visitor->start(v));
            }
          }
          return r;
        }
        case algebra::node_type::value:
        {
          value val = std::static_pointer_cast<const algebra::value>(_map)->get_value();
          switch(val.get_type())
          {
          case value_type::map:
          {
            // special case for map, as this can't be an equal comparison, since for matching, only a subset of the key might be explicitly defined
            std::string r;
            for(const auto& [k, v] : val.to_map())
            {
              std::string path = _path + "." + k;
              if(not r.empty())
              {
                r += " AND ";
              }
              r += generate_filter(std::make_shared<algebra::value>(v), path, _filter_visitor); // TODO use a seperate function
            }
            return r;
          }
          case value_type::vector:
            {
              std::string id = _filter_visitor->eval_c->query_builder.bind_value(val.to_json());
              return (" json_extract({}') = json({}) "s % _path, id);
            }
          default:
            {
              std::string id = _filter_visitor->eval_c->query_builder.bind_value(val);
              return (" json_extract({}') = {} "s % _path, id);
            }
          }
        }
        default:
          throw_exception(exception_stage::runtime, exception_code::internal_error, "Properties is neither algebra::map nor value_map, this should have been caught by the parser.");
      }
    }
    void generate_var_match(sql_match_evaluation_context* _mc, table* _table, const std::string& _oc_var, const std::string& _sql_var, const std::string& _sql_properties_var, const std::string& _label_var, oc::algebra::expression_type _variable_type)
    {
      if(not _oc_var.empty())
      {
        auto it = _mc->variables.find(_oc_var);
        if(it == _mc->variables.end())
        {
          std::string var_name = _mc->query_builder.add_variable(_sql_var, "new_match_{}");
          _table->variables[_oc_var] = {var_name, _variable_type};
          _mc->variables[_oc_var] = {_sql_var, _variable_type, _sql_properties_var, _label_var};
        } else {
          errors::check_variable_type(it->second.type, _variable_type, exception_stage::runtime, _oc_var);
          // Generate match
          _mc->query_builder.add_join_condition(" {} = {} ", _sql_var, it->second.sql_var_name);
        }
      }
    }
    value visit(algebra::match_csp _node) override
    {
      sql_match_evaluation_context mc;
      mc.exec_c = &exec_c;
      table new_table = mc.prepare(current_table, true);
      sql_match_expression_visitor sev{&mc};

      // 1) Go through the patterns
      for(const algebra::alternative<algebra::graph_node, algebra::graph_edge>& pattern : _node->get_patterns())
      {
        pattern.visit<void>([this, &mc, &sev, &new_table](const algebra::graph_node_csp _node)
        {
          std::string table_ref = sev.eval_c->query_builder.add_table(format_string("gqlite_{}_nodes", exec_c.graph_name), sql_join::inner, "node_{}");
          std::string sql_var = format_string("{}.id", table_ref);
          std::string sql_properties_var = format_string("{}.properties", table_ref);
          mc.query_builder.add_join_condition(mc.generate_labels_match(_node->get_labels(), format_string("{}.id", table_ref)));
          if(_node->get_properties())
          {
            mc.query_builder.add_where_condition(generate_filter(_node->get_properties(), sql_properties_var + ", '$", &sev));
          }
          generate_var_match(&mc, &new_table, _node->get_variable(), sql_var, sql_properties_var, std::string(), algebra::expression_type::node);
        },
        [this, &mc, &sev, &new_table](const algebra::graph_edge_csp _edge)
        {
          std::string table_ref;
          if(_edge->get_directivity() == algebra::edge_directivity::undirected)
          {
            table_ref = sev.eval_c->query_builder.add_table(format_string("gqlite_{}_edges_undirected", exec_c.graph_name), sql_join::inner, "undirected_edge_{}");
          } else {
            table_ref = sev.eval_c->query_builder.add_table(format_string("gqlite_{}_edges", exec_c.graph_name), sql_join::inner, "edge_{}");
          }
          std::string sql_source_var = format_string("{}.left", table_ref);
          std::string sql_edge_var = format_string("{}.id", table_ref);
          std::string sql_properties_edge_var = format_string("{}.properties", table_ref);
          std::string sql_label_edge_var = format_string("{}.label", table_ref);
          std::string sql_source_properties_var;
          std::string sql_destination_properties_var;
          std::string sql_destination_var = format_string("{}.right", table_ref);
          mc.query_builder.add_where_condition(mc.generate_labels_match(_edge->get_source()->get_labels(), table_ref+ ".left"));
          mc.query_builder.add_where_condition(mc.generate_labels_match(_edge->get_destination()->get_labels(), table_ref + ".right"));
          if(not _edge->get_labels().empty())
          {
            std::string label_conds = "(FALSE ";
            for(const std::string& label : _edge->get_labels())
            {
              label_conds += format_string(" OR {} = {}", sql_label_edge_var, exec_c.data->id_for_label(label));
            }
            label_conds += ")";
            mc.query_builder.add_where_condition(label_conds);
          }
          if(_edge->get_properties())
          {
            mc.query_builder.add_where_condition(generate_filter(_edge->get_properties(), sql_properties_edge_var + ", '$", &sev));
          }
          if(_edge->get_source()->get_properties())
          {
            sql_source_properties_var = mc.retrieve_sql_properties(sql_source_var, algebra::expression_type::node);
            mc.query_builder.add_where_condition(generate_filter(_edge->get_source()->get_properties(), sql_source_properties_var + ", '$", &sev));
          }
          if(_edge->get_destination()->get_properties())
          {
            sql_destination_properties_var = mc.retrieve_sql_properties(sql_destination_var, algebra::expression_type::node);
            mc.query_builder.add_where_condition(generate_filter(_edge->get_destination()->get_properties(), sql_destination_properties_var + ", '$", &sev));
          }
          // Ensure identical oc variablle are joined in SQL
          generate_var_match(&mc, &new_table, _edge->get_source()->get_variable(), sql_source_var, sql_source_properties_var, std::string(), algebra::expression_type::node);
          generate_var_match(&mc, &new_table, _edge->get_variable(), sql_edge_var, sql_properties_edge_var, sql_label_edge_var, algebra::expression_type::edge);
          generate_var_match(&mc, &new_table, _edge->get_destination()->get_variable(), sql_destination_var, sql_destination_properties_var, std::string(), algebra::expression_type::node);
          // ensure edge isomorphism, i.e., if the variable names are different, the edges must be different
          for(const std::pair<std::string, std::string>& sql_var_to_oc_var : mc.edge_sql_var_to_oc_var)
          {
            if(sql_var_to_oc_var.second != _edge->get_variable() and not _edge->get_variable().empty())
            {
              mc.query_builder.add_where_condition(" {} != {} ", sql_edge_var, sql_var_to_oc_var.first);
            }
          }
          mc.edge_sql_var_to_oc_var.push_back({sql_edge_var, _edge->get_variable()});
          // Increase variable counter
        });
      }
      // 2) Handle where
      if(_node->get_where())
      {
        mc.query_builder.add_where_condition(sev.start(_node->get_where()));
      }
      // 3) Assemble SQL query for execution
      std::string sql_query = "CREATE TEMPORARY TABLE " + new_table.view_name + " AS " + mc.query_builder.assemble();

      exec_c.data->execute_sql(sql_query, mc.query_builder.bindings);
      current_table = new_table;

      return value{};
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // delete
    value visit(algebra::delete_statement_csp _ds) override
    {
      for(algebra::node_csp node : _ds->get_expressions())
      {
        sql_match_evaluation_context mc;
        mc.exec_c = &exec_c;
        mc.prepare(current_table, false);
        sql_match_expression_visitor sev{&mc};
        std::string expr = sev.start(node);
        mc.query_builder.add_variable(expr);
        std::string what = mc.query_builder.assemble();

        switch(mc.expression_analyser.start(node).type)
        {
          case algebra::expression_type::node:
          {
            if(_ds->get_detach())
            {
              std::string query = sqlite_queries::edge_delete_by_node(exec_c.graph_name, what);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
            }
            std::string query = sqlite_queries::node_delete(exec_c.graph_name, what);
            mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
            break;
          }
          case algebra::expression_type::edge:
          {
            std::string query = sqlite_queries::edge_delete(exec_c.graph_name, what);
            mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
            break;
          }
          default:
            errors::invalid_argument_type(exception_stage::runtime, "Delete expect a node/edge.");
        }
      }
      return value();
      #if 0
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
            errors::check_condition(rows.size() == 1, exception_stage::runtime, exception_code::internal_error, "Invalid number of rows for counting edges got {} expected 1.", rows.size());
            value_vector row = rows.front().to_vector();
            errors::check_condition(row.size() == 1, exception_stage::runtime, exception_code::internal_error, "Invalid number of columns for counting edges got {} expected 1.", rows.size());
            int count = row.front().to_integer();
            errors::check_condition(count == 0, exception_stage::runtime, exception_code::unspecified, "Cannot delete node with {} relationships.", count);
          }
          self->exec_c.data->execute_sql(sqlite_queries::node_delete(self->exec_c.graph_name), {{1, _node->id}});
        }
        void operator()(const edge_ref_sp& _edge)
        {
          self->exec_c.data->execute_sql(sqlite_queries::edge_delete(self->exec_c.graph_name), {{1, _edge->id}});
        }
        void operator()(const value&)
        {
          throw_exception(exception_stage::runtime, exception_code::invalid_argument_type, "Cannot delete a value.");
        }
        void operator()(const empty&)
        {
          throw_exception(exception_stage::runtime, exception_code::invalid_argument_type, "Cannot delete an empty value.");
        }
      };

      for(std::size_t i = 0; i < table.get_rows_count(); ++i)
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
      #endif
    }
    std::string to_sqlite_path(const std::vector<std::string>& _path)
    {
      std::string path_string = "$";
      for(const std::string& pe : _path)
      {
        path_string += "." + pe;
      }
      return path_string;
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // set
    value visit(algebra::set_csp _set) override
    {
      for(algebra::node_csp node : _set->get_nodes())
      {
        struct set_visitor : public gqlite::oc::algebra::default_node_visitor<void>
        {
          statement_visitor* self;
          void visit_default(algebra::node_csp _node) override
          {
            throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Unimplemented node {} in set_visitor", oc::algebra::node_type_name(_node->get_type()));
          }
          void visit(algebra::set_property_csp _property)
          {
            sql_match_evaluation_context mc;
            mc.exec_c = &self->exec_c;
            mc.prepare(self->current_table, false);
            sql_match_expression_visitor sev{&mc};
            sql_variable_info svi = mc.get_variable_info(_property->get_left());
            // mc.query_builder.add_variable(svi.sql_var_name);
            std::string expr = sev.start(_property->get_expression());
            std::string what = mc.query_builder.assemble().substr(sizeof("SELECT null"));

            switch(svi.type)
            {
            case algebra::expression_type::node:
            {
              std::string query = sqlite_queries::node_set_property(self->exec_c.graph_name, what, expr, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            case algebra::expression_type::edge:
            {
              std::string query = sqlite_queries::edge_set_property(self->exec_c.graph_name, what, expr, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            default:
              errors::invalid_argument_type(exception_stage::runtime, "Delete expect a node/edge.");
            }
          }
          void visit(algebra::add_property_csp _property)
          {
            sql_match_evaluation_context mc;
            mc.exec_c = &self->exec_c;
            mc.prepare(self->current_table, false);
            sql_match_expression_visitor sev{&mc};
            sql_variable_info svi = mc.get_variable_info(_property->get_left());
            std::string expr = sev.start(_property->get_expression());
            std::string what = mc.query_builder.assemble().substr(sizeof("SELECT null"));

            switch(svi.type)
            {
            case algebra::expression_type::node:
            {
              std::string query = sqlite_queries::node_add_properties(self->exec_c.graph_name, what, expr, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            case algebra::expression_type::edge:
            {
              std::string query = sqlite_queries::edge_add_properties(self->exec_c.graph_name, what, expr, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            default:
              errors::invalid_argument_type(exception_stage::runtime, "Delete expect a node/edge.");
            }
          }
          void visit(algebra::edit_labels_csp _edit_label)
          {
            sql_match_evaluation_context mc;
            mc.exec_c = &self->exec_c;
            mc.prepare(self->current_table, false);
            sql_match_expression_visitor sev{&mc};
            sql_variable_info svi = mc.get_variable_info(_edit_label->get_target());
            std::vector<std::string> labels = _edit_label->get_labels();
            auto label_ids_view = labels | std::views::transform([this](const std::string& _label) { return value(self->exec_c.data->id_for_label(_label)); });
            std::string labels_ref = mc.query_builder.add_table("json_each({})"s % mc.query_builder.bind_value(value_vector(label_ids_view.begin(), label_ids_view.end())), sql_join::inner);
            mc.query_builder.add_variable("{}.value" % labels_ref);
            mc.query_builder.add_variable(svi.sql_var_name);
            std::string what = mc.query_builder.assemble();

            switch(svi.type)
            {
            case algebra::expression_type::node:
            {
              std::string query = sqlite_queries::node_add_labels(self->exec_c.graph_name, what);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            default:
              errors::invalid_argument_type(exception_stage::runtime, "Can only add label to a node.");
            }
          }
        };
        set_visitor s_v; s_v.self = this;
        s_v.start(node);
      }
      return value();

    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // remove
    value visit(algebra::remove_csp _set) override
    {
      for(algebra::node_csp node : _set->get_nodes())
      {
        struct remove_visitor : public gqlite::oc::algebra::default_node_visitor<void>
        {
          statement_visitor* self;
          void visit_default(algebra::node_csp _node) override
          {
            throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Unimplemented node {} in remove_visitor", oc::algebra::node_type_name(_node->get_type()));
          }
          void visit(algebra::remove_property_csp _property)
          {
            sql_match_evaluation_context mc;
            mc.exec_c = &self->exec_c;
            mc.prepare(self->current_table, false);
            sql_match_expression_visitor sev{&mc};
            sql_variable_info svi = mc.get_variable_info(_property->get_left());
            std::string what = mc.query_builder.assemble().substr(sizeof("SELECT null"));

            switch(svi.type)
            {
            case algebra::expression_type::node:
            {
              std::string query = sqlite_queries::node_remove_property(self->exec_c.graph_name, what, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            case algebra::expression_type::edge:
            {
              std::string query = sqlite_queries::edge_remove_property(self->exec_c.graph_name, what, self->to_sqlite_path(_property->get_path()), svi.sql_var_name);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            default:
              errors::invalid_argument_type(exception_stage::runtime, "Delete expect a node/edge.");
            }
          }
          void visit(algebra::edit_labels_csp _edit_label)
          {
            sql_match_evaluation_context mc;
            mc.exec_c = &self->exec_c;
            mc.prepare(self->current_table, false);
            sql_match_expression_visitor sev{&mc};
            sql_variable_info svi = mc.get_variable_info(_edit_label->get_target());
            std::vector<std::string> labels = _edit_label->get_labels();
            auto label_ids_view = labels | std::views::transform([this](const std::string& _label) { return std::to_string(self->exec_c.data->id_for_label(_label)); });
            std::string labels_ref = string::join(label_ids_view, ", ");
            mc.query_builder.add_variable(svi.sql_var_name);
            std::string what = mc.query_builder.assemble();

            switch(svi.type)
            {
            case algebra::expression_type::node:
            {
              std::string query = sqlite_queries::node_remove_label(self->exec_c.graph_name, what, labels_ref);
              mc.exec_c->data->execute_sql(query, mc.query_builder.bindings);
              break;
            }
            default:
              errors::invalid_argument_type(exception_stage::runtime, "Can only add label to a node.");
            }
          }
        };
        remove_visitor s_v; s_v.self = this;
        s_v.start(node);
      }
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // add_filter_expressions table
    void add_filter_expressions(sql_expression_visitor* _expr_v, algebra::modifiers_csp _modifiers)
    {
      if(_modifiers)
      {
        if(_modifiers->get_order_by())
        {
          std::string order_by = " ORDER BY ";
          bool first = true;
          for(algebra::order_by_expression_csp node : _modifiers->get_order_by()->get_expressions())
          {
            if(not first) order_by += ", ";
            first = false;
            order_by += _expr_v->start(node->get_expression()) + (node->get_asc() ? " ASC " : " DESC ");
          }
          _expr_v->eval_c->query_builder.add_modifier(order_by);
        }
        if(_modifiers->get_limit())
        {
          _expr_v->eval_c->query_builder.add_modifier(" LIMIT {} " % _expr_v->start(_modifiers->get_limit()));
        }
        if(_modifiers->get_skip())
        {
          if(not _modifiers->get_limit()) _expr_v->eval_c->query_builder.add_modifier(" LIMIT -1 ");
          _expr_v->eval_c->query_builder.add_modifier(" OFFSET {} " % _expr_v->start(_modifiers->get_skip()));
        }
      }
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
        throw_exception(exception_stage::runtime, exception_code::unspecified, "Unimplemented WITH *, expressions");
      }

      sql_evaluation_context mc;
      mc.exec_c = &exec_c;
      table new_table = mc.prepare(current_table, true);
      new_table.variables.clear();
      sql_expression_visitor sev{&mc};
      // Generate SQL Query
      for(const algebra::named_expression_csp& rv : w->get_expressions())
      {
        std::string var = mc.query_builder.add_variable(sev.start(rv->get_expression()));
        algebra::expression_type et = mc.expression_analyser.start(rv->get_expression()).type; 
        new_table.variables[rv->get_name()] = {var, et};
        mc.variables[rv->get_name()] = {var, et};
      }
      add_filter_expressions(&sev, w->get_modifiers());
      std::string query = "CREATE TEMPORARY TABLE " + new_table.view_name + " AS " + mc.query_builder.assemble();
      exec_c.data->execute_sql(query, mc.query_builder.bindings);

      current_table = new_table;
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // unwind
    value visit(algebra::unwind_csp uw) override
    {
      sql_evaluation_context mc;
      mc.exec_c = &exec_c;
      table new_table = mc.prepare(current_table, true);
      sql_expression_visitor sev{&mc};
      std::string table_ref = mc.query_builder.add_table("json_each(" + sev.start(uw->get_expression()) + ")", sql_join::inner, "a_uw_{}");
      std::string var = mc.query_builder.add_variable(format_string("{}.value", table_ref));
      std::string query = "CREATE TEMPORARY TABLE " + new_table.view_name + " AS " + mc.query_builder.assemble();
      new_table.variables[uw->get_name()] = {var, oc::algebra::expression_type::value};
      exec_c.data->execute_sql(query, mc.query_builder.bindings);
      current_table = new_table;
      return value();
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Return
    value visit(algebra::return_statement_csp _rs) override
    {
      sql_evaluation_context mc;
      mc.exec_c = &exec_c;
      mc.prepare(current_table, false);
      sql_expression_visitor sev{&mc};
      std::vector<gqlite::oc::algebra::named_expression_csp> expressions;
      if(_rs->get_all())
      {
        for(const auto& [k, var_info] : current_table.variables)
        {
          if(not k.starts_with("__gqlite_anon_"))
          {
            expressions.push_back(std::make_shared<algebra::named_expression>(k, std::make_shared<algebra::variable>(k)));
          }
        }
      } else {
        expressions = _rs->get_expressions();
      }
      // Generate SQL Query
      value_vector labels;
      std::vector<oc::algebra::expression_type> column_types;
      std::map<std::string, sql_variable_info> variables_extra; /// new variables to set for modifier

      for(const algebra::named_expression_csp& rv : expressions)
      {
        oc::algebra::expression_type et = mc.expression_analyser.start(rv->get_expression()).type;
        switch(et)
        {
        case oc::algebra::expression_type::node:
          {
            std::string table_ref = mc.query_builder.add_table(format_string("gqlite_{}_nodes_as_json", exec_c.graph_name), sql_join::inner, "node_{}");
            mc.query_builder.add_join_condition("{}.id = {}", table_ref, sev.start(rv->get_expression()));
            std::string var_expr = mc.query_builder.add_variable("{}.node" % table_ref);
            variables_extra[rv->get_name()] = {var_expr, oc::algebra::expression_type::node,
                                  "json_extract({}, '$.properties')" % var_expr, "json_extract({}, '$.labels')" % var_expr};
            break;
          }
        case oc::algebra::expression_type::edge:
          {
            std::string table_ref = mc.query_builder.add_table(format_string("gqlite_{}_edges_as_json", exec_c.graph_name), sql_join::inner, "edge_{}");
            mc.query_builder.add_join_condition("{}.id = {}", table_ref, sev.start(rv->get_expression()));
            std::string var_expr = mc.query_builder.add_variable("{}.edge" % table_ref);
            variables_extra[rv->get_name()] = {var_expr, oc::algebra::expression_type::edge,
                                    "json_extract({}, '$.properties')" % var_expr, "json_extract({}, '$.label')" % var_expr};
            break;
          }
        case oc::algebra::expression_type::boolean:
        case oc::algebra::expression_type::vector:
        case oc::algebra::expression_type::map:
          [[fallthrough]];
        default:
          {
            std::string var_expr = mc.query_builder.add_variable(sev.start(rv->get_expression()));
            variables_extra[rv->get_name()] = {var_expr, et};
            break;
          }
        }
        labels.push_back(rv->get_name());
        column_types.push_back(et);
      }
      // Execute SQL query
      if(_rs->get_modifiers())
      {
        mc.variables.insert(variables_extra.begin(), variables_extra.end());
        add_filter_expressions(&sev, _rs->get_modifiers());
      }
      value res = exec_c.data->execute_sql(mc.query_builder.assemble(), mc.query_builder.bindings);
      // Prepare to return results
      value_vector res_v = res.to_vector();
      value_vector ret_v;
      ret_v.push_back(labels);
      for(const value& r : res_v)
      {
        value_vector r_v = r.to_vector();
        for(std::size_t c = 0; c < r_v.size(); ++c)
        {
          value& v = r_v[c];
          switch(column_types[c])
          {
          case oc::algebra::expression_type::boolean:
            if(v.get_type() != value_type::invalid)
            {
              v = (v.to_integer() == 1);
            }
            break;
          case oc::algebra::expression_type::map:
          case oc::algebra::expression_type::vector:
            v = value::from_json(v.to_string());
            break;
          case oc::algebra::expression_type::node:
          case oc::algebra::expression_type::edge:
            v = value::from_json(v.to_string());
            break;
          default:
            if(v.get_type() == value_type::string)
            {
              // Attempt to parse strings as json
              try
              {
                v = value::from_json(v.to_string());
              }
              catch(const exception&)
              {
                // ignore, that probably means it is a real string
              }
              break;
            }
          }
        }
        ret_v.push_back(r_v);
      }
      return ret_v;
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Call
    value visit(algebra::call_csp rs) override
    {
      if(rs->get_arguments().size() != 0) throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Arguments to call are not supported");
      if(rs->get_yield().size() != 0) throw_exception(exception_stage::runtime, exception_code::unimplemented_error, "Yields to call are not supported");
      auto it = exec_c.data->procedures.find(rs->get_name());
      if(it == exec_c.data->procedures.end())
      {
        throw_exception(exception_stage::runtime, exception_code::unknown_function, "No procedure called {}", rs->get_name());
      }
      return it->second(value_vector());
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Statements
    value visit(algebra::statements_csp _node) override
    {
      for(algebra::node_csp node : _node->get_nodes())
      {
        value val = start(node);
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
  if(not d->table_has("gqlite_uids"))
  {
    d->execute_sql(sqlite_queries::uid_create_table());
  }
  if(not d->graph_has("default"))
  {
    d->graph_create("default");
  }

  d->procedures["gqlite.internal.stats"] = [this](const value_vector&)
  {
    gqlite::value result = d->execute_sql(sqlite_queries::get_debug_stats("default"));
    value_vector rows = result.to_vector();
    errors::check_condition(rows.size() == 7, exception_stage::runtime, exception_code::internal_error, "Invalid number of debug stats got {} expected 7.", rows.size());
    value_map stats;
    stats["nodes_count"] = rows[0].to_vector()[0].to_integer();
    stats["edges_count"] = rows[1].to_vector()[0].to_integer();
    stats["labels_assignment_count"] = rows[2].to_vector()[0].to_integer();
    stats["properties_count"] = rows[3].to_vector()[0].to_integer() + rows[4].to_vector()[0].to_integer();
    stats["labels_count"] = rows[5].to_vector()[0].to_integer();
    stats["labels_assignment_nodes_count"] = rows[6].to_vector()[0].to_integer();
    return stats;
  };
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
  initialise_sqlite_ext(handle);
  return new sqlite(handle);
}

gqlite::value sqlite::execute_oc_query(oc::algebra::node_csp _node)
{
  d->execute_sql("BEGIN");
  try
  {
    sqlite_oc_executor::statement_visitor executor;
    executor.exec_c.data = d;
    value val = executor.start(_node);
    d->execute_sql("COMMIT");
    d->clean_up();
    return val;
  } catch(const exception& _ex)
  {
    d->execute_sql("ROLLBACK");
    d->clean_up();
    rethrow_exception(exception_stage::runtime, _ex);
  }
}
