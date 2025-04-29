WITH source_delete AS NOT MATERIALIZED (SELECT id WHERE key in {{ keys }})
DELETE FROM gqlite_{{ graph_name }}_edges WHERE left IN source_delete or right IN source_delete