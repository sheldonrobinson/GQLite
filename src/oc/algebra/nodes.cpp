#include "nodes.h"

#include "abstract_node_visitor.h"

#include "node_p.h"

using namespace gqlite::oc::algebra;

namespace gqlite::oc::algebra::details
{
  template<typename _T_>
  bool equals(const _T_& _a, const _T_& _b) requires std::same_as<_T_, gqlite::value> or std::same_as<_T_, std::string> or std::same_as<_T_, edge_directivity> or std::is_integral_v<_T_> or std::is_floating_point_v<_T_>
  {
    return _a == _b;
  }
  template<typename _T_>
  bool equals(const std::shared_ptr<const _T_>& _a, const std::shared_ptr<const _T_>& _b) requires std::derived_from<_T_, node>
  {
    return (_a == _b) or (_a and _b and _a->equals(_b));
  }
  template<typename... _T_>
  bool equals(const alternative<_T_...>& _a, const alternative<_T_...>& _b)
  {
    return _a.get_node()->equals(_b.get_node());
  }
  template<typename _T_>
  bool equals(const std::vector<_T_>& _a, const std::vector<_T_>& _b)
  {
    if(_a.size() != _b.size()) return false;
    for(int i = 0; i < _a.size(); ++i)
    {
      if(not equals(_a[i], _b[i])) return false;
    }
    return true;
  }
  template<typename _T_>
  bool equals(const std::unordered_map<std::string, _T_>& _a, const std::unordered_map<std::string, _T_>& _b)
  {
    if(_a.size() != _b.size()) return false;
    for(auto const& [k,v] : _a)
    {
      auto it = _b.find(k);
      if(it == _b.end() or not equals(v, it->second)) return false;
    }
    return true;
  }
}

#define OC_ALGEBRA_GENERATE_PRIVATE_MEMBER(_KLASS_NAME_, _TYPE_, _NAME_) \
  _TYPE_ m_ ## _NAME_;

#define OC_ALGEBRA_GENERATE_ACCESSOR_DEFINITION(_KLASS_NAME_, _TYPE_, _NAME_)   \
  _TYPE_ _KLASS_NAME_::get_ ## _NAME_() const                                   \
  {                                                                             \
    return static_cast<data*>(d)->m_ ## _NAME_;                                 \
  }

#define GQL_GENERATE_DESTRUCTOR(_KLASS_NAME_, _TYPE_, _NAME_)
  
#define OC_ALGEBRA_GENERATE_ASSIGNMENT(_KLASS_NAME_, _TYPE_, _NAME_)  \
  static_cast<data*>(d)->m_ ## _NAME_ = _ ## _NAME_;

#define OC_ALGEBRA_GENERATE_COMPARISON(_KLASS_NAME_, _TYPE_, _NAME_)  \
  and details::equals(static_cast<data*>(d)->m_ ## _NAME_, static_cast<data*>(rhs->d)->m_ ## _NAME_)

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                                                 \
struct _KLASS_NAME_::data : public node::data                                                                                                           \
{                                                                                                                                                       \
  _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_PRIVATE_MEMBER)                                                                                        \
};                                                                                                                                                      \
                                                                                                                                                        \
_KLASS_NAME_::_KLASS_NAME_(_MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_CONSTRUCTOR_ARGUMENT) void* _)                                                \
  : node(node_type::_KLASS_NAME_, new data)                                                                                                             \
{                                                                                                                                                       \
  GQLITE_UNUSED(_);                                                                                                                                     \
  _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_ASSIGNMENT)                                                                                            \
}                                                                                                                                                       \
                                                                                                                                                        \
_KLASS_NAME_::~_KLASS_NAME_()                                                                                                                           \
{                                                                                                                                                       \
  _MEMBER_DEF_(_KLASS_NAME_, GQL_GENERATE_DESTRUCTOR)                                                                                                   \
}                                                                                                                                                       \
                                                                                                                                                        \
bool _KLASS_NAME_::equals(const std::shared_ptr<const node>& _node) const                                                                               \
{                                                                                                                                                       \
  if(_node->get_type() != node_type::_KLASS_NAME_) return false;                                                                                        \
  _KLASS_NAME_ ## _csp rhs = std::static_pointer_cast<const _KLASS_NAME_>(_node);                                                                       \
  return true _MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_COMPARISON);                                                                               \
}                                                                                                                                                       \
                                                                                                                                                        \
_MEMBER_DEF_(_KLASS_NAME_, OC_ALGEBRA_GENERATE_ACCESSOR_DEFINITION)                                                                                     \
                                                                                                                                                        \
void _KLASS_NAME_::accept(details::abstract_node_visitor_adaptor* _visitor_adaptor, void* _r, void* _parameter) const                                   \
{                                                                                                                                                       \
  return _visitor_adaptor->call_visit(std::static_pointer_cast<const _KLASS_NAME_>(shared_from_this()), _r, _parameter);                                \
}

#include "nodes_defs.h"
