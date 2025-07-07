SELECT COUNT(*) FROM gqlite_{{ graph_name }}_edges AS e
  JOIN gqlite_{{ graph_name }}_nodes AS n ON n.id = e.left OR n.id = e.right
  WHERE hex(n.node_key) in ('{{ keys | join("', '") }}')