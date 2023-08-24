#include "sqlite.h"

using namespace gqlite::backends;


sqlite::sqlite(void* _db)
{}
sqlite::~sqlite()
{}
sqlite* sqlite::from_file(const std::string& _filename)
{

  return new sqlite(nullptr);
}
gqlite::result sqlite::execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, std::any>& _variant)
{
  return gqlite::result::from_error("not implemented");
}
