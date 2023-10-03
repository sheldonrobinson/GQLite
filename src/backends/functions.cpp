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

    static value type(const std::vector<value>& _arguments);
    static value labels(const std::vector<value>& _arguments);
    static value size(const std::vector<value>& _arguments);
    static value keys(const std::vector<value>& _arguments);
    static value id(const std::vector<value>& _arguments);
    static value range(const std::vector<value>& _arguments);
    static value coalesce(const std::vector<value>& _arguments);
    static value properties(const std::vector<value>& _arguments);
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
      errors::check_argument_type_function<std::string>(map["type"].to_string(), "edge", "type");
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
      errors::check_argument_type_function<std::string>(map["type"].to_string(), "node", "type");
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
   * @return the properties of a node/edge
   */
  value static_data::properties(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("properties", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::map)
    {
      value_map map = arg0.to_map();
      auto it = map.find("properties");
      if(it == map.end())
      {
        return value();
      } else {
        return it->second;
      }
    } else if(arg0.get_type() == value_type::invalid)
    {
      return value();
    } else {
      errors::invalid_argument_type("properties function expect node or edge.");
    }
  }
  /**
   * @return the id of a node/edge
   */
  value static_data::id(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("id", _arguments, 1);
    value arg0 = _arguments[0];
    if(arg0.get_type() == value_type::map)
    {
      value_map map = arg0.to_map();
      auto it = map.find("id");
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
  /**
   * @return an array of value between start and end
   */
  value static_data::range(const std::vector<value>& _arguments)
  {
    errors::check_arguments_size("range", _arguments, 2);
    int start = _arguments[0].to_integer();
    int end = _arguments[1].to_integer();
    value_vector v;
    for(int i = start; i <= end; ++i) v.push_back(i);
    return v;
  }
  value static_data::coalesce(const std::vector<value>& _arguments)
  {
    for(const value& v : _arguments)
    {
      if(v.get_type() != value_type::invalid)
      {
        return v;
      }
    }
    return value();
  }
  static_data::static_data()
  {
    functions["type"] = &static_data::type;
    functions["labels"] = &static_data::labels;
    functions["size"] = &static_data::size;
    functions["keys"] = &static_data::keys;
    functions["id"] = &static_data::id;
    functions["range"] = &static_data::range;
    functions["coalesce"] = &static_data::coalesce;
    functions["properties"] = &static_data::properties;
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

