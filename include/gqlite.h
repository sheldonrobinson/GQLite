#pragma once

#include <exception>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

namespace gqlite
{
  class backend;
  struct debug_stats;
  class exception : public std::exception
  {
  public:
    exception(const std::string& _error) : m_error(_error), m_c_error(m_error.c_str()) {}
    exception(const char* _error) : m_c_error(_error) {}
    template<typename _T_, typename... _TOther_>
    inline exception(const char* _format, const _T_& _value, const _TOther_&... _other);
    template<typename _T_, typename... _TOther_>
    inline exception(const std::string& _format, const _T_& _value, const _TOther_&... _other);
    exception(const exception& _rhs);
    exception& operator=(const exception& _rhs);
    const char* what() const throw() override
    {
      return m_c_error;
    }
  private:
    std::string m_error;
    const char* m_c_error;
  };
  template<typename _TMessage_>
  void check_condition(bool _condition, _TMessage_ _message)
  {
    if(not _condition)
    {
      throw exception(_message);
    }
  }
  enum class value_type
  {
    invalid,
    boolean,
    integer,
    number,
    string,
    map,
    vector
  };
  class value;
  using value_map = std::unordered_map<std::string, value>;
  using value_vector = std::vector<value>;
  class value
  {
  public:
    value();
    value(const value& _rhs);
    value& operator=(const value& _rhs);
    ~value();
  public:
    value(int _v);
    value(double _v);
    value(const std::string& _v);
    value(const value_map& _v);
    /// not part of the public API
    template<typename _T_>
    value(const std::initializer_list<typename std::unordered_map<std::string, _T_>::value_type>& _v);
    value(const value_vector& _v);
    /// not part of the public API
    template<typename _T_>
    value(const std::vector<_T_>& _v);
    bool operator==(const value& _rhs) const;
  public:
    value_type get_type() const;
    bool to_bool() const;
    int to_integer() const;
    double to_double() const;
    std::string to_string() const;
    value_map to_map() const;
    value_vector to_vector() const;
  public:
    std::string to_json() const;
    static value from_json(const std::string& _json);
  private:
    struct data;
    std::shared_ptr<data> d;
  };
  class connection
  {
    connection(backend* _backend);
  public:
    connection();
    connection(const connection& _rhs);
    connection& operator=(const connection& _rhs);
    ~connection();
    static connection create_from_sqlite(void*, const value& _options = value());
    static connection create_from_sqlite_file(const std::string& _filename, const value& _options = value());
    /**
     * @internal
     * Part of the private API, what is returned by this function may change at any time.
     */
    gqlite::value get_debug_stats() const;
  public:
    value execute_oc_query(const std::string& _string, const value_map& _variant = {});
  private:
    struct data;
    std::shared_ptr<data> d;
  };
}

namespace std
{
  const char* to_string(gqlite::value_type _type);
}
