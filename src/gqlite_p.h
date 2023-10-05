#pragma once

#include <gqlite.h>

#include "format.h"

namespace gqlite
{
  template<typename _T_, typename... _TOther_>
  inline exception::exception(const char* _format, const _T_& _value, const _TOther_&... _other) : exception(format_string(_format, _value, _other...))
  {
  }
  template<typename _T_, typename... _TOther_>
  inline exception::exception(const std::string& _format, const _T_& _value, const _TOther_&... _other) : exception(format_string(_format, _value, _other...))
  {
  }

  template<typename _T_>
  inline value::value(const std::initializer_list<typename std::unordered_map<std::string, _T_>::value_type>& _v) : value()
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
  inline value::value(const std::vector<_T_>& _v) : value()
  {
    value_vector vo;
    for(const _T_& t : _v)
    {
      vo.push_back(t);
    }
    *this = value(vo);
  }
  inline value remove_invalid_in_map(const value& _value)
  {
    switch(_value.get_type())
    {
      using enum value_type;
      case invalid:
      case boolean:
      case integer:
      case number:
      case string:
        return _value;
      case map:
      {
        value_map vm;
        for(const auto& [k, v] : _value.to_map())
        {
          if(v.get_type() != invalid)
          {
            vm[k] = remove_invalid_in_map(v);
          }
        }
        return vm;
      }
      case vector:
      {
        value_vector vv;
        for(const value& v : _value.to_vector())
        {
          if(v.get_type() != invalid)
          {
            vv.push_back(remove_invalid_in_map(v));
          }
        }
        return vv;
      }
    }
    throw exception("Internal error, unknown value type for {}", _value);
  }
}
