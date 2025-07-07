INSERT INTO gqlite_{{ graph_name }}_edges (edge_key, labels, properties, left, right)
VALUES (
    ?1, ?2, ?3,
    (SELECT id FROM gqlite_{{ graph_name }}_nodes WHERE node_key = ?4),
    (SELECT id FROM gqlite_{{ graph_name }}_nodes WHERE node_key = ?5)
);
