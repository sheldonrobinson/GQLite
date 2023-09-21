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
    value execute_oc_query(oc::algebra::node_csp _node, const value_map& _variant) override;
  private:
    struct data;
    data* const d;
  };
}
