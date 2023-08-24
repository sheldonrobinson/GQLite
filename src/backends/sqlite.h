#include "../backend.h"

namespace gqlite::backends
{
  class sqlite : public backend
  {
  public:
    sqlite(void* _db);
    virtual ~sqlite();
    static sqlite* from_file(const std::string& _filename);
  public:
    result execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, std::any>& _variant) override;
  };
}
