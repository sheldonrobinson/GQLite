#include "gqlite.h"

#include <sstream>
#include <variant>

#include "gqlite_p.h"
#include "errors.h"

using namespace gqlite;

const char* gqlite::to_string(value_type _type)
{
  switch(_type)
  {
    using enum value_type;
    case invalid: return "invalid";
    case boolean: return "boolean";
    case integer: return "integer";
    case number: return "number";
    case string: return "string";
    case map: return "map";
    case vector: return "vector";
  }
  return "unknown value type";
}

std::string gqlite::to_string(const value& _value)
{
  return _value.to_json();
}
struct value::data
{
  struct invalid
  {
    bool operator==(const invalid&) const { return true; }
  };
  value_type type;
  std::variant<invalid, bool, int64_t, double, std::string, value_map, value_vector> value_container;

  void to_json(std::stringstream& _stream);
};

namespace {
  /**
   * @internal
   * class use to read json files
   */
  struct json_reader
  {
    std::stringstream stream;
    int last_char;
    /// @brief get the next char
    void fetch_next_non_space_char()
    {
      while(std::isspace(fetch_next_char()))
      {
        if(last_char == std::stringstream::traits_type::eof())
        {
          return;
        } else if(stream.fail())
        {
          throw gqlite::exception("Failure in getting json data from stream");
        }
      }
    };
    int fetch_next_char()
    {
      return (last_char = stream.get());
    }
    /// @brief start parsing
    /// @throw gqlite::exception
    /// @return the parsed value
    gqlite::value start()
    {
      fetch_next_non_space_char();
      return read_value();
    }
    /// @brief read a string
    /// @return the string without quotation
    std::string read_string()
    {
      is_char('"');
      fetch_next_char();
      std::string string;
      bool keep_nc = false; // this is used to indicate if the last character was a '\' or not
      while(keep_nc or last_char != '"')
      {
        if(not keep_nc and last_char == '\\')
        {
          keep_nc = true; // make sure to ignore the next character if it is a '"' or to properly handle a '\n'
        } else {
          if(keep_nc)
          {
            keep_nc = false;
            if(last_char == 'n')
            {
              string += '\n';
            } else {
              string += char(last_char);
            }
          } else {
            string += char(last_char);
          }
        }
        errors::check_condition(stream.good(), "Unifinished string: {}", string);
        fetch_next_char();
      }
      fetch_next_char(); // eat the '"'
      return string;
    }
    /// @brief throw an exception if last_char different from @param _c 
    void is_char(int _c)
    {
      errors::check_condition(last_char == _c, "Expected '{}' but got '{}'", char(_c), char(last_char));
    }
    /// @brief read the next value
    /// @return and return it
    gqlite::value read_value()
    {
      switch (last_char)
      {
      case '{':
      {
        // Parse object
        value_map map;
        fetch_next_non_space_char();
        while(last_char != '}')
        {
          std::string str = read_string();
          is_char(':');
          fetch_next_non_space_char();
          gqlite::value val = read_value();
          map[str] = val;
          if(last_char == ',')
          {
            fetch_next_non_space_char();
          } else {
            break;
          }
        }
        is_char('}');
        fetch_next_non_space_char();
        return map;
      }
      case '[':
      {
        // Parse array
        value_vector vec;
        fetch_next_non_space_char();
        while(last_char != ']')
        {
          gqlite::value val = read_value();
          vec.push_back(val);
          if(last_char == ',')
          {
            fetch_next_non_space_char();
          } else {
            break;
          }
        }
        is_char(']');
        fetch_next_non_space_char();
        return vec;
      }
      case '"':
        return read_string();
      case 't':
      {
        // Parse true
        fetch_next_non_space_char(); is_char('r');
        fetch_next_non_space_char(); is_char('u');
        fetch_next_non_space_char(); is_char('e');
        fetch_next_non_space_char();
        return true;
      }
      case 'f':
      {
        // Parse false
        fetch_next_non_space_char(); is_char('a');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char(); is_char('s');
        fetch_next_non_space_char(); is_char('e');
        fetch_next_non_space_char();
        return false;
      }
      case 'n':
      {
        // Parse null
        fetch_next_non_space_char(); is_char('u');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char();
        return value();
      }
      // Parse number
      case '-':
      case '0':
      case '1':
      case '2':
      case '3':
      case '4':
      case '5':
      case '6':
      case '7':
      case '8':
      case '9':
      {
        std::string number;
        number += char(last_char);
        fetch_next_non_space_char();
        bool is_integer = true;
        while((last_char >= '0' and last_char <= '9') or last_char == '.' or last_char == 'e' or last_char == '+' or last_char == '-')
        {
          is_integer = is_integer and (last_char != '.' and last_char != 'e');
          number += char(last_char);
          fetch_next_non_space_char();
        }
        if(is_integer)
        {
          return int64_t(std::stoll(number));
        } else {
          return std::stod(number);
        }
      }
      default:
        throw gqlite::exception("Unexpected {}", char(last_char));
      }
    }
  };
}

