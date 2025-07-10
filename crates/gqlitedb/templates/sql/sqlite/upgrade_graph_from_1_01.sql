-- Step 1: Create new nodes table
CREATE TABLE gqlite_{{ graph_name }}_nodes_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_key BLOB NOT NULL DEFAULT '',
    labels TEXT NOT NULL,
    properties TEXT NOT NULL
);

-- Step 2: Create new edges table
CREATE TABLE gqlite_{{ graph_name }}_edges_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    edge_key BLOB NOT NULL DEFAULT '',
    labels TEXT NOT NULL,
    properties TEXT NOT NULL,
    left INTEGER NOT NULL,
    right INTEGER NOT NULL,
    FOREIGN KEY(left) REFERENCES gqlite_{{ graph_name }}_nodes_new(id),
    FOREIGN KEY(right) REFERENCES gqlite_{{ graph_name }}_nodes_new(id)
);

-- Step 3: Migrate nodes with labels as JSON array
INSERT INTO gqlite_{{ graph_name }}_nodes_new (id, node_key, labels, properties)
SELECT 
    n.id,
    CAST(uuid() AS BLOB) AS node_key,
    COALESCE(json_group_array(gl.label), '[]') AS labels,
    n.properties
FROM gqlite_{{ graph_name }}_nodes n
LEFT JOIN gqlite_{{ graph_name }}_labels nl ON nl.node_id = n.id
LEFT JOIN gqlite_labels gl ON gl.id = nl.label
GROUP BY n.id;

-- Step 4: Migrate edges, converting label to text
INSERT INTO gqlite_{{ graph_name }}_edges_new (id, edge_key, labels, properties, left, right)
SELECT 
    e.id,
    CAST(uuid() AS BLOB) AS edge_key,
    COALESCE(json_group_array(gl.label), '[]') AS labels,
    e.properties,
    e.left,
    e.right
FROM gqlite_{{ graph_name }}_edges e
LEFT JOIN gqlite_labels gl ON gl.id = e.label
GROUP BY e.id;

-- Step 5: Drop old views and tables
DROP VIEW IF EXISTS gqlite_{{ graph_name }}_nodes_as_json;
DROP VIEW IF EXISTS gqlite_{{ graph_name }}_edges_as_json;
DROP VIEW IF EXISTS gqlite_{{ graph_name }}_edges_undirected;

DROP TABLE gqlite_{{ graph_name }}_labels;
DROP TABLE gqlite_{{ graph_name }}_edges;
DROP TABLE gqlite_{{ graph_name }}_nodes;

-- Step 6: Rename new tables
ALTER TABLE gqlite_{{ graph_name }}_nodes_new RENAME TO gqlite_{{ graph_name }}_nodes;
ALTER TABLE gqlite_{{ graph_name }}_edges_new RENAME TO gqlite_{{ graph_name }}_edges;

-- Step 7: Recreate the updated undirected edges view
CREATE VIEW gqlite_{{ graph_name }}_edges_undirected (
    id, edge_key, labels, properties, left, right, reversed
) AS
    SELECT id, edge_key, labels, properties, left, right, 0 FROM gqlite_{{ graph_name }}_edges
    UNION
    SELECT id, edge_key, labels, properties, right, left, 1 FROM gqlite_{{ graph_name }}_edges;
