#include "../default_node_visitor.h"

#include <ranges>
#include <unordered_map>

#include "../../../errors.h"
#include "../../../logging.h"

namespace gqlite::oc::algebra::visitors
{
  using expression_type = gqlite::oc::algebra::expression_type;
  /**
   * This class analyse an algebra to get expression types and wether they are constant expressions.
   */
  struct expression_analyser : public gqlite::oc::algebra::default_node_visitor<expression_info>
  {
    exception_stage stage;
    std::function<expression_type(const std::string&)> variables_f;
    expression_info visit_default(algebra::node_csp _node) override
    {
      throw_exception(stage, exception_code::unimplemented_error, "Unimplemented node {} in expression_analyser visitor", oc::algebra::node_type_name(_node->get_type()));
    }
    expression_info visit(algebra::value_csp _var) override
    {
      switch(_var->get_value().get_type())
      {
        case value_type::invalid:
          return {expression_type::empty, true };
        case value_type::boolean:
          return {expression_type::boolean, true };
        case value_type::integer:
          return {expression_type::integer, true };
        case value_type::floating_point:
          return {expression_type::floating_point, true };
        case value_type::string:
          return {expression_type::string, true };
        case value_type::map:
          return {expression_type::map, true };
        case value_type::vector:
          return {expression_type::vector, true };
      };
      throw_exception(exception_stage::unspecified, exception_code::internal_error, "Unknown value type.");
    }
    template<typename _T_>
    bool constant_nodes(const _T_& _nodes)
    {
      for(algebra::node_csp n : _nodes)
      {
        if(not start(n).constant)
        {
          return false;
        }
      }
      return true;
    }
    expression_info visit(algebra::array_csp _value) override
    {
      return {expression_type::vector, constant_nodes(_value->get_array())};
    }
    expression_info visit(algebra::map_csp _value) override
    {
      return {expression_type::map, constant_nodes(workarounds::views::values(_value->get_map()))};
    }
    expression_info visit(algebra::member_access_csp _ma) override
    {
      return {expression_type::value, start(_ma->get_left()).constant};
    }
    expression_info visit(algebra::indexed_access_csp _ia) override
    {
      return {expression_type::value, start(_ia->get_left()).constant};
    }
    expression_info visit(algebra::has_labels_csp) override
    {
      return {expression_type::boolean, false};
    }
#define GQLITE_ET_LOGICAL_OP(_NAME_)                              \
    expression_info visit(algebra::_NAME_ ## _csp _node) override \
    {                                                             \
      bool constant = start(_node->get_left()).constant           \
                  and start(_node->get_right()).constant;         \
      return {expression_type::boolean, constant};                \
    }
    GQLITE_ET_LOGICAL_OP(logical_and)
    GQLITE_ET_LOGICAL_OP(logical_or)
    GQLITE_ET_LOGICAL_OP(logical_xor)
    GQLITE_ET_LOGICAL_OP(relational_equal)
    GQLITE_ET_LOGICAL_OP(relational_different)
    GQLITE_ET_LOGICAL_OP(relational_inferior)
    GQLITE_ET_LOGICAL_OP(relational_superior)
    GQLITE_ET_LOGICAL_OP(relational_inferior_equal)
    GQLITE_ET_LOGICAL_OP(relational_superior_equal)
    GQLITE_ET_LOGICAL_OP(relational_in)
    GQLITE_ET_LOGICAL_OP(relational_not_in)
#define GQLITE_ET_LOGICAL_UNARY_OP(_NAME_)                          \
    expression_info visit(algebra::_NAME_ ## _csp _node) override   \
    {                                                               \
      bool constant = start(_node->get_value()).constant;           \
      return {expression_type::boolean, constant};                  \
    }
    GQLITE_ET_LOGICAL_UNARY_OP(is_null)
    GQLITE_ET_LOGICAL_UNARY_OP(is_not_null)
#define GQLITE_ET_BINARY_OP(_NAME_)                                                                       \
    expression_info visit(algebra::_NAME_ ## _csp _node) override                                         \
    {                                                                                                     \
      expression_info left = start(_node->get_left());                                                    \
      expression_info right = start(_node->get_right());                                                  \
      bool constant = left.constant and right.constant;                                                   \
      if(left.type == right.type) return {left.type, constant};                                           \
      if(left.type == expression_type::value or right.type == expression_type::value)                     \
        return {expression_type::value, constant};                                                        \
      if(left.type == expression_type::floating_point or right.type == expression_type::floating_point)   \
      {                                                                                                   \
        return {expression_type::floating_point, constant};                                               \
      }                                                                                                   \
      if(left.type == expression_type::string or right.type == expression_type::string)                   \
      {                                                                                                   \
        return {expression_type::string, constant};                                                       \
      }                                                                                                   \
      if(std::is_same_v<algebra::_NAME_ ## _csp, algebra::addition_csp>                                   \
            and left.type == expression_type::vector)                                                     \
      {                                                                                                   \
        return {expression_type::vector, constant};                                                       \
      }                                                                                                   \
      errors::invalid_argument_type(stage, "In binary operation.");                                       \
    }
    GQLITE_ET_BINARY_OP(addition)
    GQLITE_ET_BINARY_OP(substraction)
    GQLITE_ET_BINARY_OP(multiplication)
    GQLITE_ET_BINARY_OP(division)
    GQLITE_ET_BINARY_OP(modulo)

    expression_info visit(algebra::logical_negation_csp _node)
    {
      bool constant = start(_node->get_value()).constant;
      return {expression_type::boolean, constant};
    }
    expression_info visit(algebra::variable_csp _var) override
    {
      return {variables_f(_var->get_identifier()), false};
    }
    struct function_info
    {
      expression_type return_type;
      std::vector<expression_type> arguments_types;
      bool deterministic;
    };
    static std::unordered_multimap<std::string, function_info> create_functions()
    {
      std::unordered_multimap<std::string, function_info> f;
      using enum expression_type;
      using FI = function_info;
      using V = std::vector<expression_type>;
      f.emplace("head", FI{value, V{vector}, true});
      f.emplace("id", FI{integer, V{node}, true});
      f.emplace("id", FI{integer, V{edge}, true});
      f.emplace("keys", FI{vector, V{edge}, true});
      f.emplace("keys", FI{vector, V{node}, true});
      f.emplace("keys", FI{vector, V{map}, true});
      f.emplace("labels", FI{vector, V{node}, true});
      f.emplace("properties", FI{map, V{node}, true});
      f.emplace("properties", FI{map, V{edge}, true});
      f.emplace("properties", FI{map, V{map}, true});
      f.emplace("range", FI{string, V{integer, integer}, true});
      f.emplace("size", FI{integer, V{vector}, true});
      f.emplace("tail", FI{vector, V{vector}, true});
      f.emplace("toInteger", FI{integer, V{integer}, true});
      f.emplace("toInteger", FI{integer, V{floating_point}, true});
      f.emplace("type", FI{string, V{edge}, true});
      return f;
    }
    expression_info visit(algebra::function_call_csp _node) override
    {
      if(_node->get_name() == "coalesce")
      { // coalesce is a special case that can accept any value
        expression_type et;
        bool first = true;
        for(const algebra::node_csp& n : _node->get_arguments())
        {
          expression_info ei = start(n);
          if(first)
          {
            first = false;
            et = ei.type;
          } else if(et != ei.type)
          {
            et = expression_type::value;
            break;
          }
        }
        return {et, constant_nodes(_node->get_arguments())};
      }
      // 1) Check if all arguments are constant and get their type
      bool constant_arguments = true;
      std::vector<expression_type> argument_types;
      for(algebra::node_csp arg : _node->get_arguments())
      {
        expression_info arg_ei = start(arg);
        argument_types.push_back(arg_ei.type);
        constant_arguments = constant_arguments and arg_ei.constant;
      }
      // 2) Find a matching function
      static std::unordered_multimap<std::string, function_info> functions = create_functions();
      auto it_functions = functions.equal_range(_node->get_name());
      for(auto it = it_functions.first; it != it_functions.second; ++it)
      {
        if(argument_types.size() == it->second.arguments_types.size())
        {
          bool match = true;
          for(std::size_t i = 0; i < argument_types.size(); ++i)
          {
            expression_type arg_type = argument_types[i];
            expression_type f_type = it->second.arguments_types[i];
            if(arg_type != f_type and arg_type != expression_type::value and f_type != expression_type::value and arg_type != expression_type::empty)
            {
              match = false;
            }
            break;
          }
          if(match) return {it->second.return_type, it->second.deterministic and constant_arguments};
        }
      }
      // 3) If no function was found, return an exception for unknown function or invalid arguments
      if(it_functions.first == it_functions.second)
      {
        throw_exception(stage, exception_code::unknown_function, "Function {} is unknown.", _node->get_name());
      } else {
        errors::invalid_argument_type(stage, "When calling function {}.", _node->get_name());
      }
    }


  };
}
