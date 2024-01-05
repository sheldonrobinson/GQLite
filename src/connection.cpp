#include "backends/sqlite.h"

#include <sstream>

#include "logging.h"
#include "oc/lexer.h"
#include "oc/parser.h"

using namespace gqlite;

struct connection::data
{
  ~data()
  {
    delete backend_;
  }
  backend* backend_ = nullptr;
};

connection::connection()
{}

connection::connection(const connection& _rhs) : d(_rhs.d)
{
}

connection::connection(backend* _backend) : d(new data{_backend})
{
}

connection& connection::operator=(const connection& _rhs)
{
  d = _rhs.d;
  return *this;
}

connection::~connection()
{
}

connection connection::create_from_sqlite(void* _sqlite_connection, const gqlite::value&)
{
  return connection(new backends::sqlite(_sqlite_connection));
}

connection connection::create_from_sqlite_file(const std::string& _filename, const gqlite::value&)
{
  return connection(backends::sqlite::from_file(_filename));
}

value connection::execute_oc_query(const std::string& _string, const value_map& _bindings)
{
#if 0
  std::cout << "============= execute_oc_query =============\n"
            << _string <<
             "\n============================================" << std::endl;
#endif
  std::stringstream ss(_string);
  oc::lexer l(&ss);
  oc::parser parser(&l, _bindings);
  oc::algebra::node_csp node = parser.parse();
  return d->backend_->execute_oc_query(node);
}
