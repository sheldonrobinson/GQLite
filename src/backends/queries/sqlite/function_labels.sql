(CASE WHEN /%= _node_id %/ IS NULL
  THEN NULL
  ELSE
    (SELECT json_group_array(labels.label)
                FROM gqlite_/%= _graph_name %/_labels as l
                JOIN gqlite_labels AS labels ON labels.id = l.label WHERE l.node_id = /%= _node_id %/)
  END)
