//To update this file, run 'queries_gen.rb'
#include <sstream>

namespace gqlite::backends::sqlite_queries
{
  std::string edge_create(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/edge_create.sql"
stream << "_edges (label, properties, left, right) VALUES (?001, ?002, ?003, ?004)";

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
#line 7 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/get_debug_stats.sql"
stream << "_edges t, json_each(properties) j\n"
"UNION ALL SELECT COUNT(*) FROM gqlite_labels\n"
"";

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
"       label INTEGER,\n"
"       properties TEXT NOT NULL,\n"
"       left INTEGER,\n"
"       right INTEGER,\n"
"       FOREIGN KEY(label) REFERENCES gqlite_labels(id),\n"
"       FOREIGN KEY(left) REFERENCES gqlite_";
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
stream << "_nodes(id))";

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
  std::string node_create(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_create.sql"
stream << "_nodes (properties) VALUES (?001)";

    return stream.str();
  }
  std::string node_map_to_label(const std::string& _graph_name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_map_to_label.sql"
stream << "INSERT INTO gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_map_to_label.sql"
stream << ( _graph_name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/node_map_to_label.sql"
stream << "_labels   (label, node_id) VALUES (?001, ?002)";

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
  std::string table_has()
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/table_has.sql"
stream << "SELECT count(*) FROM sqlite_master WHERE type='table' AND (name=?001)";

    return stream.str();
  }
}
