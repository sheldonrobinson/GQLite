#include "gqlite_p.h"

namespace gqlite::errors
{
  template<typename... _TOther_>
  inline void check_condition(bool _condition, exception_stage _stage, exception_code _code, const char* _format, const _TOther_&... _other)
  {
    if(not _condition)
    {
      throw_exception(_stage, _code, _format, _other...);
    }
  }
  template<typename... _TOther_>
  inline void check_condition(bool _condition, exception_stage _stage, exception_code _code, const std::string& _format, const _TOther_&... _other)
  {
    if(not _condition)
    {
      throw_exception(_stage, _code, _format, _other...);
    }
  }
  template<typename _T_>
  inline void check_arguments_size(const char* _fname, const std::vector<_T_>& _arguments, std::size_t _size)
  {
    check_condition(_arguments.size() == _size, exception_stage::runtime, exception_code::invalid_number_of_arguments, "'{}' function expect {} arguments, got {}", _fname, _size, _arguments.size());
  }
  template<typename _T_, typename... _TOther_>
  inline void check_argument_type(_T_ _got, _T_ _expected, exception_stage _stage, const char* _format, const _TOther_&... _other)
  {
    check_condition(_got == _expected, _stage, exception_code::invalid_argument_type, _format, _other...);
  }
  template<typename _T_, typename... _TOther_>
  inline void check_argument_type_function(_T_ _got, _T_ _expected, const char* _fname, const _TOther_&... _other)
  {
    check_argument_type(_got, _expected, exception_stage::runtime, "Invalid argument type for function '{}'.", _fname, _other...);
  }
  inline void check_argument_is_map(const char* _fname, const value& _value)
  {
    check_argument_type_function(_value.get_type(), value_type::map, "Function '{}' expect a map, got '{}'.", _fname, _value);
  }
  template<typename... _TOther_>
  [[noreturn]] inline void invalid_argument_type(exception_stage _stage, const char* _format, const _TOther_&... _other)
  {
    throw_exception(_stage, exception_code::invalid_argument_type, _format, _other...);
  }  
}
