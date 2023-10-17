DELETE FROM gqlite_/%= _graph_name %/_labels WHERE node_id IN (/%= _what %/);
DELETE FROM gqlite_/%= _graph_name %/_nodes WHERE id IN (/%= _what %/)