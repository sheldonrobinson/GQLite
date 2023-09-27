#pragma once

#include <gqlite.h>

#include <string>

namespace gqlite
{
  template<typename T> concept std_to_stringable = requires (T a) {
      std::to_string(a);
  };
  template<typename T> concept container_begin_end = requires (T a) {
      a.begin(); a.end();
  };

  /**
   * @internal
   * @return \p _integer converted to a string, with a minimum size of \p _width, filled with 0.
   */
  inline std::string to_string_fixed_width(int _integer, std::size_t _width)
  {
    std::string s = std::to_string(_integer);
    while(s.size() < _width)
    {
      s = '0' + s;
    }
    return s;
  }
  inline std::string format_string(const std::string& _format)
  {
    return _format;
  }
  template<typename _T_>
  struct to_string_impl;
  template<typename _T_>
  inline std::string to_string(const _T_& _v)
  {
    return to_string_impl<_T_>::to_string(_v);
  }
  template<typename _T_>  requires std_to_stringable<_T_>
  struct to_string_impl<_T_>
  {
    static inline std::string to_string(const _T_& _v)
    {
      return std::to_string(_v);
    }
  };

  template<typename _T_>  requires container_begin_end<_T_>
  struct to_string_impl<_T_>
  {
    static inline std::string to_string(const _T_& _c)
    {
      std::string r = "(";
      for(auto v: _c)
      {
        r += gqlite::to_string(v) + " ";
      }
      r += ")";
      return r;
    }
  };

  const char* to_string(gqlite::value_type _type);
  std::string to_string(const value& _value);
  template<>
  inline std::string to_string<std::string>(const std::string& _v)
  {
    return _v;
  }
  template<>
  inline std::string to_string<const char*>(const char* const& _v)
  {
    return _v;
  }
  template<>
  inline std::string to_string<char>(const char& _v)
  {
    std::string v;
    return v += _v;
  }
  
  template<typename _T_, typename... _Targs_>
  inline std::string format_string(const std::string& _format, const _T_& _a, const _Targs_&... _args)
  {
    std::size_t start_pos = _format.find("{}");
    if(start_pos == std::string::npos)
    {
      return _format;
    }
    std::string f = _format;
    f.replace(start_pos, 2, to_string(_a));
    return format_string(f, _args...);
  }
}
