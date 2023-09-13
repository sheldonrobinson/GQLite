#include "functions.h"

#include <functional>

#include "../gqlite_p.h"
#include "../errors.h"

namespace gqlite::backends::functions
{
  struct static_data
  {
    static_data();

    static void check_value_is_map(const char* _fname, const value& _value);

    static gqlite::value type(const std::vector<value>& _arguments);
    static gqlite::value labels(const std::vector<value>& _arguments);
    static gqlite::value size(const std::vector<value>& _arguments);
    static gqlite::value keys(const std::vector<value>& _arguments);
    std::unordered_map<std::string, std::function<value(const std::vector<value>&)>> functions;
  };

  void static_data::check_value_is_map(const char* _fname, const value& _value)
  {
    if(_value.get_type() != value_type::map)
    {
      throw exception("'{}' expected a map.", _fname);
    }
  }

  /**
   * @return the type of a relationship, i.e., the label
   */
  value static_data::type(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("type", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::map)
    {
      value_map map = arg0.to_map();
      auto it = map.find("label");
      if(it == map.end())
      {
        return value();
      } else {
        return it->second;
      }
    } else {
      return value();
    }
  }
  /**
   * @return the labels of a node
   */
  value static_data::labels(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("labels", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::map)
    {
      value_map map = arg0.to_map();
      auto it = map.find("labels");
      if(it == map.end())
      {
        return value();
      } else {
        return it->second;
      }
    } else {
      return value();
    }
  }
  /**
   * @return the keys of a map
   */
  value static_data::keys(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("labels", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::map)
    {
      value_map map = arg0.to_map();
      value_vector out;
      for(auto it = map.begin(); it != map.end(); ++it)
      {
        out.push_back(it->first);
      }
      return out;
    } else {
      return value();
    }
  }
  /**
   * @return the keys of a vector
   */
  value static_data::size(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("labels", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::vector)
    {
      return arg0.to_vector();
    } else {
      return value();
    }
  }
  static_data::static_data()
  {
    functions["type"] = &static_data::type;
    functions["labels"] = &static_data::labels;
    functions["size"] = &static_data::size;
    functions["keys"] = &static_data::keys;
  }

  value call(const std::string& _name, const std::vector<value>& _arguments)
  {
    static static_data sd;
    auto it = sd.functions.find(_name);
    if(it == sd.functions.end())
    {
      throw exception("No function {}.", _name);
    } else {
      return it->second(_arguments);
    }
  }
}

