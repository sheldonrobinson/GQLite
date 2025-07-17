GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine.
GQLite support multiple database backends, such as SQLite and redb.
This enable to achieve high performance and for application to combine Graph queries with traditional SQL queries.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for Rust, C, C++, Python, Ruby and Crystal.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of OpenCypher, with some ISO GQL extensions.

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

The documentation for the GQL query language can found in [GQL](https://gitlab.com/gqlite/GQLite/-/blob/docs/gql.md) and for the [API](https://gitlab.com/gqlite/GQLite/-/blob/docs/api.md).