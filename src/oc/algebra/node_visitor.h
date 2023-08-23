#include "abstract_node_visitor.h"

namespace gqlite::oc::algebra
{
  namespace details
  {
    template<typename _TVisitor_, typename _T_>
    struct Call
    {
      template<int ...S>
      static _TVisitor_::ReturnType accept(_TVisitor_* _visitor, const _T_& _t, const _TVisitor_::ParametersTuple& _parameters, std::index_sequence<S...>) {
        return _visitor->visit(_t, std::get<S>(_parameters)...);
      }
    };
    template<typename _TVisitor_>
    struct Call<_TVisitor_, NodeCSP>
    {
      template<int ...S>
      static _TVisitor_::ReturnType accept(_TVisitor_* _visitor, const NodeCSP& _t, const _TVisitor_::ParametersTuple& _parameters, std::index_sequence<S...>) {
        return _visitor->accept(_t, std::get<S>(_parameters)...);
      }
    };

    // TODO move to knowCore/Global.h? And check if equivalent is available in newer GCC
    template < template <typename...> class Template, typename T >
    struct is_specialisation_of : std::false_type {};

    template < template <typename...> class Template, typename... Args >
    struct is_specialisation_of< Template, Template<Args...> > : std::true_type {};
    
    template< template <typename...> class Template, typename... Args >
    inline constexpr bool is_specialisation_of_v = is_specialisation_of<Template, Args...>::value;
    
    template<typename _T_>
    struct RemoveConstExplicitlySharedDataPointer { typedef _T_ type; };
    template<typename _T_>
    struct RemoveConstExplicitlySharedDataPointer<knowCore::ConstExplicitlySharedDataPointer<_T_>> { typedef _T_ type; };
    
    template<typename _T_>
    using RemoveConstExplicitlySharedDataPointerT = typename RemoveConstExplicitlySharedDataPointer<_T_>::type;
    
    template<typename _TVisitor_, typename _T_, typename = void>
    struct VisitorHelper;
    
    template<typename _TVisitor_, typename _T_>
    struct VisitorHelper<_TVisitor_, knowCore::ConstExplicitlySharedDataPointer<_T_>, std::enable_if_t<std::is_base_of_v<Node, _T_>>>
    {
      static _TVisitor_::ReturnType call(_TVisitor_* _visitor, const knowCore::ConstExplicitlySharedDataPointer<_T_>& _t, const _TVisitor_::ParametersTuple& _parameters)
      {
        if(_t)
        {
          return Call<_TVisitor_, knowCore::ConstExplicitlySharedDataPointer<_T_>>::accept(_visitor, _t, _parameters, std::make_index_sequence<_TVisitor_::ParametersCount>());
        }
        return typename _TVisitor_::ReturnType();
      }
    };
    template<typename _TVisitor_, typename _T_>
    struct VisitorHelper<_TVisitor_, _T_, std::enable_if_t<not std::is_base_of_v<Node, RemoveConstExplicitlySharedDataPointerT<_T_>>
                                              and not is_specialisation_of_v<QList, _T_>
                                              and not is_specialisation_of_v<Alternative, _T_>>>
    {
      static _TVisitor_::ReturnType call(_TVisitor_* _visitor, const _T_& _t, const _TVisitor_::ParametersTuple& _parameters)
      {
        Q_UNUSED(_t);
        Q_UNUSED(_visitor);
        Q_UNUSED(_parameters);
        return typename _TVisitor_::ReturnType();
      }
    };
    template<typename _TVisitor_, typename... _T_>
    struct VisitorHelper<_TVisitor_, Alternative<_T_...>>
    {
      static _TVisitor_::ReturnType call(_TVisitor_* _visitor, const Alternative<_T_...>& _t, const _TVisitor_::ParametersTuple& _parameters)
      {
        if(_t.node())
        {
          return Call<_TVisitor_, NodeCSP>::accept(_visitor, _t.node(), _parameters, std::make_index_sequence<_TVisitor_::ParametersCount>());
        } else {
          return typename _TVisitor_::ReturnType();
        }
      }
    };
    template<typename _TVisitor_, typename _T_>
    struct VisitorHelper<_TVisitor_, QList<_T_>, std::enable_if_t<std::is_base_of_v<Node, RemoveConstExplicitlySharedDataPointerT<_T_>>
                                                      or is_specialisation_of_v<Alternative, _T_>>>
    {
      static _TVisitor_::ReturnType call(_TVisitor_* _visitor, const QList<_T_>& _t, const _TVisitor_::ParametersTuple& _parameters)
      {
        for(const _T_& t : _t)
        {
          VisitorHelper<_TVisitor_, _T_>::call(_visitor, t, _parameters);
        }
        return typename _TVisitor_::ReturnType();
      }
    };
    template<typename _TVisitor_, typename _T_>
    struct VisitorHelper<_TVisitor_, QList<_T_>, std::enable_if_t<not std::is_base_of_v<Node, RemoveConstExplicitlySharedDataPointerT<_T_>> and not is_specialisation_of_v<Alternative, _T_> >>
    {
      static _TVisitor_::ReturnType call(_TVisitor_* _visitor, const QList<_T_>& _t, const _TVisitor_::ParametersTuple& _parameters)
      {
        Q_UNUSED(_t);
        Q_UNUSED(_visitor);
        Q_UNUSED(_parameters);
        return typename _TVisitor_::ReturnType();
      }
    };
  }
  // TODO rename to TraverseNodeVisitor?
  template<typename _TR_, typename... _TArgs_>
  class NodeVisitor : public abstract_node_visitor<_TR_, _TArgs_...>
  {
    template<typename _TVisitor_, typename _T_>
    friend struct details::Call;

  public:
    NodeVisitor() {}
    ~NodeVisitor() {}
  private:
#define GQL_GENERATE_VISIT_CALL(_KLASS_NAME_, _TYPE_, _NAME_)        \
  details::VisitorHelper<NodeVisitor<_TR_, _TArgs_...>, _TYPE_>::call(this, _node->_NAME_(), std::make_tuple(_parameters...));

#define OC_ALGEBRA_GENERATE(_KLASS_NAME_, _MEMBER_DEF_)                       \
    _TR_ visit(_KLASS_NAME_ ## CSP _node, const _TArgs_&... _parameters) override     \
    {                                                                                 \
      _MEMBER_DEF_(_KLASS_NAME_, GQL_GENERATE_VISIT_CALL)                      \
      return _TR_();                                                                  \
    }
#include "nodes_defs.h"
#undef OC_ALGEBRA_GENERATE
#undef GQL_GENERATE_VISIT_CALL
  };
}
