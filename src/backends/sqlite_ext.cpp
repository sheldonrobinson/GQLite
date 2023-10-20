#include <sqlite3.h>

#include <algorithm>
#include <limits>
#include <ranges>

#include <gqlite.h>

#include "../logging.h"
#include "../gqlite_p.h"
#include "../global.h"

#include "sqlite_queries.h"

namespace gqlite::backends
{
  template<typename... _TArgs_>
  void report_error(sqlite3_context* _context, exception_code _code, const std::string& _format, const _TArgs_&... _args)
  {
    value_map err;
    err["code"] = int64_t(_code);
    err["message"] = format_string(_format, _args...);
    std::string s_err = "__gqlite_fun: " + value(err).to_json();
    sqlite3_result_error(_context, s_err.c_str(), s_err.size());
  }
  void report_exception(sqlite3_context* _context, const exception& _ex)
  {
    value_map err;
    err["code"] = int64_t(get_execption_code(_ex));
    err["message"] = get_execption_message(_ex);
    std::string s_err = "__gqlite_fun: " + value(err).to_json();
    sqlite3_result_error(_context, s_err.c_str(), s_err.size());
  }
#define SAFE(_X_)                             \
  try { _X_; }                                \
  catch(const exception& ex)                  \
  {                                           \
    report_exception(_context, ex); return;   \
  }
#define NULL_CHECK(_ARGS_COUNT_)                      \
  for(std::size_t i = 0; i < _ARGS_COUNT_; ++i)       \
  {                                                   \
    if(sqlite3_value_type(_argv[i]) == SQLITE_NULL)   \
    {                                                 \
      sqlite3_result_null(_context); return ;         \
    }                                                 \
  }

