WITH
  node_labels AS (
    SELECT DISTINCT value AS label
    FROM gqlite_{{ graph_name }}_nodes, json_each(gqlite_{{ graph_name }}_nodes.labels)
  ),
  node_properties AS (
    SELECT 1
    FROM gqlite_{{ graph_name }}_nodes, json_each(gqlite_{{ graph_name }}_nodes.properties)
    WHERE json_each.value IS NOT NULL
  ),
  edge_properties AS (
    SELECT 1
    FROM gqlite_{{ graph_name }}_edges, json_each(gqlite_{{ graph_name }}_edges.properties)
    WHERE json_each.value IS NOT NULL
  )

SELECT
  (SELECT COUNT(*) FROM gqlite_{{ graph_name }}_nodes)     AS node_count,
  (SELECT COUNT(*) FROM gqlite_{{ graph_name }}_edges)     AS edge_count,
  (SELECT COUNT(*) FROM node_labels)                       AS unique_node_label_count,
  (SELECT COUNT(*) FROM node_properties)
  + (SELECT COUNT(*) FROM edge_properties)                 AS total_property_count;