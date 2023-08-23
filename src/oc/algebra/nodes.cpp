#include "nodes.h"

#include "abstract_node_visitor.h"

#include "node_p.h"

using namespace gqlite::oc::algebra;

#define OC_ALGEBRA_GENERATE_PRIVATE_MEMBER(_KLASS_NAME_, _TYPE_, _NAME_) \
  _TYPE_ m_ ## _NAME_;

#define OC_ALGEBRA_GENERATE_ACCESSOR_DEFINITION(_KLASS_NAME_, _TYPE_, _NAME_)  \
  _TYPE_ _KLASS_NAME_::get_ ## _NAME_() const                                           \
  {                                                                             \
    return static_cast<data*>(d)->m_ ## _NAME_;                                 \
  }

#define GQL_GENERATE_DESTRUCTOR(_KLASS_NAME_, _TYPE_, _NAME_)
  
#define OC_ALGEBRA_GENERATE_ASSIGNMENT(_KLASS_NAME_, _TYPE_, _NAME_)  \
  static_cast<data*>(d)->m_ ## _NAME_ = _ ## _NAME_;

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                                         \
struct _KLASS_NAME_::data : public node::data                                                                                                     \
{                                                                                                                                                       \
  _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_PRIVATE_MEMBER)                                                                                \
};                                                                                                                                                      \
                                                                                                                                                        \
_KLASS_NAME_::_KLASS_NAME_(_MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_CONSTRUCTOR_ARGUMENT) void* _)                                        \
  : node(node_type::_KLASS_NAME_, new data)                                                                                                           \
{                                                                                                                                                       \
  GQLITE_UNUSED(_);                                                                                                                                          \
  _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_ASSIGNMENT)                                                                                    \
}                                                                                                                                                       \
                                                                                                                                                        \
_KLASS_NAME_::~_KLASS_NAME_()                                                                                                                           \
{                                                                                                                                                       \
  _MEMBER_DEF_(_KLASS_NAME_, GQL_GENERATE_DESTRUCTOR)                                                                                            \
}                                                                                                                                                       \
                                                                                                                                                        \
_MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_ACCESSOR_DEFINITION)                                                                             \
                                                                                                                                                        \
void _KLASS_NAME_::accept(details::abstract_node_visitor_adaptor* _visitor_adaptor, void* _r, void* _parameter) const                                      \
{                                                                                                                                                       \
  return _visitor_adaptor->call_visit(_KLASS_NAME_ ## _csp(this), _r, _parameter);                                                                                            \
}

#include "nodes_defs.h"
