#include "../default_node_visitor.h"

#include "../../../format.h"
#include "../../../string.h"

namespace gqlite::oc::algebra::visitors
{
  struct stringify : public gqlite::oc::algebra::default_node_visitor<std::string>
  {
    std::string visit_default(algebra::node_csp _node) override
    {
      throw gqlite::exception("Unimplemented statement node {} in stringify", oc::algebra::node_type_name(_node->get_type()));
    }
    std::string visit(algebra::member_access_csp _var) override
    {
      return format_string("{}.{}", _var->get_left(), string::join(_var->get_path(), ","));
    }
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
