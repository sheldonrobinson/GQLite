#pragma once

#include "config.h"

#include <string>

namespace std
{
  const char* to_string(gqlite::value_type _type);
}


#ifdef GQLITE_HAVE_CLOG

#include <fmt>

namespace gqlite
{
  template<typename... _Targs_>
  std::string format_string(const std::string& _format, const _Targs_&... _args)
  {
    return fmt::format(_format, _args...);
  }
}

#else

namespace gqlite
{
  /**
   * @internal
   * @return \p _integer converted to a string, with a minimum size of \p _width, filled with 0.
   */
  inline std::string to_string_fixed_width(int _integer, int _width)
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
  namespace details
  {
    template<typename _T_>
    inline std::string to_string(const _T_& _v)
    {
      return std::to_string(_v);
    }
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
  }
  template<typename _T_, typename... _Targs_>
  inline std::string format_string(const std::string& _format, const _T_& _a, const _Targs_&... _args)
  {
    int start_pos = _format.find("{}");
    if(start_pos == std::string::npos)
    {
      return _format;
    }
    std::string f = _format;
    f.replace(start_pos, 2, details::to_string(_a));
    return format_string(f, _args...);
  }
}


#endif
