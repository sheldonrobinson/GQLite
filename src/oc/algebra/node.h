#ifndef _OC_ALGEBRA_ABSTRACT_NODE_H_
#define _OC_ALGEBRA_ABSTRACT_NODE_H_

#include <memory>
#include "node_type.h"
#include "../../global.h"

namespace gqlite::oc::algebra
{
  template<typename _TR_, typename... _TArgs_>
  class abstract_node_visitor;
  namespace details
  {
    class abstract_node_visitor_adaptor;
  }
  
  class node : public std::enable_shared_from_this<node>
  {
    template<typename _TR_, typename... _TArgs_>
    friend class abstract_node_visitor;
  protected:
    struct data;
    data* d;
    node(node_type _type, data* );
  public:
    virtual ~node();
  public:
    node_type get_type() const;

  protected:
    virtual void accept(details::abstract_node_visitor_adaptor* _node, void* _r, void* _parameter) const = 0;
  };
  typedef std::shared_ptr<const node> node_csp;
  template<typename... _T_>
  class alternative
  {
    static_assert(std::conjunction_v<std::is_base_of<node, _T_>...> , "All types should subclass node");
  public:
    alternative() : m_type(node_type::__Invalid__)
    {
    }
    template<typename _To_, std::enable_if_t<traits::contains_v<_To_, _T_...>, int> = 0>
    alternative(std::shared_ptr<const _To_> _v) : m_node(_v), m_type(_v->type())
    {}
    template<typename _To_, std::enable_if_t<traits::contains_v<_To_, _T_...>, int> = 0>
    alternative(const _To_* _v) : m_node(_v), m_type(_v->type())
    {}
    node_type get_type() const { return m_type; }
    template<typename _To_, std::enable_if_t<traits::contains_v<_To_, _T_...>, int> = 0>
    std::shared_ptr<const _To_> get_value() const
    {
      return std::static_pointer_cast<const _To_>(m_node);
    }
    bool is_valid() const { return m_type != node_type::__Invalid__; }
    node_csp get_node() const { return m_node; }
  private:
    node_csp m_node;
    node_type m_type;
  };
}

#endif
