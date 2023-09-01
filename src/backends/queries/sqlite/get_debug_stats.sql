SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_nodes
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_edges
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_labels
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_nodes WHERE properties != '{}'
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_edges WHERE properties != '{}'
UNION ALL SELECT COUNT(*) FROM gqlite_labels
