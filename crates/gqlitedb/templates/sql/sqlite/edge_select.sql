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
    {% if has_edge_keys %}
    (
        hex(e.edge_key) IN (
            SELECT value FROM json_each(:edge_keys)
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if has_edge_labels %}
    -- Filter by required labels (must all be in e.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:edge_labels) AS required_label
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
    {% if has_edge_properties %}
    -- Filter by required properties (must all exist and match)
        NOT EXISTS (
            SELECT 1
            FROM json_each(:edge_properties) AS required_prop
            WHERE json_extract(e.properties, '$.' || required_prop.key) IS NULL
                OR json_extract(e.properties, '$.' || required_prop.key) != required_prop.value
        )
    {% else %}
        1
    {% endif %}
    -- Filter by key list (if not empty)
    AND
    {% if has_n_left_keys %}
    (
        hex(n_left.node_key) IN (
              SELECT value FROM json_each(:n_left_keys)
        )
    )
    {% else %}
        1
    {% endif %}
    AND
    {% if has_n_left_labels %}
    -- Filter by required labels (must all be in n_left.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:n_left_labels) AS required_label
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
    {% if has_n_left_properties %}
    -- Filter by required properties (must all exist and match)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:n_left_properties) AS required_prop
            WHERE json_extract(n_left.properties, '$.' || required_prop.key) IS NULL
               OR json_extract(n_left.properties, '$.' || required_prop.key) != required_prop.value
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    -- Filter by key list (if not empty)
    {% if has_n_right_keys %}
    (
        hex(n_right.node_key) IN (
            SELECT value FROM json_each(:n_right_keys)
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if has_n_right_labels %}
    -- Filter by required labels (must all be in n_right.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:n_right_labels) AS required_label
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

    {% if has_n_right_properties %}
    -- Filter by required properties (must all exist and match)
    AND (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:n_right_properties) AS required_prop
            WHERE json_extract(n_right.properties, '$.' || required_prop.key) IS NULL
               OR json_extract(n_right.properties, '$.' || required_prop.key) != required_prop.value
        )
    );
    {% endif %}