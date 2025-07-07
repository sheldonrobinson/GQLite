SELECT node_key, labels, properties
FROM gqlite_{{ graph_name }}_nodes AS nodes
WHERE
    -- Filter by key list (if not empty)
    {% if has_keys %}
    (
        hex(nodes.node_key) IN (
            SELECT value FROM json_each(:keys)
        )
    )
    {% else %}
    1
    {% endif %}
    AND
    {% if has_labels %}
    -- Filter by required labels (must all be in nodes.labels)
    (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:labels) AS required_label
            WHERE NOT EXISTS (
                SELECT 1
                FROM json_each(nodes.labels) AS node_label
                WHERE node_label.value = required_label.value
            )
        )
    )
    {% else %}
    1
    {% endif %}

    {% if has_properties %}
    -- Filter by required properties (must all exist and match)
    AND (
        NOT EXISTS (
            SELECT 1
            FROM json_each(:properties) AS required_prop
            WHERE json_extract(nodes.properties, '$.' || required_prop.key) IS NULL
               OR json_extract(nodes.properties, '$.' || required_prop.key) != required_prop.value
        )
    );
    {% endif %}
