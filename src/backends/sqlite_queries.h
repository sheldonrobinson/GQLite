//To update this file, run 'queries_gen.rb'
#include <sstream>

namespace gqlite::backends::sqlite_queries
{
  std::string edge_add_properties(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_add_properties.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_add_properties.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_add_properties.sql"
stream << "_edges SET properties=json_set(properties, ?002, json_patch(json_extract(properties, ?002) , ?003) ) WHERE id=?001\n"
"";

    return stream.str();
  }
  std::string edge_count_by_node(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_count_by_node.sql"
stream << "SELECT count(*) FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_count_by_node.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_count_by_node.sql"
stream << "_edges WHERE left=?001 or right=?001";

    return stream.str();
  }
  std::string edge_create(const std::string& _graph_name, const std::string& _values)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << "_edges (label, properties, left, right) ";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << ( _values );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << "\n"
"RETURNING id\n"
"";

    return stream.str();
  }
  std::string edge_delete(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete.sql"
stream << "DELETE FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete.sql"
stream << "_edges WHERE id=?001";

    return stream.str();
  }
  std::string edge_delete_by_node(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete_by_node.sql"
stream << "DELETE FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete_by_node.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_delete_by_node.sql"
stream << "_edges WHERE left=?001 or right=?001";

    return stream.str();
  }
  std::string edge_get_label_properties(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_get_label_properties.sql"
stream << "SELECT label, properties FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_get_label_properties.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_get_label_properties.sql"
stream << "_edges WHERE id = ?001";

    return stream.str();
  }
  std::string edge_set_property(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_set_property.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_set_property.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_set_property.sql"
stream << "_edges SET properties=json_set(properties, ?002, json(?003)) WHERE id=?001";

    return stream.str();
  }
  std::string edge_remove_property(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_remove_property.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_remove_property.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_remove_property.sql"
stream << "_edges SET properties=json_remove(properties, ?002) WHERE id=?001";

    return stream.str();
  }
  std::string function_labels(const std::string& _graph_name, const std::string& _node_id)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/function_labels.sql"
stream << "(SELECT json_group_array(labels.label)\n"
"            FROM gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/function_labels.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/function_labels.sql"
stream << "_labels as l\n"
"            JOIN gqlite_labels AS labels ON labels.id = l.label WHERE l.node_id = ";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/function_labels.sql"
stream << ( _node_id );
#line 4 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/function_labels.sql"
stream << ")\n"
"";

    return stream.str();
  }
  std::string get_debug_stats(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "SELECT COUNT(*) FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_nodes\n"
"UNION ALL SELECT COUNT(*) FROM gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_edges\n"
"UNION ALL SELECT COUNT(*) FROM gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_labels\n"
"UNION ALL SELECT COUNT(j.value) FROM gqlite_";
#line 4 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 4 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_nodes t, json_each(properties) j\n"
"UNION ALL SELECT COUNT(j.value) FROM gqlite_";
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_edges t, json_each(properties) j\n"
"UNION ALL SELECT COUNT(*) FROM gqlite_labels\n"
"UNION ALL SELECT COUNT(DISTINCT label) FROM gqlite_";
#line 7 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << ( _graph_name );
#line 7 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_labels";

    return stream.str();
  }
  std::string graph_create(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "CREATE TABLE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL);\n"
"CREATE TABLE gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges(id INTEGER PRIMARY KEY AUTOINCREMENT,\n"
"        label INTEGER,\n"
"        properties TEXT NOT NULL,\n"
"        left INTEGER,\n"
"        right INTEGER,\n"
"        FOREIGN KEY(label) REFERENCES gqlite_labels(id),\n"
"        FOREIGN KEY(left) REFERENCES gqlite_";
#line 8 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 8 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_";
#line 8 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 8 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id));\n"
"CREATE TABLE gqlite_";
#line 9 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 9 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_labels(label INTEGER, node_id INTEGER,\n"
"        FOREIGN KEY(label) REFERENCES gqlite_labels(id), FOREIGN KEY(node_id) REFERENCES gqlite_";
#line 10 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 10 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id));\n"
"\n"
"---------------- views ----------------\n"
"\n"
"-- view for querying for undirected edges\n"
"CREATE VIEW gqlite_";
#line 15 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 15 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges_undirected (id, label, properties, left, right) AS\n"
"        SELECT id, label, properties, left, right FROM gqlite_";
#line 16 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 16 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges\n"
"        UNION SELECT id, label, properties, right, left FROM gqlite_";
#line 17 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 17 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges;\n"
"\n"
"-- view for getting the nodes as a json representation\n"
"CREATE VIEW gqlite_";
#line 20 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 20 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes_as_json (id, node) AS\n"
"        SELECT id, json_object('id', id, 'properties', json(properties), \n"
"          'labels', (SELECT json_group_array(value) FROM json_each(labels) AS result WHERE value IS NOT NULL),\n"
"          'type', 'node') FROM\n"
"          (SELECT n.id AS id, n.properties AS properties, json_group_array(labels.label) AS labels\n"
"            FROM gqlite_";
#line 25 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 25 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes n\n"
"            LEFT JOIN gqlite_";
#line 26 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 26 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_labels as l ON l.node_id = n.id\n"
"            LEFT JOIN gqlite_labels AS labels ON labels.id = l.label\n"
"            GROUP BY n.id);\n"
"\n"
"CREATE VIEW gqlite_";
#line 30 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 30 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges_as_json (id, edge) AS\n"
"        SELECT e.id, json_object('id', e.id, 'properties', json(e.properties), \n"
"          'label', label.label,\n"
"          'type', 'edge') FROM\n"
"          gqlite_";
#line 34 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 37 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_edges e\n"
"          JOIN gqlite_labels label ON label.id = e.label\n"
"\n"
"";

    return stream.str();
  }
  std::string graph_has(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << "SELECT count(*) FROM sqlite_master\n"
"  WHERE type='table'\n"
"    AND (name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << ( _graph_name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << "_nodes' or name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << ( _graph_name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << "_edges' or name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << ( _graph_name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_has.sql"
stream << "_labels')";

    return stream.str();
  }
  std::string label_create_table()
  {
    std::stringstream stream;
    #line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/label_create_table.sql"
stream << "CREATE TABLE gqlite_labels(id INTEGER PRIMARY KEY AUTOINCREMENT, label TEXT NOT NULL UNIQUE);\n"
"INSERT INTO gqlite_labels(id, label) VALUES (0, \"\")";

    return stream.str();
  }
  std::string label_get_from_id()
  {
    std::stringstream stream;
    #line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/label_get_from_id.sql"
stream << "SELECT label FROM gqlite_labels WHERE id = ?001\n"
"";

    return stream.str();
  }
  std::string label_get_from_name()
  {
    std::stringstream stream;
    #line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/label_get_from_name.sql"
stream << "SELECT id FROM gqlite_labels WHERE label = ?001\n"
"";

    return stream.str();
  }
  std::string label_insert()
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/label_insert.sql"
stream << "INSERT INTO gqlite_labels(label) VALUES (?001)";

    return stream.str();
  }
  std::string node_add_label(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_label.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_label.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_label.sql"
stream << "_labels(label, node_id)  SELECT label.value, node.value FROM json_each(?001) label  JOIN json_each(?002) node";

    return stream.str();
  }
  std::string node_add_properties(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_properties.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_properties.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_add_properties.sql"
stream << "_nodes SET properties=json_set(properties, ?002, json_patch(json_extract(properties, ?002) , ?003) ) WHERE id=?001\n"
"";

    return stream.str();
  }
  std::string node_create(const std::string& _graph_name, const std::string& _values)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << "_nodes (properties) ";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << ( _values );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << "\n"
