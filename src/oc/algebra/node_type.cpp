#include "node_type.h"

#include "../../logging.h"

namespace gqlite::oc::algebra
{
  const char * node_type_name(node_type _nt)
  {
    switch(_nt)
    {
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_) \
      case node_type::_KLASS_NAME_:                              \
        return #_KLASS_NAME_;
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
      case node_type::__Invalid__:
        return "__Invalid__";
    }
    gqlite_fatal("impossible");
  }
}

