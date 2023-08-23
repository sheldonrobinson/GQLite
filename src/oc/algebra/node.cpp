#include "node_p.h"

using namespace gqlite::oc::algebra;

node::node(node_type _type, node::data* _d) : d(_d)
{
  d->type = _type;
}

node::~node()
{
  delete d;
}

node_type node::get_type() const
{
  return d->type;
}
