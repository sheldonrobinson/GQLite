#include <tuple>
#include "token.h"
#include "algebra/node.h"

namespace gqlite::oc
{
  class lexer;
  class parser
  {
  public:
    parser(lexer* _lexer, const value_map& _bindings);
    ~parser();
    algebra::node_csp parse();
  private:
    struct data;
    data* const d;
  };
} // namespace gqlite::oc