  void report_value(sqlite3_context* _context, const value& _val)
  {
    switch(_val.get_type())
    {
      case value_type::boolean:
        sqlite3_result_int(_context, _val.to_bool() ? 1 : 0);
        break;
      case value_type::integer:
        sqlite3_result_int64(_context, _val.to_integer());
        break;
      case value_type::floating_point:
        sqlite3_result_double(_context, _val.to_double());
        break;
      case value_type::string:
      {
        std::string str = _val.to_string();
        sqlite3_result_text(_context, str.c_str(), str.size(), SQLITE_TRANSIENT);
        break;
      }
      case value_type::map:
      case value_type::vector:
      {
        std::string str = _val.to_json();
        sqlite3_result_text(_context, str.c_str(), str.size(), SQLITE_TRANSIENT);
        break;
      }
      case value_type::invalid:
      {
        sqlite3_result_null(_context);
        break;
      }
    }
  }
  enum class comparison_result
  {
    true_result, false_result, compared_null
  };
  comparison_result compare(const gqlite::value& _left, const gqlite::value& _right)
  {
    using enum value_type;
    if(_left.get_type() == invalid  or _right.get_type() == invalid)
    {
      return comparison_result::compared_null;
    }
    if(_left.get_type() != _right.get_type()) return comparison_result::false_result;
    bool comp = false;
    bool compared_to_null = false;
    switch(_left.get_type())
    {
    case invalid:
      break;
    case boolean:
      comp = _left.to_bool() == _right.to_bool();
      break;
    case integer:
      comp = _left.to_integer() == _right.to_integer();
      break;
    case floating_point:
      comp = _left.to_double() == _right.to_double();
      break;
    case string:
      comp = _left.to_string() == _right.to_string();
      break;
    case vector:
      {
        value_vector vv_left = _left.to_vector();
        value_vector vv_right = _right.to_vector();
        if(vv_left.size() != vv_right.size()) return comparison_result::false_result;
        comp = true;
        for(std::size_t idx = 0; idx < vv_left.size(); ++idx)
        {
          const value& l = vv_left[idx];
          const value& r = vv_right[idx];
          comparison_result l_r_cr = compare(l, r);
          // if(l.get_type() == invalid)
          // {
          //   compared_to_null = true;
          // }
          switch(l_r_cr)
          {
          case comparison_result::true_result:
            break;
          case comparison_result::compared_null:
            compared_to_null = true;
            break;
          case comparison_result::false_result:
            comp = false;
            break;
          }
          if(not comp) break;
        }
      }
      break;
    case map:
      comp = _left.to_map() == _right.to_map();
      break;
    }
    // this is opencypher madness, if you compare [nil, 2] with [1, 2] this should return null because of the nil/1 comparison,
    // however [nil, 2] with [1, "2"] should return false because 2 != "2" :facepalm:
    if(not comp) return comparison_result::false_result;
    return (compared_to_null) ? comparison_result::compared_null : comparison_result::true_result;
  }
  comparison_result contains(const gqlite::value_vector& _container, const gqlite::value& _value)
  {
    using enum value_type;
    if(_value.get_type() == invalid)
    {
      return comparison_result::compared_null;
    }
    bool has_compared_to_null = false;
    for(const value& v_c : _container)
    {
      switch(compare(v_c, _value))
      {
      case comparison_result::true_result:
        return comparison_result::true_result;
      case comparison_result::false_result:
        break;
      case comparison_result::compared_null:
        has_compared_to_null = true;
        break;
      }
    }
    return has_compared_to_null ? comparison_result::compared_null : comparison_result::false_result;
  }
  bool contains_null(const gqlite::value_vector& _v)
  {
    for(const value& val : _v)
    {
      if(val.get_type() == value_type::invalid) return true;
    }
    return false;
  }
  /**
   * @internal
   * class use to read json files
   */
  struct is_valid_json
  {
    struct invalid
    {};
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
          throw invalid();
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
    bool start()
    {
      try
      {
        fetch_next_non_space_char();
        validate_value();
        return true;
      } catch(const invalid&)
      {
        return false;
      }
    }
    static bool validate(const char* _string)
    {
      is_valid_json ivj{std::stringstream(_string), 0};
      return ivj.start();
    }
    /// @brief read a string
    /// @return the string without quotation
    void validate_string()
    {
      is_char('"');
      fetch_next_char();
      bool keep_nc = false; // this is used to indicate if the last character was a '\' or not
      while(keep_nc or last_char != '"')
      {
        keep_nc = (not keep_nc and last_char == '\\');
        fetch_next_char();
      }
      fetch_next_char(); // eat the '"'
    }
    /// @brief throw an exception if last_char different from @param _c 
    void is_char(int _c)
    {
      if(last_char != _c) throw invalid();
    }
    /// @brief read the next value
    /// @return and return it
    void validate_value()
    {
      switch (last_char)
      {
      case '{':
      {
        // Parse object
        fetch_next_non_space_char();
        while(last_char != '}')
        {
          validate_string();
          is_char(':');
          fetch_next_non_space_char();
          validate_value();
          if(last_char == ',')
          {
            fetch_next_non_space_char();
          } else {
            break;
          }
        }
        is_char('}');
        fetch_next_non_space_char();
        return;
      }
      case '[':
      {
        // Parse array
        fetch_next_non_space_char();
        while(last_char != ']')
        {
          validate_value();
          if(last_char == ',')
          {
            fetch_next_non_space_char();
          } else {
            break;
          }
        }
        is_char(']');
        fetch_next_non_space_char();
        return;
      }
      case '"':
        validate_string();
        return;
      case 't':
      {
        // Parse true
        fetch_next_non_space_char(); is_char('r');
        fetch_next_non_space_char(); is_char('u');
        fetch_next_non_space_char(); is_char('e');
        fetch_next_non_space_char();
        return;
      }
      case 'f':
      {
        // Parse false
        fetch_next_non_space_char(); is_char('a');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char(); is_char('s');
        fetch_next_non_space_char(); is_char('e');
        fetch_next_non_space_char();
        return;
      }
      case 'n':
      {
        // Parse null
        fetch_next_non_space_char(); is_char('u');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char(); is_char('l');
        fetch_next_non_space_char();
        return;
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
        fetch_next_non_space_char();
        bool is_integer = true;
        while((last_char >= '0' and last_char <= '9') or last_char == '.' or last_char == 'e' or last_char == '+' or last_char == '-')
        {
          is_integer = is_integer and (last_char != '.' and last_char != 'e');
          fetch_next_non_space_char();
        }
        return;
      }
      default:
        throw invalid{};
      }
    }
  };
  void gqlite_next_uid(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    // We get our db connection from the context
    sqlite3 *db = sqlite3_context_db_handle(_context);
    const char* label = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    sqlite3_stmt *ps;
    int rc = sqlite3_prepare_v2(db, sqlite_queries::uid_next().c_str(), -1, &ps, 0);
    if (rc != SQLITE_OK) {
      // Something bad happened, we'll come back to look at this later
      return;
    }
    sqlite3_bind_text(ps, 1, label, -1, SQLITE_STATIC);

    rc = sqlite3_step(ps);
    if (rc == SQLITE_ROW)
    {
      gqlite_debug("counter {} = {}", label, sqlite3_column_int64(ps, 0));
      sqlite3_result_int64(_context, sqlite3_column_int64(ps, 0));
    } else {
      report_error(_context, gqlite::exception_code::internal_error, "No uid counter for '{}'.", label);
    }
    sqlite3_finalize(ps);    
  }
  void gqlite_jsonify(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    constexpr int JSON_TYPE = 74;
    if(sqlite3_value_subtype(_argv[0]) == JSON_TYPE)
    {
      sqlite3_result_subtype(_context, JSON_TYPE);
      sqlite3_result_text(_context, reinterpret_cast<const char*>(sqlite3_value_text(_argv[0])), -1, SQLITE_TRANSIENT);
    } else {
      switch(sqlite3_value_type(_argv[0]))
      {
        case SQLITE_NULL:
          sqlite3_result_null(_context);
          break;
        case SQLITE_INTEGER:
          sqlite3_result_int64(_context, sqlite3_value_int64(_argv[0]));
          break;
        case SQLITE_FLOAT:
          sqlite3_result_double(_context, sqlite3_value_double(_argv[0]));
          break;
        case SQLITE_TEXT:
        {
          const char* str =reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
          sqlite3_result_text(_context, str, -1, SQLITE_TRANSIENT);
          if(is_valid_json::validate(str))
          {
            sqlite3_result_subtype(_context, JSON_TYPE);
          }
        }
          break;
        default:
          report_error(_context, exception_code::invalid_argument_type, "Invalid arguments for addition.");
      }
    }
  }
  void gqlite_range(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    int start = sqlite3_value_int(_argv[0]);
    int end = sqlite3_value_int(_argv[1]);
    value_vector vv;
    for(int64_t i = start; i <= end; ++i)
    {
      vv.push_back(i);
    }
    std::string str = value(vv).to_json().c_str();
    sqlite3_result_text(_context, str.c_str(), str.size(), SQLITE_TRANSIENT);
  }
  void gqlite_concat(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    std::string left = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    std::string right = reinterpret_cast<const char*>(sqlite3_value_text(_argv[1]));

    try
    {
      value_vector left_v = value::from_json(left).to_vector();
      value right_v = value::from_json(right);
      if(right_v.get_type() == value_type::vector)
      {
        value_vector right_vv = right_v.to_vector();
        left_v.insert(left_v.end(), right_vv.begin(), right_vv.end());
      } else {
        left_v.push_back(right_v);
      }
        std::string str = value(left_v).to_json();
        sqlite3_result_text(_context, str.c_str(), str.size(), SQLITE_TRANSIENT);
    } catch(const exception& _ex)
    {
      report_error(_context, exception_code::unspecified, _ex.what());
    }
  }
  void gqlite_addition(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    int left_type = sqlite3_value_type(_argv[0]);
    int right_type = sqlite3_value_type(_argv[1]);
    if(left_type == right_type)
    {
      switch(left_type)
      {
        case SQLITE_INTEGER:
          sqlite3_result_int64(_context, sqlite3_value_int64(_argv[0]) + sqlite3_value_int64(_argv[1]));
          break;
        case SQLITE_FLOAT:
          sqlite3_result_double(_context, sqlite3_value_double(_argv[0]) + sqlite3_value_double(_argv[1]));
          break;
        case SQLITE_TEXT:
        {
          std::string left_s(reinterpret_cast<const char*>(sqlite3_value_text(_argv[0])));
          std::string right_s(reinterpret_cast<const char*>(sqlite3_value_text(_argv[1])));
          std::string res = left_s + right_s;
          sqlite3_result_text(_context, res.c_str(), res.size(), SQLITE_TRANSIENT);
        }
          break;
        default:
          report_error(_context, exception_code::invalid_argument_type, "Invalid arguments for addition.");
      }
    } else {
      double left_v = 0.0, right_v = 0.0;
      switch(left_type)
      {
        case SQLITE_INTEGER:
          left_v = sqlite3_value_int64(_argv[0]);
          break;
        case SQLITE_FLOAT:
          left_v = sqlite3_value_double(_argv[0]);
          break;
        default:
          report_error(_context, exception_code::invalid_argument_type, "Inavlid left argument for addition.");
          return;
      }
      switch(right_type)
      {
        case SQLITE_INTEGER:
          right_v = sqlite3_value_int64(_argv[1]);
          break;
        case SQLITE_FLOAT:
          right_v = sqlite3_value_double(_argv[1]);
          break;
        default:
          report_error(_context, exception_code::invalid_argument_type, "Inavlid right argument for addition.");
          return;
      }
      sqlite3_result_double(_context, left_v + right_v);
    }
  }
  void gqlite_indexed_access(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    std::string left = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    value val = value::from_json(left);
    switch(val.get_type())
    {
      case value_type::vector:
      {
        std::size_t idx = sqlite3_value_int(_argv[1]);
        value_vector vv = val.to_vector();
        if(idx >= vv.size())
        {
          report_error(_context, exception_code::unspecified, "Index out of bound.");
          return;
        }
        val = vv[idx];
        break;
      }
      case value_type::map:
      {
        std::string key = reinterpret_cast<const char*>(sqlite3_value_text(_argv[1]));
        val = val.to_map()[key];
        break;
      }
      default:
      {
        report_error(_context, exception_code::invalid_argument_type, "Invalid index access to non map/vector value.");
        return;
      }
    }
    report_value(_context, val);
  }
  void gqlite_range_access(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    std::string left = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    value val;
    SAFE(val = value::from_json(left));
    switch(val.get_type())
    {
      case value_type::vector:
      {
        if(sqlite3_value_type(_argv[2]) == SQLITE_NULL or sqlite3_value_type(_argv[1]) == SQLITE_NULL)
        {
          val = value();
        } else {
          value_vector vv = val.to_vector();
          int64_t idx_first = sqlite3_value_int64(_argv[1]);
          int64_t idx_second = sqlite3_value_int64(_argv[2]);
          int64_t v_size = vv.size();
          if(idx_first <= idx_second and idx_first < v_size)
          {
            if(idx_second == std::numeric_limits<int64_t>::max())
            {
              idx_second = vv.size();
            }
            if(idx_first < 0)
            {
              idx_first = 0;
            }
            if(idx_second > v_size)
            {
              idx_second = v_size;
            }
            while(idx_second < 0)
            {
              idx_second += v_size;
            }
            value_vector vv_out;
            for(int64_t i = idx_first; i < idx_second; ++i)
            {
              vv_out.push_back(vv[i]);
            }
            val = vv_out;
          } else {
            val = value_vector();
          }
        }
        break;
      }
      default:
      {
        report_error(_context, exception_code::invalid_argument_type, "Invalid range access to non vector value.");
        return;
      }
    }
    report_value(_context, val);
  }
  void gqlite_contains(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    NULL_CHECK(2);
    std::string left = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    value val_left;
    SAFE(val_left = value::from_json(left));
    bool contains_result = false;
    switch(val_left.get_type())
    {
      case value_type::vector:
      {
        std::string right = reinterpret_cast<const char*>(sqlite3_value_text(_argv[1]));
        value_vector vv = val_left.to_vector();
        value val_right;
        SAFE(val_right = value::from_json(right));

        switch(contains(vv, val_right))
        {
        case comparison_result::true_result:
          contains_result = true;
          break;
        case comparison_result::false_result:
          contains_result = false;
          break;
        case comparison_result::compared_null:
          sqlite3_result_null(_context);
          return;
        }
        break;
      }
      case value_type::map:
      {
        std::string key = reinterpret_cast<const char*>(sqlite3_value_text(_argv[1]));
        value_map map = val_left.to_map();
        auto kv = workarounds::views::keys(map);
        contains_result = std::ranges::find(kv, key) != kv.end();
        break;
      }
      default:
      {
        report_error(_context, exception_code::invalid_argument_type, "Invalid IN operator on non map/vector value.");
        return;
      }
    }
    sqlite3_result_int(_context, contains_result ? 1 : 0);
  }
  void gqlite_tail(sqlite3_context* _context,int,sqlite3_value** _argv)
  {
    NULL_CHECK(1);
    std::string left = reinterpret_cast<const char*>(sqlite3_value_text(_argv[0]));
    value val_left;
    SAFE(val_left = value::from_json(left));
    if(val_left.get_type() == value_type::vector)
    {
      value_vector vv = val_left.to_vector();
      if(vv.empty())
      {
        sqlite3_result_null(_context);
      } else {
        report_value(_context, value_vector(std::next(vv.begin()), vv.end()));
      }
    } else {
      report_error(_context, exception_code::invalid_argument_type, "Invalid argument to function tail, list was expected.");
    }
  }

  void initialise_sqlite_ext(sqlite3* db)
  {
    sqlite3_create_function(db, "gqlite_jsonify", 1, SQLITE_UTF8, nullptr, &gqlite_jsonify, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_next_uid", 1, SQLITE_UTF8, nullptr, &gqlite_next_uid, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_range", 2, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_range, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_concat", 2, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_concat, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_addition", 2, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_addition, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_indexed_access", 2, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_indexed_access, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_range_access", 3, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_range_access, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_contains", 2, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_contains, nullptr, nullptr);
    sqlite3_create_function(db, "gqlite_tail", 1, SQLITE_DETERMINISTIC | SQLITE_UTF8, nullptr, &gqlite_tail, nullptr, nullptr);
  }
}