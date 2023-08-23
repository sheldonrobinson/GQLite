#include "node.h"

namespace gqlite::oc::algebra
{
  struct node::data
  {
    virtual ~data() {}
    node_type type;
  };
}
