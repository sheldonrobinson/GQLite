#include <gqlite.h>

namespace gqlite::errors
{
  template<typename... _TOther_>
  inline void check_condition(bool _condition, const char* _format, const _TOther_&... _other)
  {
    if(not _condition)
    {
      throw exception(_format, _other...);
    }
  }
  template<typename... _TOther_>
  inline void check_condition(bool _condition, const std::string& _format, const _TOther_&... _other)
  {
    if(not _condition)
    {
      throw exception(_format, _other...);
    }
  }
  template<typename _T_>
  inline void check_arguments_size(const char* _fname, const std::vector<_T_>& _arguments, std::size_t _size)
  {
    check_condition(_arguments.size() == _size, "'{}' function expect {} arguments, got {}", _fname, _size, _arguments.size());
  }
  template<typename _T_, typename... _TOther_>
  inline void check_argument_type(_T_ _got, _T_ _expected, const char* _format, const _TOther_&... _other)
  {
    check_condition(_got == _expected, "InvalidArgumentType: " + std::string(_format), _other...);
  }
  template<typename _T_, typename... _TOther_>
  inline void check_argument_type_function(_T_ _got, _T_ _expected, const char* _fname, const _TOther_&... _other)
  {
    check_argument_type(_got, _expected, "InvalidArgumentType: invalid argument type for function '{}'", _fname, _other...);
  }
  template<typename... _TOther_>
  [[noreturn]] inline void invalid_argument_type(const char* _format, const _TOther_&... _other)
  {
    throw exception("InvalidArgumentType: " + std::string(_format), _other...);
  }  
}
