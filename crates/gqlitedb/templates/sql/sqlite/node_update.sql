UPDATE gqlite_{{ graph_name }}_nodes
SET labels = :labels,
    properties = :properties
WHERE node_key = :key;
