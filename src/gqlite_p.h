#include <gqlite.h>

#include "format.h"

namespace gqlite
{
  template<typename _T_, typename... _TOther_>
  exception::exception(const char* _format, const _T_& _value, const _TOther_&... _other) : exception(format_string(_format, _value, _other...))
  {
  }
  template<typename _T_, typename... _TOther_>
  exception::exception(const std::string& _format, const _T_& _value, const _TOther_&... _other) : exception(format_string(_format, _value, _other...))
  {
  }

  template<typename _T_>
  value::value(const std::initializer_list<typename std::unordered_map<std::string, _T_>::value_type>& _v) : value()
  {
    std::unordered_map<std::string, _T_> m{_v};
    std::unordered_map<std::string, _T_> mo;
    for(auto const&[k, v] : m)
    {
      mo[k] = v;
    }
    *this = value(mo);
  }
  template<typename _T_>
  value::value(const std::vector<_T_>& _v) : value()
  {
    std::vector<value> vo;
    for(const _T_& t : _v)
    {
      vo.push_back(t);
    }
    *this = value(vo);
  }
}
