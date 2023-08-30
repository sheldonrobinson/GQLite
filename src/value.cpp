#include "gqlite.h"

#include <sstream>
#include <variant>

#include "gqlite_p.h"

using namespace gqlite;

const char* std::to_string(value_type _type)
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

struct value::data
{
  value_type type;
  std::variant<bool, int, double, std::string, std::unordered_map<std::string, value>, std::vector<value>> value_container;

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
    void fetch_next_char()
    {
      while(std::isspace(last_char = stream.get()))
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
    /// @brief start parsing
    /// @throw gqlite::exception
    /// @return the parsed value
    gqlite::value start()
    {
      fetch_next_char();
      return read_value();
    }
    std::string read_string()
    {
      is_char('"');
      fetch_next_char();
      std::string string;
      bool keep_nc = false;
      while(keep_nc or last_char != '"')
      {
        if(not keep_nc and last_char == '\\')
        {
          keep_nc = true;
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
        if(not stream.good())
        {
          throw gqlite::exception("Unifinished string: {}", string);
        }
        fetch_next_char();
      }
      fetch_next_char(); // eat the '"'
      return string;
    }
    void is_char(int _c)
    {
      if(last_char != _c)
      {
        throw gqlite::exception("Expected '{}' but got '{}'", char(_c), char(last_char));
      }
    }
    gqlite::value read_value()
    {
      switch (last_char)
      {
      case '{':
      {
        // Parse object
        std::unordered_map<std::string, value> map;
        fetch_next_char();
        while(last_char != '}')
        {
          std::string str = read_string();
          is_char(':');
          fetch_next_char();
          gqlite::value val = read_value();
          map[str] = val;
          if(last_char == ',')
          {
            fetch_next_char();
          } else {
            break;
          }
        }
        is_char('}');
        fetch_next_char();
        return map;
      }
      case '[':
      {
        // Parse array
        std::vector<value> vec;
        fetch_next_char();
        while(last_char != ']')
        {
          gqlite::value val = read_value();
          vec.push_back(val);
          if(last_char == ',')
          {
            fetch_next_char();
          } else {
            break;
          }
        }
        is_char(']');
        fetch_next_char();
        return vec;
      }
      case '"':
        return read_string();
      case 't':
      {
        // Parse true
        fetch_next_char(); is_char('r');
        fetch_next_char(); is_char('u');
        fetch_next_char(); is_char('e');
        return true;
      }
      case 'f':
      {
        // Parse false
        fetch_next_char(); is_char('a');
        fetch_next_char(); is_char('l');
        fetch_next_char(); is_char('s');
        fetch_next_char(); is_char('e');
        return false;
      }
      case 'n':
      {
        // Parse null
        fetch_next_char(); is_char('u');
        fetch_next_char(); is_char('l');
        fetch_next_char(); is_char('l');
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
        fetch_next_char();
        bool is_integer = true;
        while((last_char >= '0' and last_char <= '9') or last_char == '.' or last_char == 'e' or last_char == '+' or last_char == '-')
        {
          is_integer = is_integer and (last_char != '.' and last_char != 'e');
          number += char(last_char);
          fetch_next_char();
        }
        if(is_integer)
        {
          return std::stoi(number);
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
  case value_type::integer:
    _stream << std::get<int>(value_container);
    break;
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

value::value() : d(new data{value_type::invalid})
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

value::value(int _v) : d(new data{value_type::integer, _v})
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
    case value_type::integer:
      return std::get<int>(d->value_container) != 0;
    case value_type::number:
      return std::get<double>(d->value_container) != 0.0;
    default:
      throw exception(format_string("Value is not a bool, it is {}", d->type));
  }
}

int value::to_integer() const
{
  switch(d->type)
  {
    case value_type::integer:
      return std::get<int>(d->value_container);
    case value_type::number:
      return std::get<double>(d->value_container);
    default:
      throw exception(format_string("Value is not a number, it is {}", d->type));
  }
}

double value::to_double() const
{
  switch(d->type)
  {
    case value_type::integer:
      return std::get<int>(d->value_container);
    case value_type::number:
      return std::get<double>(d->value_container);
    default:
      throw exception(format_string("Value is not a number, it is {}", d->type));
  }
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

value value::from_json(const std::string& _json)
{
  json_reader reader{std::stringstream(_json)};
  return reader.start();
}
