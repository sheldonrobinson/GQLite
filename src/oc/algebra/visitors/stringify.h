#include "../default_node_visitor.h"

#include "../../../format.h"
#include "../../../string.h"

namespace gqlite::oc::algebra::visitors
{
  struct stringify : public gqlite::oc::algebra::default_node_visitor<std::string>
  {
    std::string visit_default(algebra::node_csp _node) override
    {
      throw gqlite::exception("Unimplemented node {} in stringify", oc::algebra::node_type_name(_node->get_type()));
    }
    std::string visit(algebra::value_csp _var) override
    {
      return _var->get_value().to_json();
    }
     std::string visit(algebra::array_csp _var) override
    {
      std::string r = "[";
      bool first = true;
      for(algebra::node_csp n : _var->get_array())
      {
        if(not first) r += ", ";
        r += start(n);
        first = false;
      }
      return r + "]";
    }
    std::string visit(algebra::map_csp _var) override
    {
      std::string r = "{";
      bool first = true;
      for(auto const& [k, n] : _var->get_map())
      {
        if(not first) r += ", ";
        r += k + ": " + start(n);
        first = false;
      }
      return r + "}";
    }
    std::string visit(algebra::member_access_csp _var) override
    {
      return format_string("{}.{}", _var->get_left(), string::join(_var->get_path(), ","));
    }
    std::string visit(algebra::indexed_access_csp _var) override
    {
      return format_string("{}[{}]", start(_var->get_left()), start(_var->get_index()));
    }
#define GQLITE_STRINGIFY_BIN_OP(_NAME_, _OP_)                                     \
    std::string visit(algebra::_NAME_ ## _csp _var) override                      \
    {                                                                             \
      return start(_var->get_left()) + " " # _OP_ " " + start(_var->get_right()); \
    }
    GQLITE_STRINGIFY_BIN_OP(logical_and, AND)
    GQLITE_STRINGIFY_BIN_OP(logical_or, OR)
    GQLITE_STRINGIFY_BIN_OP(logical_xor, XOR)
#define GQLITE_STRINGIFY_UNARY_OP(_NAME_, _OP_)                                   \
    std::string visit(algebra::_NAME_ ## _csp _var) override                      \
    {                                                                             \
      return # _OP_ " " + start(_var->get_value());                                 \
    }
    GQLITE_STRINGIFY_UNARY_OP(logical_negation, NOT)
    std::string visit(algebra::variable_csp _var) override
    {
      return _var->get_identifier();
    }
    std::string visit(algebra::function_call_csp _var) override
    {
      std::string args;
      for(algebra::node_csp node : _var->get_arguments())
      {
        if(not args.empty()) args += ",";
        args += start(node);
      }
      return format_string("{}({})", _var->get_name(), args);
    }
  };
}
