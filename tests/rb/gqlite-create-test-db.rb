#!/usr/bin/env ruby

#
# This script is used to create a test database to use for testing version to version stability.
#

if ARGV.size != 1
  puts "gqlite-create-test-db [version]"
  exit -1
end

require 'gqlite'

file = "data/test-#{ARGV[0]}.db"
connection = GQLite::Connection.new(sqlite_filename: file)

connection.execute_oc_query <<-QUERY

CREATE (a:A:E {name: 'A', index: 0}), (b:B {name: 'B', index: 1}), (c:C:F {name: 'C', index: 2}), (d:D {name: 'D', index: 3}),
       (a)-[:AB { name: 'AB', sum: 1 }]->(b),
       (b)-[:BC { name: 'BC', sum: 3 }]->(c),
       (c)-[:CD { name: 'CD', sum: 5 }]->(d),
       (d)-[:DA { name: 'DA', sum: 3 }]->(a)
QUERY

`xz -z #{file}; chmod a-w #{file}.xz`
