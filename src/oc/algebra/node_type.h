namespace gqlite::oc::algebra
{
  enum class node_type
  {
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                                             \
  _KLASS_NAME_,
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
    __Invalid__
  };
  const char* node_type_name(node_type _nt);
}
