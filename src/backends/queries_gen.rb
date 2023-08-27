#!/usr/bin/env ruby

require 'tempfile'

sqlite_queries = [
  ['create_graph(const std::string& _name)', 'create_graph.sql'],
  ['has_graph(const std::string& _name)', 'has_graph.sql'],
  ['add_label(const std::string& _name)', 'add_label.sql'],
  ['create_node(const std::string& _name)', 'create_node.sql']
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