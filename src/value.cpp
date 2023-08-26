#include "gqlite.h"
#include <variant>

using namespace gqlite;

struct value::data
{
  std::variant<int, double, std::string, std::unordered_map<std::string, value>, std::vector<value>> value_container;
};

value::value() : d(new data)
{}

value::value(const value& _rhs) : d(_rhs.d)
{
}

value& value::operator=(const value& _rhs)
{
  d = _rhs.d;
  return *this;
}

value::~value()
{}
