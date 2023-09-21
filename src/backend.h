#include "gqlite_p.h"

#include "oc/algebra/node.h"

namespace gqlite
{
  class backend
  {
  public:
    virtual ~backend();
  public:
    virtual value execute_oc_query(oc::algebra::node_csp _node) = 0;
  };
}
