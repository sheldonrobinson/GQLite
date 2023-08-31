CREATE TABLE gqlite_/%= _graph_name %/_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL);
CREATE TABLE gqlite_/%= _graph_name %/_edges(id INTEGER PRIMARY KEY AUTOINCREMENT,
       label INTEGER,
       properties TEXT NOT NULL,
       left INTEGER,
       right INTEGER,
       FOREIGN KEY(label) REFERENCES gqlite_labels(id),
       FOREIGN KEY(left) REFERENCES gqlite_/%= _graph_name %/_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_/%= _graph_name %/_nodes(id));
CREATE TABLE gqlite_/%= _graph_name %/_labels(label INTEGER, node_id INTEGER,
        FOREIGN KEY(label) REFERENCES gqlite_labels(id), FOREIGN KEY(node_id) REFERENCES gqlite_/%= _graph_name %/_nodes(id))