#ifndef _OC_ALGEBRA_ABSTRACT_NODE_VISITOR_H_
#define _OC_ALGEBRA_ABSTRACT_NODE_VISITOR_H_

#include <any>
#include <vector>
#include <unordered_map>

#include "nodes.h"

namespace gqlite::oc::algebra
{
  namespace details
  {
    class abstract_node_visitor_adaptor
    {
    public:
      abstract_node_visitor_adaptor();
      ~abstract_node_visitor_adaptor();
  #define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                         \
      virtual void call_visit(_KLASS_NAME_ ## _csp _node, void* _r, void* _parameter) = 0;
  #include "nodes_defs.h"
  #undef OC_ALGEBRA_GENERATE
    // protected:
      // void accept(node_csp _node, void* _r, void* _parameter)
      // {
      //   return accept(_r, _node.ge(), _parameter);
      // }
      // void accept(const node* _node, void* _r, void* _parameter)
      // {
      //   if(_node)
      //   {
      //     _node->accept(this, _r, _parameter);
      //   }
      // }
    };    
  }
  
  template<typename _TR_, typename... _TArgs_>
  class abstract_node_visitor : details::abstract_node_visitor_adaptor
  {
    friend class node;
  public:
    using ParametersTuple = std::tuple<_TArgs_...>;
    static constexpr std::size_t ParametersCount = sizeof...(_TArgs_);
    using ReturnType = _TR_;
  public:
    abstract_node_visitor() {}
    ~abstract_node_visitor() {}
  public:
    /**
     * Call this function to start visiting
     */
    _TR_ start(node_csp _node, const _TArgs_&... _args)
    {
      return accept(_node, _args...);
    }
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                           \
  private:                                                                                                \
    virtual _TR_ visit(_KLASS_NAME_ ## _csp _node, const _TArgs_&... _parameter) = 0;                      \
    virtual void call_visit(_KLASS_NAME_ ## _csp _node, void* _r, void* _parameter) override               \
    {                                                                                                     \
      typedef _TR_ (abstract_node_visitor::* F)(_KLASS_NAME_ ## _csp _node, const _TArgs_&...);              \
      *reinterpret_cast<_TR_*>(_r) = std::apply(F(&abstract_node_visitor::visit), std::tuple_cat(std::make_tuple(this, _node),   \
                                                *reinterpret_cast<ParametersTuple*>(_parameter)));        \
    }
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
  protected:
    _TR_ accept(node_csp _node, const _TArgs_... _arguments)
    {
      return accept(_node.get(), _arguments...);
    }
    _TR_ accept(const node* _node, const _TArgs_... _arguments)
    {
      _TR_ r = _TR_();
      if(_node)
      {
        ParametersTuple pt = std::make_tuple(_arguments...);
        _node->accept(this, &r, &pt);
      }
      return r;
    }
  };
  
  template<typename... _TArgs_>
  class abstract_node_visitor<void, _TArgs_...> : details::abstract_node_visitor_adaptor
  {
    friend class node;
  public:
    using ParametersTuple = std::tuple<_TArgs_...>;
    static constexpr std::size_t ParametersCount = sizeof...(_TArgs_);
    using ReturnType = void;
  public:
    abstract_node_visitor() {}
    ~abstract_node_visitor() {}
  public:
    /**
     * Call this function to start visiting
     */
    void start(node_csp _node, const _TArgs_&... _args)
    {
      accept(_node, _args...);
    }
#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                                                                 \
  private:                                                                                                                      \
    virtual void visit(_KLASS_NAME_ ## _csp _node, const _TArgs_&... _parameter) = 0;                                            \
    virtual void call_visit(_KLASS_NAME_ ## _csp _node, void*, void* _parameter) override                                        \
    {                                                                                                                           \
      typedef void (abstract_node_visitor::* F)(_KLASS_NAME_ ## _csp _node, const _TArgs_&...);                                    \
      std::apply(F(&abstract_node_visitor::visit), std::tuple_cat(std::make_tuple(this, _node), *reinterpret_cast<ParametersTuple*>(_parameter) ));    \
    }
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
  protected:
    void accept(node_csp _node, const _TArgs_&... _arguments)
    {
      accept(_node.get(), _arguments...);
    }
    void accept(const node* _node, const _TArgs_&... _arguments)
    {
      if(_node)
      {
        ParametersTuple pt = std::make_tuple(_arguments...);
        _node->accept(this, nullptr, &pt);
      }
    }
  };
}

#endif
