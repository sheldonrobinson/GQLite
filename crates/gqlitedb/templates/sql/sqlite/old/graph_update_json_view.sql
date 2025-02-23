-- view for getting the nodes as a json representation

DROP VIEW IF EXISTS gqlite_/%= _graph_name %/_nodes_as_json;
CREATE VIEW gqlite_/%= _graph_name %/_nodes_as_json (id, node) AS
        SELECT id, json_object('id', id, 'properties', json(properties), 
          'labels', (SELECT json_group_array(value) FROM json_each(labels) AS result WHERE value IS NOT NULL),
          'type', 'node') FROM
          (SELECT n.id AS id, n.properties AS properties, json_group_array(labels.label) AS labels
            FROM gqlite_/%= _graph_name %/_nodes n
            LEFT JOIN gqlite_/%= _graph_name %/_labels as l ON l.node_id = n.id
            LEFT JOIN gqlite_labels AS labels ON labels.id = l.label
            GROUP BY n.id)
        UNION SELECT NULL, NULL;

DROP VIEW IF EXISTS gqlite_/%= _graph_name %/_edges_as_json;
CREATE VIEW gqlite_/%= _graph_name %/_edges_as_json (id, edge) AS
        SELECT e.id, json_object('id', e.id, 'properties', json(e.properties), 
          'label', label.label,
          'type', 'edge') FROM
          gqlite_/%= _graph_name %/_edges e
          JOIN gqlite_labels label ON label.id = e.label
        UNION SELECT NULL, NULL