void value::data::to_json(std::stringstream& _stream)
{
  bool first = true;
  switch(type)
  {
  case value_type::boolean:
    _stream << (std::get<bool>(value_container) ? "true" : "false");
    break;
  case value_type::integer:
    _stream << std::get<int64_t>(value_container);
    break;
  case value_type::number:
    _stream << std::get<double>(value_container);
    break;
  case value_type::string:
  {
    std::string string = std::get<std::string>(value_container);
    std::size_t start_pos = 0;
    while((start_pos = string.find("\"", start_pos)) != std::string::npos)
    {
      string.replace(start_pos, 1, "\\\"");
      start_pos += 2;
    }
    _stream << '"' << string << '"';
  }
    break;
  case value_type::map:
    _stream << '{';
    for(auto const& [k, v] : std::get<value_map>(value_container))
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
    for(const value& v : std::get<value_vector>(value_container))
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

value::value() : d(new data{value_type::invalid, data::invalid{}})
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

value::value(bool _v) : d(new data{value_type::boolean, _v})
{}

value::value(int64_t _v) : d(new data{value_type::integer, _v})
{}

value::value(double _v) : d(new data{value_type::number, _v})
{}

value::value(const char* _v) : value(std::string(_v))
{}

value::value(const std::string& _v) : d(new data{value_type::string, _v})
{}

value::value(const value_map& _v) : d(new data{value_type::map, _v})
{}

value::value(const value_vector& _v) : d(new data{value_type::vector, _v})
{}

value_type value::get_type() const
{
  return d->type;
}

bool value::operator==(const value& _rhs) const
{
  return d->type == _rhs.d->type and d->value_container == _rhs.d->value_container;
}

bool value::operator<(const value& _rhs) const
{
  errors::check_condition(d->type == _rhs.d->type, "Value of different types are not comparable with '<'");
  switch(d->type)
  {
  case value_type::boolean:
    return to_bool() < _rhs.to_bool();
  case value_type::integer:
    return to_integer() < _rhs.to_integer();
  case value_type::number:
    return to_double() < _rhs.to_double();
  case value_type::string:
    return to_string() < _rhs.to_string();
  case value_type::vector:
    return to_vector() < _rhs.to_vector();
  default:
    throw exception("Cannot compare {} and {} with inferior operator.", *this, _rhs);
  }
}

bool value::to_bool() const
{
  switch(d->type)
  {
  case value_type::boolean:
    return std::get<bool>(d->value_container);
  case value_type::integer:
    return std::get<int64_t>(d->value_container) != 0;
  case value_type::number:
    return std::get<double>(d->value_container) != 0.0;
  default:
    throw exception("Value is not a bool, it is {}.", d->type);
  }
}

int64_t value::to_integer() const
{
  switch(d->type)
  {
    case value_type::integer:
      return std::get<int64_t>(d->value_container);
    case value_type::number:
      return std::get<double>(d->value_container);
    default:
      throw exception(format_string("Value is not a number, it is {}.", d->type));
  }
}

double value::to_double() const
{
  switch(d->type)
  {
    case value_type::integer:
      return std::get<int64_t>(d->value_container);
    case value_type::number:
      return std::get<double>(d->value_container);
    default:
      throw exception(format_string("Value is not a number, it is {}", d->type));
  }
}

std::string value::to_string() const
{
  errors::check_condition(d->type == value_type::string, "Value is not a string, it is {}", d->type);
  return std::get<std::string>(d->value_container);
}

value_map value::to_map() const
{
  errors::check_condition(d->type == value_type::map, "Value is not a map, it is {}", d->type);
  return std::get<value_map>(d->value_container);
}

value_vector value::to_vector() const
{
  errors::check_condition(d->type == value_type::vector, "Value is not a vector, it is {}", d->type);
  return std::get<value_vector>(d->value_container);
}

std::string value::to_json() const
{
  std::stringstream ss;
  ss.precision(15);
  d->to_json(ss);
  return ss.str();
}

value value::from_json(const std::string& _json)
{
  json_reader reader{std::stringstream(_json), 0};
  return reader.start();
}
