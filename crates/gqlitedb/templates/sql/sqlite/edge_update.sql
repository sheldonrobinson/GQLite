UPDATE gqlite_{{ graph_name }}_edges
SET labels = :labels,
    properties = :properties
WHERE edge_key = :key;
