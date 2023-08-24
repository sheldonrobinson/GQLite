#include <gqlite.h>

#include "oc/algebra/node.h"

namespace gqlite
{
  class backend
  {
  public:
    virtual ~backend();
  public:
    virtual result execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, std::any>& _variant = std::unordered_map<std::string, std::any>()) = 0;
  };
}
