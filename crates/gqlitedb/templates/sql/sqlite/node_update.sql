UPDATE gqlite_{{ graph_name }}_nodes
SET labels = ?2,
    properties = ?3
WHERE node_key = ?1;
