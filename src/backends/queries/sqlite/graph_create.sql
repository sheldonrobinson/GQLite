CREATE TABLE gqlite_/%= _graph_name %/_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL);
CREATE TABLE gqlite_/%= _graph_name %/_edges(id INTEGER PRIMARY KEY AUTOINCREMENT,
        label INTEGER,
        properties TEXT NOT NULL,
        left INTEGER,
        right INTEGER,
        FOREIGN KEY(label) REFERENCES gqlite_labels(id),
        FOREIGN KEY(left) REFERENCES gqlite_/%= _graph_name %/_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_/%= _graph_name %/_nodes(id));
CREATE TABLE gqlite_/%= _graph_name %/_labels(label INTEGER, node_id INTEGER,
        FOREIGN KEY(label) REFERENCES gqlite_labels(id), FOREIGN KEY(node_id) REFERENCES gqlite_/%= _graph_name %/_nodes(id));

---------------- views ----------------

-- view for querying for undirected edges
CREATE VIEW gqlite_/%= _graph_name %/_edges_undirected (id, label, properties, left, right) AS
        SELECT id, label, properties, left, right FROM gqlite_/%= _graph_name %/_edges
        UNION SELECT id, label, properties, right, left FROM gqlite_/%= _graph_name %/_edges;

-- view for getting the nodes as a json representation
CREATE VIEW gqlite_/%= _graph_name %/_nodes_as_json (id, node) AS
        SELECT id, json_object('id', id, 'properties', json(properties), 
          'labels', (SELECT json_group_array(value) FROM json_each(labels) AS result WHERE value IS NOT NULL),
          'type', 'node') FROM
          (SELECT n.id AS id, n.properties AS properties, json_group_array(labels.label) AS labels
            FROM gqlite_/%= _graph_name %/_nodes n
            LEFT JOIN gqlite_/%= _graph_name %/_labels as l ON l.node_id = n.id
            LEFT JOIN gqlite_labels AS labels ON labels.id = l.label
            GROUP BY n.id);

CREATE VIEW gqlite_/%= _graph_name %/_edges_as_json (id, edge) AS
        SELECT e.id, json_object('id', e.id, 'properties', json(e.properties), 
          'label', label.label,
          'type', 'edge') FROM
          gqlite_/%= _graph_name %/_edges e
          JOIN gqlite_labels label ON label.id = e.label

