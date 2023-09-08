#include <gqlite.h>

namespace gqlite::backends::functions
{
  value call(const std::string& _name, const std::vector<value>& _arguments);
}
