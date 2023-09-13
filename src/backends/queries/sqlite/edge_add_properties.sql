UPDATE gqlite_/%= _graph_name %/_edges SET properties=json_set(properties, ?002, json_patch(json_extract(properties, ?002) , ?003) ) WHERE id=?001
