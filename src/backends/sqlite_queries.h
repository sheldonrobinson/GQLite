//To update this file, run 'queries_gen.rb'
#include <sstream>

namespace gqlite::backends::sqlite_queries
{
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
stream << "_edges(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL,\n"
"       left INTEGER,\n"
"       right INTEGER,\n"
"       FOREIGN KEY(left) REFERENCES gqlite_";
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id), FOREIGN KEY(right) REFERENCES gqlite_";
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 5 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_nodes(id));\n"
"CREATE TABLE gqlite_";
#line 6 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 6 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << "_labels(label INTEGER, node_id INTEGER,\n"
"        FOREIGN KEY(label) REFERENCES gqlite_labels(id), FOREIGN KEY(node_id) REFERENCES gqlite_";
#line 7 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
stream << ( _graph_name );
#line 7 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/graph_create.sql"
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
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/label_create_table.sql"
stream << "CREATE TABLE gqlite_labels(id INTEGER PRIMARY KEY AUTOINCREMENT, label TEXT NOT NULL UNIQUE)";

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
  std::string table_has()
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/table_has.sql"
stream << "SELECT count(*) FROM sqlite_master WHERE type='table' AND (name=?001)";

    return stream.str();
  }
}
