#include <gqlite.h>

#include "format.h"

namespace gqlite
{
  template<typename _T_, typename... _TOther_>
  exception::exception(const char* _format, const _T_& _value, const _TOther_&... _other) : exception(format_string(_format, _value, _other...))
  {
  }
}
