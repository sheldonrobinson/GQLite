#!/usr/bin/env ruby

require 'gqlite'

if ARGV.length != 2
  puts 'gqlite_rb_example [database] [query]'
  exit -1
end

begin
  # Create a database on the file given in argv[0]
  connection = GQLite::Connection.new filename: ARGV[0]

  # Execute the query from argv[1]
  value = connection.execute_oc_query ARGV[1]

  # Print the result
  if value.nil?
    puts "Empty results"
  else
    puts "Results are #{value.to_s}"
  end
rescue GQLite::Error => ex
  # Report any error
  puts "An error has occured: #{ex.message}"
end
