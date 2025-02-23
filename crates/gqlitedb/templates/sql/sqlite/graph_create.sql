CREATE TABLE gqlite_{{ graph_name }}_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, key BLOB NOT NULL, labels TEXT NOT NULL, properties TEXT NOT NULL);
CREATE TABLE gqlite_{{ graph_name }}_edges(id INTEGER PRIMARY KEY AUTOINCREMENT,
        key BLOB NOT NULL, 
        labels TEXT NOT NULL,
        properties TEXT NOT NULL,
        left INTEGER,
        right INTEGER,
        FOREIGN KEY(label) REFERENCES gqlite_labels(id),
        FOREIGN KEY(left) REFERENCES gqlite_{{ graph_name }}_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_{{ graph_name }}_nodes(id));

---------------- views ----------------

-- view for querying for undirected edges
CREATE VIEW gqlite_{{ graph_name }}_edges_undirected (id, labels, properties, left, right) AS
        SELECT id, label, properties, left, right FROM gqlite_{{ graph_name }}_edges
        UNION SELECT id, label, properties, right, left FROM gqlite_{{ graph_name }}_edges;

