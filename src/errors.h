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
  template<typename _T_>
  inline void check_arguments_size(const char* _fname, const std::vector<_T_>& _arguments, std::size_t _size)
  {
    check_condition(_arguments.size() == _size, "'{}' function expect {} arguments, got {}", _fname, _size, _arguments.size());
  }
  template<typename _T_>
  inline void check_argument_type(const char* _fname, _T_ _got, _T_ _expected)
  {
    check_condition(_got == _expected, "Invalid argument type for function '{}'", _fname);
  }
}
