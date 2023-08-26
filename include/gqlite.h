#include <exception>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

namespace gqlite
{
  class backend;
  class exception : public std::exception
  {
  public:
    exception(const std::string& _error) : m_error(_error), m_c_error(m_error.c_str()) {}
    exception(const char* _error) : m_c_error(_error) {}
    const char* what() const throw() override
    {
      return m_c_error;
    }
  private:
    std::string m_error;
    const char* m_c_error;
  };
  class value
  {
  public:
    value();
    value(const value& _rhs);
    value& operator=(const value& _rhs);
    ~value();
  public:
    double to_double() const;
    std::string to_string() const;
    std::unordered_map<std::string, value> to_map() const;
    std::vector<value> to_vector() const;
  private:
    struct data;
    std::shared_ptr<data> d;
  };
  class database
  {
    database(backend* _backend);
  public:
    database();
    database(const database& _rhs);
    database& operator=(const database& _rhs);
    ~database();
    static database create_from_sqlite(void*);
    static database create_from_sqlite_file(const std::string& _filename);
  public:
    value execute_oc_query(const std::string& _string, const std::unordered_map<std::string, value>& _variant = std::unordered_map<std::string, value>());
  private:
    struct data;
    std::shared_ptr<data> d;
  };
}
