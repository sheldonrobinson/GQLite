UPDATE gqlite_{{ graph_name }}_edges
SET labels = ?2,
    properties = ?3
WHERE edge_key = ?1;
