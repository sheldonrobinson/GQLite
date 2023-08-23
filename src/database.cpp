#include "backends/sqlite.h"

using namespace gqlite;

struct database::data
{
  ~data()
  {
    if(backend_)
    {
      backend_->close();
    }
  }
  backend* backend_;
};

database::database(backend* _backend) : d(new data{_backend})
{

}

database::~database()
{
}

database database::create_from_sqlite(void* _sqlite_database)
{
  return database(new backends::sqlite(_sqlite_database));
}

result database::execute_oc_query(const std::string& _string, const std::unordered_map<std::string, std::any>& _variant)
{

}
