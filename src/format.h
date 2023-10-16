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
  namespace details
  {
    inline std::string format_string(std::size_t, const std::string& _format)
    {
      return _format;
    }
    template<typename _T_, typename... _Targs_>
    inline std::string format_string(std::size_t _start_pos, const std::string& _format, const _T_& _a, const _Targs_&... _args)
    {
      std::size_t start_pos = _format.find("{}", _start_pos);
      if(start_pos == std::string::npos)
      {
        return _format;
      }
      std::string f = _format;
      std::string replacement = to_string(_a);
      f.replace(start_pos, 2, replacement);
      return format_string(start_pos + replacement.size(), f, _args...);
    }
  }
  template<typename... _Targs_>
  inline std::string format_string(const std::string& _format, const _Targs_&... _args)
  {
    return details::format_string(0, _format, _args...);
  }
  namespace details
  {
    template<typename... _Targs_>
    struct formatter
    {
      template<typename _Targ_>
      formatter<_Targs_..., _Targ_> operator ,(const _Targ_& _arg)
      {
        return formatter<_Targs_..., _Targ_>{std::tuple_cat(m_args, std::make_tuple(_arg))};
      }
      operator std::string() const
      {
        return std::apply(&gqlite::format_string<_Targs_...>, std::tuple_cat(m_args));
      }
      std::tuple<std::string, _Targs_...> m_args;
    };
  }
  template<typename _Targ_>
  inline details::formatter<_Targ_> operator%(const std::string& _format, const _Targ_& _arg)
  {
    return details::formatter<_Targ_>{std::make_tuple(_format, _arg)};
  }
}
