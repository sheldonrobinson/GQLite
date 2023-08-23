#include "../backend.h"

namespace gqlite::backends
{
  class sqlite : public backend
  {
  public:
    sqlite(void* _db);
    virtual ~sqlite();
  public:
    void close() override;
    result execute_oc_query(oc::algebra::node_csp _node, const std::unordered_map<std::string, std::any>& _variant) override;
  };
}
