#!/usr/bin/env ruby

source_dir = ARGV[0]

# Order of headers matters
src_files = [
  "format.h",
  "logging.h",
  "string.h",
  "gqlite_p.h",
  "global.h",
  # errors.h depends on expression_info
  "oc/algebra/expression_info.h",
  "errors.h",
  "oc/token.h",
  "oc/lexer.h",
  "oc/algebra/node_type.h",
  "oc/algebra/node.h",
  "oc/algebra/node_p.h",
  "oc/algebra/nodes.h",
  # node visitors headers depends on nodes.h
  "oc/algebra/abstract_node_visitor.h",
  "oc/algebra/default_node_visitor.h",
  "oc/algebra/visitors/expression_analyser.h",
  "oc/algebra/visitors/print.h",
  "oc/algebra/visitors/stringify.h",
  "oc/parser.h",
  "backend.h",
  "backends/sqlite_queries.h",
  "backends/sqlite.h",
  
  
  "backend.cpp", "connection.cpp", "exception.cpp", "gqlite-c.cpp", "value.cpp",
  "oc/lexer.cpp", "oc/parser.cpp", "oc/token.cpp",
  "oc/algebra/abstract_node_visitor.cpp", "oc/algebra/node_type.cpp", "oc/algebra/node.cpp", "oc/algebra/nodes.cpp",
  "oc/algebra/visitors/print.cpp",
  "backends/sqlite.cpp", "backends/sqlite_ext.cpp"
 ]

token_keywords = File.open(source_dir + "/src/oc/token_keywords.h").read
nodes_defs     = File.open(source_dir + "/src/oc/algebra/nodes_defs.h").read


ofile = File.open("gqlite-amalgamate.cpp", "w")

ofile.write <<-HEADERS
#include <gqlite.h>
#include <gqlite-c.h>
HEADERS

src_files.each() do |filename|
  ifile = File.open(source_dir + "/src/" + filename).read
  if filename == "oc/algebra/nodes.cpp"
    ifile.sub! 'using namespace gqlite::oc::algebra;', 'namespace gqlite::oc::algebra {'
    ifile += "}\n"
  end
  ifile.gsub! /\#include \"token_keywords.h\"$/, token_keywords
  ifile.gsub! /\#include \"nodes_defs.h\"$/, nodes_defs
  ifile.gsub! /\#include \".*\"$/, ''
  ifile.gsub! /\#line .*$/, ''
  ifile.gsub! /\#pragma once$/, ''
  ofile.write "//BEGIN #{filename}\n"
  ofile.write ifile
  ofile.write "//END #{filename}\n"
end
