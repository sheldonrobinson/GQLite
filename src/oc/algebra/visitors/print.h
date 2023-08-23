#include "../abstract_node_visitor.h"

namespace gqlite::oc::algebra::visitors
{
  namespace details
  {
    template<typename _T_, typename = void>
    struct print_helper;
  }
  class print : public abstract_node_visitor<void>
  {
    template<typename _T_, typename = void>
    friend class details::print_helper;
  public:
    print();
    ~print();
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_) \
  virtual void visit(_KLASS_NAME_ ## _csp _node) override;
#include "../nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
  private:
    struct data;
    data* const d;
  };
}
