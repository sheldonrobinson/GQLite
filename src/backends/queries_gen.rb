#!/usr/bin/env ruby

require 'tempfile'

sqlite_queries = [
  ['graph_create(const std::string& _graph_name)', 'graph_create.sql'],
  ['graph_has(const std::string& _graph_name)', 'graph_has.sql'],
  ['label_create_table()', 'label_create_table.sql'],
  ['label_get_from_id()', 'label_get_from_id.sql'],
  ['label_get_from_name()', 'label_get_from_name.sql'],
  ['label_insert()', 'label_insert.sql'],
  ['node_create(const std::string& _graph_name)', 'node_create.sql'],
  ['node_map_to_label(const std::string& _graph_name)', 'node_map_to_label.sql'],
  ['table_has()', 'table_has.sql']
]

output = File.open("sqlite_queries.h", "w")

output.write <<HEADER
//To update this file, run 'queries_gen.rb'
#include <sstream>

namespace gqlite::backends::sqlite_queries
{
HEADER

file = Tempfile.new('foo')

sqlite_queries.each() do |x|
  `ptc cppstream queries/sqlite/#{x[1]} #{file.path}`
  output.write <<FUNCTION
  std::string #{x[0]}
  {
    std::stringstream stream;
    #{File.open(file.path).read}
    return stream.str();
  }
FUNCTION
end

output.write <<FOOTER
}
FOOTER