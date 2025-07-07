SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_nodes
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_edges
UNION ALL SELECT COUNT(*) FROM gqlite_/%= _graph_name %/_labels
UNION ALL SELECT COUNT(j.value) FROM gqlite_/%= _graph_name %/_nodes t, json_each(properties) j
UNION ALL SELECT COUNT(j.value) FROM gqlite_/%= _graph_name %/_edges t, json_each(properties) j
UNION ALL SELECT COUNT(*) FROM gqlite_labels
UNION ALL SELECT COUNT(DISTINCT label) FROM gqlite_/%= _graph_name %/_labels
UNION ALL SELECT COUNT(DISTINCT label) FROM (SELECT label FROM gqlite_/%= _graph_name %/_edges UNION SELECT label FROM gqlite_/%= _graph_name %/_labels)
UNION ALL SELECT COUNT(DISTINCT label) FROM gqlite_/%= _graph_name %/_labels
