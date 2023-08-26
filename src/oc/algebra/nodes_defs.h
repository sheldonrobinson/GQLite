#define OC_ALGEBRA_GRAPH_NODE_MEMBERS(_KLASS_NAME_, F)          \
  F(_KLASS_NAME_, std::string, variable)                        \
  F(_KLASS_NAME_, std::vector<std::string>, labels)             \
  F(_KLASS_NAME_, std::unordered_map<GQLITE_LIST(std::string, std::any)>, properties)

OC_ALGEBRA_GENERATE(graph_node, OC_ALGEBRA_GRAPH_NODE_MEMBERS)

#define OC_ALGEBRA_CREATE_NODES_QUERY_MEMBERS(_KLASS_NAME_, F) \
  F(_KLASS_NAME_, std::vector<graph_node_csp>, nodes)

OC_ALGEBRA_GENERATE(create_nodes, OC_ALGEBRA_CREATE_NODES_QUERY_MEMBERS)
