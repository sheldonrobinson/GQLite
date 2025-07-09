#include "gqlite.h"
#include <string>

namespace gqlite
{
  /**
   * Represent the value type.
   */
  enum class value_type
  {
    invalid,
    boolean,
    integer,
    floating_point,
    string,
    map,
    vector
  };
} // namespace gqlite
