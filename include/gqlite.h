#include <any>
#include <unordered_map>
#include <memory>
#include <string>

namespace gqlite
{
  class backend;
  class result
  {
  public:

    enum status
    {
      success, failure
    };
  public:
    result();
    result(const result& _rhs);
    result& operator=(const result& _rhs);
    ~result();
    status get_status() const;
    std::string get_error() const;
    std::any value();
  private:
    struct data;
    std::shared_ptr<data> const d;
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
  public:
    result execute_oc_query(const std::string& _string, const std::unordered_map<std::string, std::any>& _variant = std::unordered_map<std::string, std::any>());
  private:
    struct data;
    std::shared_ptr<data> const d;
  };
}
