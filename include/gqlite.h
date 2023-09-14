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
  /**
   * Represents an error that occurs during the execution of a query.
   */
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
    /**
     * @return the error message
     */
    const char* what() const throw() override
    {
      return m_c_error;
    }
  private:
    std::string m_error;
    const char* m_c_error;
  };
  /**
   * Represent the value type.
   */
  enum class value_type
  {
    invalid,
    boolean,
    integer,
    number, ///< aka float
    string,
    map,
    vector
  };
  class value;
  using value_map = std::unordered_map<std::string, value>;
  using value_vector = std::vector<value>;
  /**
   * Represent a value (boolean, integer, number, string, map or vector)
   */
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
    /**
     * @return the type hold by this value
     */
    value_type get_type() const;
    /**
     * Attempt to return a bool. Throw an exception if not possible.
     */
    bool to_bool() const;
    /**
     * Attempt to return an integer. Throw an exception if not possible.
     */
    int to_integer() const;
    /**
     * Attempt to return a double. Throw an exception if not possible.
     */
    double to_double() const;
    /**
     * Attempt to return a string. Throw an exception if not possible.
     */
    std::string to_string() const;
    /**
     * Attempt to return a map. Throw an exception if not possible.
     */
    value_map to_map() const;
    /**
     * Attempt to return a vector. Throw an exception if not possible.
     */
    value_vector to_vector() const;
  public:
    /**
     * Attempt to return a json string to represent a value.
     */
    std::string to_json() const;
    /**
     * Construct a value from a json string. Throw an exception if not possible.
     */
    static value from_json(const std::string& _json);
  private:
    struct data;
    std::shared_ptr<data> d;
  };
  /**
   * Main class for connecting to a gqlite database.
   */
  class connection
  {
    connection(backend* _backend);
  public:
    connection();
    connection(const connection& _rhs);
    connection& operator=(const connection& _rhs);
    ~connection();
    /**
     * Create a sqlite connection from a \p _pointer to a valid sqlite handle.
     */
    static connection create_from_sqlite(void* _pointer, const value& _options = value());
    /**
     * Create a sqlite connection from a file.
     */
    static connection create_from_sqlite_file(const std::string& _filename, const value& _options = value());
    /**
     * @internal
     * Part of the private API, what is returned by this function may change at any time.
     */
    gqlite::value get_debug_stats() const;
  public:
    /**
     * Execute a query.
     */
    value execute_oc_query(const std::string& _string, const value_map& _variant = {});
  private:
    struct data;
    std::shared_ptr<data> d;
  };
}
