#pragma once

#include "abstract_node_visitor.h"

#include "../../global.h"

#define __OC_ALGEBRA_NODE_VISITOR_OVERLOAD(_NAME_, _I_)             \
  Return visit(_NAME_ ## CSP _node, const Parameter& _parameter);

#define OC_ALGEBRA_NODE_VISITOR_OVERLOAD(...)                       \
  GQLITE_FOREACH(__OC_ALGEBRA_NODE_VISITOR_OVERLOAD, __VA_ARGS__)

namespace gqlite::oc::algebra
{
  /**
   * @ingroup GQL_Algebra
   * Base abstract_node_visitor with empty implementation
   */
  template<typename _TR_, typename... _TArgs_>
  class default_node_visitor : public abstract_node_visitor<_TR_, _TArgs_...>
  {
  protected:
    virtual _TR_ visit_default(node_csp _node, const _TArgs_&...) = 0;
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                               \
        _TR_ visit(_KLASS_NAME_ ## _csp _node, const _TArgs_&... _args) override       \
        {                                                                             \
          return visit_default(_node, _args...);                                      \
        }
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
  };
}

