#!/usr/bin/env ruby

require 'tempfile'

sqlite_queries = [
  ['edge_add_properties(const std::string& _graph_name, const std::string& _what, const std::string& _expr, const std::string& _path, const std::string& _edge_id)', 'edge_add_properties.sql'],
  ['edge_count_by_node(const std::string& _graph_name)', 'edge_count_by_node.sql'],
  ['edge_create(const std::string& _graph_name, const std::string& _values)', 'edge_create.sql'],
  ['edge_delete(const std::string& _graph_name, const std::string& _what)', 'edge_delete.sql'],
  ['edge_delete_by_node(const std::string& _graph_name, const std::string& _what)', 'edge_delete_by_node.sql'],
  ['edge_get_label_properties(const std::string& _graph_name)', 'edge_get_label_properties.sql'],
  ['edge_set_property(const std::string& _graph_name, const std::string& _what, const std::string& _expr, const std::string& _path, const std::string& _edge_id)', 'edge_set_property.sql'],
  ['edge_remove_property(const std::string& _graph_name, const std::string& _what, const std::string& _path, const std::string& _edge_id)', 'edge_remove_property.sql'],
  ['function_labels(const std::string& _graph_name, const std::string& _node_id)', 'function_labels.sql'],
  ['get_debug_stats(const std::string& _graph_name)', 'get_debug_stats.sql'],
  ['graph_create(const std::string& _graph_name)', 'graph_create.sql'],
  ['graph_has(const std::string& _graph_name)', 'graph_has.sql'],
  ['label_create_table()', 'label_create_table.sql'],
  ['label_get_from_id()', 'label_get_from_id.sql'],
  ['label_get_from_name()', 'label_get_from_name.sql'],
  ['label_insert()', 'label_insert.sql'],
  ['node_add_label(const std::string& _graph_name)', 'node_add_label.sql'],
  ['node_add_labels(const std::string& _graph_name, const std::string& _what)', 'node_add_labels.sql'],
  ['node_add_properties(const std::string& _graph_name, const std::string& _what, const std::string& _expr, const std::string& _path, const std::string& _node_id)', 'node_add_properties.sql'],
  ['node_create(const std::string& _graph_name, const std::string& _values)', 'node_create.sql'],
  ['node_delete(const std::string& _graph_name, const std::string& _what)', 'node_delete.sql'],
  ['node_get_labels(const std::string& _graph_name)', 'node_get_labels.sql'],
  ['node_get_properties(const std::string& _graph_name)', 'node_get_properties.sql'],
  ['node_remove_label(const std::string& _graph_name, const std::string& _what, const std::string& _labels)', 'node_remove_label.sql'],
  ['node_remove_property(const std::string& _graph_name, const std::string& _what, const std::string& _path, const std::string& _node_id)', 'node_remove_property.sql'],
  ['node_set_property(const std::string& _graph_name, const std::string& _what, const std::string& _expr, const std::string& _path, const std::string& _node_id)', 'node_set_property.sql'],
  ['node_select_many(const std::string& _graph_name, int _idx)', 'node_select_many.sql'],
  ['table_has()', 'table_has.sql'],
  ['uid_create_table()', 'uid_create_table.sql'],
  ['uid_next()', 'uid_next.sql']
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
  inline std::string #{x[0]}
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