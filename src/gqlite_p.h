#pragma once

#include <gqlite.h>

#include "format.h"

namespace gqlite
{
  enum class exception_stage
  {
    unspecified,
    compiletime,
    runtime
  };
  enum class exception_code
  {
    unspecified,
    // Generic error code
    internal_error, ///< this should never happen
    invalid_json, ///< error that occurs when parsing json
    table_error,
    value_error, ///< generic error that occurs in the value class
    unimplemented_error,
    // OpenCypher error code
    column_name_conflict = 100,
    integer_overflow,
    invalid_argument_type,
    invalid_delete,
    invalid_number_literal,
    invalid_number_of_arguments,
    invalid_parameter_use,
    negative_integer_argument,
    non_constant_expression,
    parse_error,
    requires_directed_relationship,
    undefined_variable,
    unexpected_syntax,
    unknown_function,
    variable_already_bound,
    variable_type_conflict
  };

  exception_code get_execption_code(const exception& _exception);
  std::string get_execption_message(const exception& _exception);
  [[noreturn]] void throw_exception(exception_stage _stage, exception_code _code, const std::string& _error);
  [[noreturn]] void throw_exception(exception_stage _stage, exception_code _code, const char* _error);
  [[noreturn]] void rethrow_exception(exception_stage _stage, const exception& _ex);
  template<typename _T_, typename... _TOther_>
  [[noreturn]] inline void throw_exception(exception_stage _stage, exception_code _code, const char* _format, const _T_& _value, const _TOther_&... _other)
  {
    throw_exception(_stage, _code, format_string(_format, _value, _other...));
  }
  template<typename _T_, typename... _TOther_>
  [[noreturn]] inline void throw_exception(exception_stage _stage, exception_code _code, const std::string& _format, const _T_& _value, const _TOther_&... _other)
  {
    throw_exception(_stage, _code, format_string(_format, _value, _other...));
  }
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

  template<typename _T_>
  inline value::value(const std::initializer_list<typename std::unordered_map<std::string, _T_>::value_type>& _v) : value()
  {
    std::unordered_map<std::string, _T_> m{_v};
    std::unordered_map<std::string, _T_> mo;
    for(auto const&[k, v] : m)
    {
      mo[k] = v;
    }
    *this = value(mo);
  }
  template<typename _T_>
  inline value::value(const std::vector<_T_>& _v) : value()
  {
    value_vector vo;
    for(const _T_& t : _v)
    {
      vo.push_back(t);
    }
    *this = value(vo);
  }
  inline value remove_invalid_in_map(const value& _value)
  {
    switch(_value.get_type())
    {
      using enum value_type;
      case invalid:
      case boolean:
      case integer:
      case floating_point:
      case string:
        return _value;
      case map:
      {
        value_map vm;
        for(const auto& [k, v] : _value.to_map())
        {
          if(v.get_type() != invalid)
          {
            vm[k] = remove_invalid_in_map(v);
          }
        }
        return vm;
      }
      case vector:
      {
        value_vector vv;
        for(const value& v : _value.to_vector())
        {
          if(v.get_type() != invalid)
          {
            vv.push_back(remove_invalid_in_map(v));
          }
        }
        return vv;
      }
    }
    throw_exception(exception_stage::unspecified, exception_code::internal_error, "Unknown value type for {}.", _value);
  }
}
