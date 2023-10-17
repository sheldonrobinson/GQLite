GQLite is a C++-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine. The data is stored in a SQLite database, which the fasted and most used SQL database. This enable to achieve high performance and for application to combine Graph queries with traditional SQL queries.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for C, C++, Python, Ruby and Crystal.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of [OpenCypher](https://opencypher.org/), and the intent is to also support ISO GQL in the future when it become available.

Example of use
--------------

```python
import gqlite

try:
  # Create a database on the file "test.db"
  connection = gqlite.connect("test.db")

  # Execute a simple query to create a node and return all the nodes
  value = connection.execute_oc_query("CREATE () MATCH (n) RETURN n")

  # Print the result
  print(f"Results are {value}")
except gqlite.Error as ex:
  # Report any error
  print(f"An error has occured: #{ex.msg}")
```

The documentation for the openCypher query language can found in [openCypher](https://gitlab.com/gqlite/GQLite/-/blob/docs/opencypher.md) and for the [API](https://gitlab.com/gqlite/GQLite/-/blob/docs/api.md).
