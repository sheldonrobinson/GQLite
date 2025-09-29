SELECT
  e.edge_key     AS edge_key,
  e.labels       AS edge_labels,
  e.properties   AS edge_properties,
  {% if is_undirected %}e.reversed{% else %}0{% endif %} AS edge_reversed,

  n_left.node_key   AS left_node_key,
  n_left.labels     AS left_node_labels,
  n_left.properties AS left_node_properties,

  n_right.node_key      AS right_node_key,
  n_right.labels        AS right_node_labels,
  n_right.properties    AS right_node_properties

FROM gqlite_{{ graph_name }}_edges{{ table_suffix }} AS e
JOIN gqlite_{{ graph_name }}_nodes AS n_left  ON e.left  = n_left.id
JOIN gqlite_{{ graph_name }}_nodes AS n_right ON e.right = n_right.id
WHERE
    -- Filter by key list (if not empty)
    {% if let Some(edge_keys_var) = edge_keys_var %}
    (
        hex(e.edge_key) IN (
            SELECT value FROM json_each(?{{ edge_keys_var }})
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if let Some(edge_labels_var) = edge_labels_var %}
    -- Filter by required labels (must all be in e.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ edge_labels_var }}) AS required_label
            WHERE NOT EXISTS (
                SELECT 1
                FROM json_each(e.labels) AS edge_label
                WHERE edge_label.value = required_label.value
            )
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if let Some(edge_properties_var) = edge_properties_var %}
    -- Filter by required properties (must all exist and match)
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ edge_properties_var }}) AS required_prop
            WHERE json_extract(e.properties, '$.' || required_prop.key) IS NULL
                OR json_extract(e.properties, '$.' || required_prop.key) != required_prop.value
        )
    {% else %}
        1
    {% endif %}
    -- Filter by key list (if not empty)
    AND
    {% if let Some(left_keys_var) = left_keys_var %}
    (
        hex(n_left.node_key) IN (
              SELECT value FROM json_each(?{{ left_keys_var }})
        )
    )
    {% else %}
        1
    {% endif %}
    AND
    {% if let Some(left_labels_var) = left_labels_var %}
    -- Filter by required labels (must all be in n_left.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ left_labels_var }}) AS required_label
            WHERE NOT EXISTS (
                SELECT 1
                FROM json_each(n_left.labels) AS node_label
                WHERE node_label.value = required_label.value
            )
        )
    )
    {% else %}
    1
    {% endif %}
    AND 
    {% if let Some(left_properties_var) = left_properties_var %}
    -- Filter by required properties (must all exist and match)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ left_properties_var }}) AS required_prop
            WHERE json_extract(n_left.properties, '$.' || required_prop.key) IS NULL
               OR json_extract(n_left.properties, '$.' || required_prop.key) != required_prop.value
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    -- Filter by key list (if not empty)
    {% if let Some(right_keys_var) = right_keys_var %}
    (
        hex(n_right.node_key) IN (
            SELECT value FROM json_each(?{{ right_keys_var }})
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if let Some(right_labels_var) = right_labels_var %}
    -- Filter by required labels (must all be in n_right.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ right_labels_var }}) AS required_label
            WHERE NOT EXISTS (
                SELECT 1
                FROM json_each(n_right.labels) AS node_label
                WHERE node_label.value = required_label.value
            )
        )
    )
    {% else %}
    1
    {% endif %}

    {% if let Some(right_properties_var) = right_properties_var %}
    -- Filter by required properties (must all exist and match)
    AND (
        NOT EXISTS (
            SELECT 1
            FROM json_each(?{{ right_properties_var }}) AS required_prop
            WHERE json_extract(n_right.properties, '$.' || required_prop.key) IS NULL
               OR json_extract(n_right.properties, '$.' || required_prop.key) != required_prop.value
        )
    );
    {% endif %}