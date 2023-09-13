// Queries

#define OC_ALGEBRA_CREATE_NODES_MEMBERS(_KLASS_NAME_, F)                          \
  F(_KLASS_NAME_, std::vector<alternative<GQLITE_LIST(graph_node, graph_edge)>>, patterns)

OC_ALGEBRA_GENERATE(create, OC_ALGEBRA_CREATE_NODES_MEMBERS)

#define OC_ALGEBRA_MATCH_NODES_MEMBERS(_KLASS_NAME_, F)                                     \
  F(_KLASS_NAME_, std::vector<alternative<GQLITE_LIST(graph_node, graph_edge)>>, patterns)  \
  F(_KLASS_NAME_, node_csp, where)

OC_ALGEBRA_GENERATE(match, OC_ALGEBRA_MATCH_NODES_MEMBERS)

// Statements

#define OC_ALGEBRA_NODES_LIST_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<node_csp>, nodes)

OC_ALGEBRA_GENERATE(statements, OC_ALGEBRA_NODES_LIST_MEMBERS)

#define OC_ALGEBRA_RETURN_MEMBERS(_KLASS_NAME_, F)                \
  F(_KLASS_NAME_, bool, all)                                      \
  F(_KLASS_NAME_, std::vector<named_expression_csp>, expressions)

OC_ALGEBRA_GENERATE(return_statement, OC_ALGEBRA_RETURN_MEMBERS)

#define OC_ALGEBRA_WITH_MEMBERS(_KLASS_NAME_, F)                  \
  F(_KLASS_NAME_, bool, all)                                      \
  F(_KLASS_NAME_, std::vector<named_expression_csp>, expressions)

OC_ALGEBRA_GENERATE(with, OC_ALGEBRA_WITH_MEMBERS)

#define OC_ALGEBRA_DELETE_MEMBERS(_KLASS_NAME_, F)  \
  F(_KLASS_NAME_, bool, detach)                     \
  F(_KLASS_NAME_, std::vector<node_csp>, expressions)

OC_ALGEBRA_GENERATE(delete_statement, OC_ALGEBRA_DELETE_MEMBERS)

OC_ALGEBRA_GENERATE(set, OC_ALGEBRA_NODES_LIST_MEMBERS)
OC_ALGEBRA_GENERATE(remove, OC_ALGEBRA_NODES_LIST_MEMBERS)

// Set/remove Expressions

#define OC_ALGEBRA_SET_ADD_VALUE_MEMBERS(_KLASS_NAME_, F)                 \
  F(_KLASS_NAME_, member_access_csp, target)                              \
  F(_KLASS_NAME_, node_csp, expression)

OC_ALGEBRA_GENERATE(set_property, OC_ALGEBRA_SET_ADD_VALUE_MEMBERS)


#define OC_ALGEBRA_SET_EQUAL_MEMBERS(_KLASS_NAME_, F)                     \
  F(_KLASS_NAME_, member_access_csp, target)                              \
  F(_KLASS_NAME_, nocde_csp, expression)

OC_ALGEBRA_GENERATE(add_property, OC_ALGEBRA_SET_ADD_VALUE_MEMBERS)

#define OC_ALGEBRA_REMOVE_PROPERTY_MEMBERS(_KLASS_NAME_, F)                   \
  F(_KLASS_NAME_, member_access_csp, target)

OC_ALGEBRA_GENERATE(remove_property, OC_ALGEBRA_REMOVE_PROPERTY_MEMBERS)

#define OC_ALGEBRA_EDIT_LABELS_MEMBERS(_KLASS_NAME_, F)                   \
  F(_KLASS_NAME_, std::string, target)                                    \
  F(_KLASS_NAME_, std::vector<std::string>, labels)

OC_ALGEBRA_GENERATE(edit_labels, OC_ALGEBRA_EDIT_LABELS_MEMBERS)


// Values

#define OC_ALGEBRA_GRAPH_NODE_MEMBERS(_KLASS_NAME_, F)          \
  F(_KLASS_NAME_, std::string, variable)                        \
  F(_KLASS_NAME_, std::vector<std::string>, labels)             \
  F(_KLASS_NAME_, map_csp, properties)

