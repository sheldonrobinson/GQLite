#ifndef _OC_ALGEBRA_NODES_H_
#define _OC_ALGEBRA_NODES_H_

#include <gqlite.h>

#include "node.h"

namespace gqlite::oc::algebra
{
  enum class match_modifier
  {
    none, distinct
  };
}

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                                             \
  class _KLASS_NAME_;                                                                                                                                       \
  typedef std::shared_ptr<const _KLASS_NAME_> _KLASS_NAME_ ## _csp;


namespace gqlite::oc::algebra
{
#include "nodes_defs.h"
}

#undef OC_ALGEBRA_GENERATE

#define OC_ALGEBRA_GENERATE_CONSTRUCTOR_ARGUMENT(_KLASS_NAME_, _TYPE_, _NAME_)      \
  _TYPE_ _ ## _NAME_,

#define OC_ALGEBRA_GENERATE_ACCESSOR_DECLARATION(_KLASS_NAME_, _TYPE_, _NAME_,...)  \
  _TYPE_ get_ ## _NAME_() const;

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                                         \
  class _KLASS_NAME_ : public node                                                                                                              \
  {                                                                                                                                             \
  public:                                                                                                                                       \
    _KLASS_NAME_(_MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_CONSTRUCTOR_ARGUMENT) void* _ = nullptr);                                       \
    ~_KLASS_NAME_();                                                                                                                            \
  public:                                                                                                                                       \
    _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_ACCESSOR_DECLARATION);                                                                       \
    using node::accept;                                                                                                                         \
    bool equals(const std::shared_ptr<const node>& _node) const override;                                                                       \
  private:                                                                                                                                      \
    void accept(details::abstract_node_visitor_adaptor* _node, void* _r, void* _parameter) const override;                                      \
  private:                                                                                                                                      \
    struct data;                                                                                                                                \
  };

namespace gqlite::oc::algebra
{
#include "nodes_defs.h"
}

#undef OC_ALGEBRA_GENERATE

#endif
