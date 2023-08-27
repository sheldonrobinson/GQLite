//To update this file, run 'queries_gen.rb'
#include <sstream>

namespace gqlite::backends::sqlite_queries
{
  std::string create_graph(const std::string& _name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << "CREATE TABLE gqlite_";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << ( _name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << "_nodes(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL;\n"
"CREATE TABLE gqlite_";
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << ( _name );
#line 2 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << "_edges(id INTEGER PRIMARY KEY AUTOINCREMENT, properties TEXT NOT NULL, left INTEGER, right INTEGER, FOREIGN KEY(left) REFERENCES \" + _name + \"_nodes(id), FOREIGN KEY(right) REFERENCES \" + _name + \"_nodes(id))\");\n"
"CREATE TABLE gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << ( _name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_graph.sql"
stream << "_labels(label TEXT NOT NULL, node_id INTEGER)";

    return stream.str();
  }
  std::string has_graph(const std::string& _name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << "SELECT count(*) FROM sqlite_master\n"
"  WHERE type='table'\n"
"    AND (name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << ( _name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << "_nodes' or name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << ( _name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << "_edges' or name='gqlite_";
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << ( _name );
#line 3 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/has_graph.sql"
stream << "_labels')";

    return stream.str();
  }
  std::string add_label(const std::string& _name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/add_label.sql"
stream << "INSERT INTO ";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/add_label.sql"
stream << ( _name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/add_label.sql"
stream << "_nodes (label, node_id) VALUES (?000, ?001)";

    return stream.str();
  }
  std::string create_node(const std::string& _name)
  {
    std::stringstream stream;
    #line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_node.sql"
stream << "INSERT INTO ";
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_node.sql"
stream << ( _name );
#line 1 "/home/cyrille/lrs-pkg/src/gqlite/src/backends/queries/sqlite/create_node.sql"
stream << "_nodes (properties) VALUES (?000)";

    return stream.str();
  }
}
