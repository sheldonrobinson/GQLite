SELECT nas.node as node_map, ROW_NUMBER() over () as node_rowid
  FROM gqlite_/%= _graph_name %/_nodes_as_json nas
  JOIN json_each(?/%= to_string_fixed_width(_idx, 3) %/) s ON s.value = nas.id
