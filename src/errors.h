#include <gqlite.h>

namespace gqlite::errors
{
  template<typename _T_>
  inline void check_arguments_size(const char* _fname, const std::vector<_T_>& _arguments, int _size)
  {
    if(_arguments.size() != _size)
    {
      throw exception("'{}' function expect {} arguments, got {}", _fname, _size, _arguments.size());
    }
  }
  template<typename _T_>
  inline void check_argument_type(const char* _fname, _T_ _got, _T_ _expected)
  {
    if(_got != _expected)
    {
      throw exception("Invalid argument type for function '{}'", _fname);
    }
  }
}