"RETURNING id";

    return stream.str();
  }
  std::string node_delete(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_delete.sql"
stream << "DELETE FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_delete.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_delete.sql"
stream << "_labels WHERE node_id=?001;\n"
"DELETE FROM gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_delete.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_delete.sql"
stream << "_nodes WHERE id=?001";

    return stream.str();
  }
  std::string node_get_labels(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_labels.sql"
stream << "SELECT label FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_labels.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_labels.sql"
stream << "_labels WHERE node_id = ?001";

    return stream.str();
  }
  std::string node_get_properties(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_properties.sql"
stream << "SELECT properties FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_properties.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_get_properties.sql"
stream << "_nodes WHERE id = ?001";

    return stream.str();
  }
  std::string node_remove_label(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_label.sql"
stream << "DELETE FROM gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_label.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_label.sql"
stream << "_labels WHERE label = ?001 AND node_id = ?002";

    return stream.str();
  }
  std::string node_remove_property(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_property.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_property.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_remove_property.sql"
stream << "_nodes SET properties=json_remove(properties, ?002) WHERE id=?001";

    return stream.str();
  }
  std::string node_set_property(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_set_property.sql"
stream << "UPDATE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_set_property.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_set_property.sql"
stream << "_nodes SET properties=json_set(properties, ?002, json(?003)) WHERE id=?001";

    return stream.str();
  }
  std::string node_select_many(const std::string& _graph_name, int _idx)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_select_many.sql"
stream << "SELECT nas.node as node_map, ROW_NUMBER() over () as node_rowid\n"
"  FROM gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_select_many.sql"
stream << ( _graph_name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_select_many.sql"
stream << "_nodes_as_json nas\n"
"  JOIN json_each(?";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_select_many.sql"
stream << ( to_string_fixed_width(_idx, 3) );
#line 4 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_select_many.sql"
stream << ") s ON s.value = nas.id\n"
"";

    return stream.str();
  }
  std::string table_has()
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/table_has.sql"
stream << "SELECT count(*) FROM sqlite_master WHERE type='table' AND (name=?001)";

    return stream.str();
  }
}
