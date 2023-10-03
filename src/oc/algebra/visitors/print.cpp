#include "print.h"

#include <iostream>
#include <type_traits>

#include "../../../logging.h"

using namespace gqlite::oc::algebra::visitors;

namespace
{
  static void print_h(const std::string& _indentation, const std::string& _message)
  {
    std::cerr << _indentation << _message << std::endl;;
  }
  template<typename _T_, typename... _TOther_>
  static void print_h(const std::string& _indentation, const std::string& _message, _T_ const& _v, _TOther_... _other)
  {
    print_h(_indentation, gqlite::format_string(_message, _v, _other...));
  }
}

  
namespace gqlite::oc::algebra::visitors::details
{
  
  template < template <typename...> class Template, typename T >
  struct is_specialisation_of : std::false_type {};

  template < template <typename...> class Template, typename... Args >
  struct is_specialisation_of< Template, Template<Args...> > : std::true_type {};
  
  template<typename _T_>
  struct remove_const_explicitly_shared_data_pointer { typedef _T_ type; };
  template<typename _T_>
  struct remove_const_explicitly_shared_data_pointer<std::shared_ptr<const _T_>> { typedef _T_ type; };
  
  template<typename... _T_>
  struct print_helper<gqlite::oc::algebra::alternative<_T_...>>
  {
    static void call(print* _visitor, std::string& _indentation, const char* _name, const gqlite::oc::algebra::alternative<_T_...>& _t)
    {
      if(_t.is_valid())
      {
        if(_name != 0)
        {
          print_h(_indentation, "{}:", _name);
        }
        std::string indentation = _indentation;
        _indentation += "  ";
        _visitor->accept(_t.get_node());
        _indentation = indentation;
      }
    }
  };
  template<typename _T_>
  struct print_helper<std::shared_ptr<const _T_>, typename std::enable_if<std::is_base_of<gqlite::oc::algebra::node, _T_>::value>::type>
  {
    static void call(print* _visitor, std::string& _indentation, const char* _name, std::shared_ptr<const _T_> _t)
    {
      if(_t)
      {
        if(_name != 0)
        {
          print_h(_indentation, "{}:", _name);
        }
        std::string indentation = _indentation;
        _indentation += "  ";
        _visitor->accept(_t);
        _indentation = indentation;
      }
    }
  };
  template<typename _T_, typename>
  struct print_helper
  {
    static void call(print* _visitor, std::string& _indentation, const char* _name, const _T_& _t)
    {
      GQLITE_UNUSED(_visitor);
      print_h(_indentation, "{}: {}", _name, _t);
    }
  };
  template<typename _T_>
  struct print_helper<std::vector<_T_>, void>
  {
    static void call(print* _visitor, std::string& _indentation, const char* _name, const std::vector<_T_>& _t)
    {
      print_h(_indentation, "{} ({}): [", _name, _t.size());
      for(const _T_& t : _t)
      {
        print_helper<_T_>::call(_visitor, _indentation, "null", t);
      }
      print_h(_indentation, "]");
    }
  };
  template<typename _K_, typename _V_>
  struct print_helper<std::unordered_map<_K_, _V_>, void>
  {
    static void call(print* _visitor, std::string& _indentation, const char* _name, const std::unordered_map<_K_, _V_>& _t)
    {
      print_h(_indentation, "{} ({}): [", _name, _t.size());
      for(auto const& [k,v] : _t)
      {
        print_helper<_K_>::call(_visitor, _indentation, "null", k);
        std::string indentation = _indentation;
        _indentation += "  ";
        print_helper<_V_>::call(_visitor, _indentation, "null", v);
        _indentation = indentation;
      }
      print_h(_indentation, "]");
    }
  };
}

struct print::data
{
  std::string indentation;
};

print::print() : d(new data)
{

}

print::~print()
{
  delete d;
}

#define GQL_GENERATE_VISIT_CALL(_KLASS_NAME_, _TYPE_, _NAME_)          \
  details::print_helper<_TYPE_>::call(this, d->indentation, # _NAME_, _node->get_ ## _NAME_());

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                 \
void print::visit(_KLASS_NAME_ ## _csp _node)                           \
{                                                                       \
  GQLITE_UNUSED(_node);                                                 \
  print_h(d->indentation, # _KLASS_NAME_);                              \
  std::string oldindentation = d->indentation;                          \
  d->indentation += "  ";                                               \
  _MEMBER_DEF_(_KLASS_NAME_, GQL_GENERATE_VISIT_CALL)                   \
  d->indentation = oldindentation;                                      \
}

#include "../nodes_defs.h"

#undef OC_ALGEBRA_GENERATE
