WITH source_delete AS NOT MATERIALIZED (/%= _what %/)
SELECT count(*) FROM gqlite_/%= _graph_name %/_edges WHERE left IN source_delete or right IN source_delete