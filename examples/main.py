#!/usr/bin/env python3

import gqlite
import sys

if len(sys.argv) != 3:
  print('gqlite_rb_example [database] [query]')
  sys.exit(-1)

try:
  # Create a database on the file given in argv[1]
  connection = gqlite.Connection(sys.argv[1])

  # Execute the query from argv[2]
  value = connection.execute_oc_query(sys.argv[2])

  # Print the result
  print(f"Results are {value}")
except gqlite.Error as ex:
  # Report any error
  print(f"An error has occured: #{ex.msg}")
