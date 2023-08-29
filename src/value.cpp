#include "gqlite.h"

#include <sstream>
#include <variant>

#include "format.h"

using namespace gqlite;

const char* std::to_string(value_type _type)
{
  switch(_type)
  {
    using enum value_type;
    case invalid: return "invalid";
    case boolean: return "boolean";
    case number: return "number";
    case string: return "string";
    case map: return "map";
    case vector: return "vector";
  }
  return "unknown value type";
}

struct value::data
{
  value_type type;
  std::variant<bool, double, std::string, std::unordered_map<std::string, value>, std::vector<value>> value_container;

  void to_json(std::stringstream& _stream);
};

void value::data::to_json(std::stringstream& _stream)
{
  bool first = true;
  switch(type)
  {
  case value_type::number:
    _stream << std::get<double>(value_container);
    break;
  case value_type::string:
    _stream << '"' << std::get<std::string>(value_container) << '"';
    break;
  case value_type::map:
    _stream << '{';
    for(auto const& [k, v] : std::get<std::unordered_map<std::string, value>>(value_container))
    {
      if(first)
      {
        first = false;
      } else {
        _stream << ',';
      }
      _stream << '"' << k << "\":";
      v.d->to_json(_stream);
    }
    _stream << '}';
    break;
  case value_type::vector:
    _stream << '[';
    for(const value& v : std::get<std::vector<value>>(value_container))
    {
      if(first)
      {
        first = false;
      } else {
        _stream << ',';
      }
      v.d->to_json(_stream);
    }
    _stream << ']';
    break;
  case value_type::invalid:
    _stream << "null";
  }

}

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

value::value(double _v) : d(new data{value_type::number, _v})
{}

value::value(const std::string& _v) : d(new data{value_type::string, _v})
{}

value::value(const std::unordered_map<std::string, value>& _v) : d(new data{value_type::map, _v})
{}

value::value(const std::vector<value>& _v) : d(new data{value_type::vector, _v})
{}

value_type value::get_type() const
{
  return d->type;
}

bool value::to_bool() const
{
  switch(d->type)
  {
    case value_type::boolean:
      return std::get<bool>(d->value_container);
    case value_type::number:
      return std::get<double>(d->value_container) != 0.0;
    default:
      throw exception(format_string("Value is not a bool, it is {}", d->type));
  }
}
double value::to_double() const
{
  if(d->type != value_type::number) throw exception(format_string("Value is not a number, it is {}", d->type));
  return std::get<double>(d->value_container);
}

std::string value::to_string() const
{
  if(d->type != value_type::string) throw exception(format_string("Value is not a string, it is {}", d->type));
  return std::get<std::string>(d->value_container);
}

std::unordered_map<std::string, value> value::to_map() const
{
  if(d->type != value_type::map) throw exception(format_string("Value is not a map, it is {}", d->type));
  return std::get<std::unordered_map<std::string, value>>(d->value_container);
}

std::vector<value> value::to_vector() const
{
  if(d->type != value_type::vector) throw exception(format_string("Value is not a vector, it is {}", d->type));
  return std::get<std::vector<value>>(d->value_container);
}

std::string value::to_json() const
{
  std::stringstream ss;
  d->to_json(ss);
  return ss.str();
}
