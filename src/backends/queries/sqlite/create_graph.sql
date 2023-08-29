CREATE TABLE gqlite_/%= _name %/_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL);
CREATE TABLE gqlite_/%= _name %/_edges(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL,
       left INTEGER,
       right INTEGER,
       FOREIGN KEY(left) REFERENCES /%= _name %/_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_/%= _name %/_nodes(id));
CREATE TABLE gqlite_/%= _name %/_labels(label TEXT NOT NULL, node_id INTEGER)