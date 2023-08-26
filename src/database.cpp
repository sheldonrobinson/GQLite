#include "backends/sqlite.h"

#include <sstream>

#include "logging.h"
#include "oc/lexer.h"
#include "oc/parser.h"

using namespace gqlite;

struct database::data
{
  ~data()
  {
    delete backend_;
  }
  backend* backend_ = nullptr;
};

database::database()
{}

database::database(const database& _rhs) : d(_rhs.d)
{
}

database::database(backend* _backend) : d(new data{_backend})
{
}

database& database::operator=(const database& _rhs)
{
  d = _rhs.d;
  return *this;
}

database::~database()
{
}

database database::create_from_sqlite(void* _sqlite_database)
{
  return database(new backends::sqlite(_sqlite_database));
}

database database::create_from_sqlite_file(const std::string& _filename)
{
  return database(backends::sqlite::from_file(_filename));
}

value database::execute_oc_query(const std::string& _string, const std::unordered_map<std::string, value>& _bindings)
{
  std::stringstream ss(_string);
  oc::lexer l(&ss);
  oc::parser parser(&l);
  oc::algebra::node_csp node = parser.parse();
  return d->backend_->execute_oc_query(node, _bindings);
}