OC_ALGEBRA_GENERATE(graph_node, OC_ALGEBRA_GRAPH_NODE_MEMBERS)

#define OC_ALGEBRA_GRAPH_EDGE_MEMBERS(_KLASS_NAME_, F)          \
  F(_KLASS_NAME_, std::string, variable)                        \
  F(_KLASS_NAME_, graph_node_csp, source)                       \
  F(_KLASS_NAME_, graph_node_csp, destination)                  \
  F(_KLASS_NAME_, edge_directivity, directivity)                \
  F(_KLASS_NAME_, std::vector<std::string>, labels)             \
  F(_KLASS_NAME_, map_csp, properties)

OC_ALGEBRA_GENERATE(graph_edge, OC_ALGEBRA_GRAPH_EDGE_MEMBERS)

#define OC_ALGEBRA_VALUE_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, gqlite::value, value)

OC_ALGEBRA_GENERATE(value, OC_ALGEBRA_VALUE_MEMBERS)


#define OC_ALGEBRA_MAP_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::unordered_map<GQLITE_LIST(std::string, node_csp)>, map)

OC_ALGEBRA_GENERATE(map, OC_ALGEBRA_MAP_MEMBERS)

#define OC_ALGEBRA_ARRAY_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<node_csp>, array)

OC_ALGEBRA_GENERATE(array, OC_ALGEBRA_ARRAY_MEMBERS)


// Expressions

#define OC_ALGEBRA_NAMED_EXPRESSION(_KLASS_NAME_, F)      \
  F(_KLASS_NAME_, std::string, name)                      \
  F(_KLASS_NAME_, node_csp, expression)

OC_ALGEBRA_GENERATE(named_expression, OC_ALGEBRA_NAMED_EXPRESSION)

#define OC_ALGEBRA_VARIABLE_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::string, identifier)

OC_ALGEBRA_GENERATE(variable, OC_ALGEBRA_VARIABLE_MEMBERS)

#define OC_ALGEBRA_MEMBER_ACCESS_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::string, left)                      \
  F(_KLASS_NAME_, std::vector<std::string>, path)

OC_ALGEBRA_GENERATE(member_access, OC_ALGEBRA_MEMBER_ACCESS_MEMBERS)

#define OC_ALGEBRA_HAS_LABELS_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::string, left)                   \
  F(_KLASS_NAME_, std::vector<std::string>, labels)

OC_ALGEBRA_GENERATE(has_labels, OC_ALGEBRA_HAS_LABELS_MEMBERS)

#define OC_ALGEBRA_FUNCTION_CALL_EXPRESSION(_KLASS_NAME_, F)  \
  F(_KLASS_NAME_, std::string, name)                          \
  F(_KLASS_NAME_, std::vector<node_csp>, arguments)

OC_ALGEBRA_GENERATE(function_call, OC_ALGEBRA_FUNCTION_CALL_EXPRESSION)

#define OC_ALGEBRA_BINARY_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, node_csp, left)                  \
  F(_KLASS_NAME_, node_csp, right)

OC_ALGEBRA_GENERATE(logical_and, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(logical_or, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_equal, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_different, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_inferior, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_superior, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_inferior_equal, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_superior_equal, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_in, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(relational_not_in, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(addition, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(substraction, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(multiplication, OC_ALGEBRA_BINARY_MEMBERS)
OC_ALGEBRA_GENERATE(division, OC_ALGEBRA_BINARY_MEMBERS)

#define OC_ALGEBRA_UNARY_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, node_csp, value)

OC_ALGEBRA_GENERATE(logical_negation, OC_ALGEBRA_UNARY_MEMBERS)
OC_ALGEBRA_GENERATE(negation, OC_ALGEBRA_UNARY_MEMBERS)
OC_ALGEBRA_GENERATE(is_null, OC_ALGEBRA_UNARY_MEMBERS)
OC_ALGEBRA_GENERATE(is_not_null, OC_ALGEBRA_UNARY_MEMBERS)
