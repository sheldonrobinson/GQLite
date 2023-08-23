#include <tuple>
#include "token.h"
#include "algebra/node.h"

namespace gqlite::oc
{
  class lexer;
  class parser
  {
  public:
    parser(lexer* _lexer);
    ~parser();
    algebra::node_csp parse();
    std::string get_error() const;
  private:
    struct data;
    data* const d;
  };
} // namespace gqlite::oc
