// Queries

#define OC_ALGEBRA_CREATE_NODES_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<graph_node_csp>, nodes)

OC_ALGEBRA_GENERATE(create_nodes, OC_ALGEBRA_CREATE_NODES_MEMBERS)

#define OC_ALGEBRA_MATCH_NODES_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<graph_node_csp>, nodes)

OC_ALGEBRA_GENERATE(match_nodes, OC_ALGEBRA_MATCH_NODES_MEMBERS)

// Statements

#define OC_ALGEBRA_STATEMENTS_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<node_csp>, nodes)

OC_ALGEBRA_GENERATE(statements, OC_ALGEBRA_STATEMENTS_MEMBERS)

#define OC_ALGEBRA_RETURN_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<std::string>, variables)

OC_ALGEBRA_GENERATE(return_statement, OC_ALGEBRA_RETURN_MEMBERS)

// Expressions

#define OC_ALGEBRA_GRAPH_NODE_MEMBERS(_KLASS_NAME_, F)          \
  F(_KLASS_NAME_, std::string, variable)                        \
  F(_KLASS_NAME_, std::vector<std::string>, labels)             \
  F(_KLASS_NAME_, std::unordered_map<GQLITE_LIST(std::string, node_csp)>, properties)

OC_ALGEBRA_GENERATE(graph_node, OC_ALGEBRA_GRAPH_NODE_MEMBERS)

#define OC_ALGEBRA_VALUE_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, gqlite::value, value)

OC_ALGEBRA_GENERATE(value, OC_ALGEBRA_VALUE_MEMBERS)
