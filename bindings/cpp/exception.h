#include "gqlite.h"

#include "format.h"

namespace gqlite
{
  [[noreturn]] void throw_exception(const std::string& _msg);
  [[noreturn]] void throw_exception(const char* _msg);
  template<typename _T_, typename... _TOther_>
  [[noreturn]] inline void throw_exception(const std::string& _format, const _T_& _value,
                                           const _TOther_&... _other)
  {
    throw_exception(format_string(_format, _value, _other...));
  }
  template<typename _T_, typename... _TOther_>
  inline void throw_exception_if(bool _cond, const std::string& _format, const _T_& _value,
                                 const _TOther_&... _other)
  {
    if(_cond)
    {
      throw_exception(_format, _value, _other...);
    }
  }
  inline void throw_exception_if(bool _cond, const char* _error)
  {
    if(_cond)
    {
      throw_exception(_error);
    }
  }
} // namespace gqlite