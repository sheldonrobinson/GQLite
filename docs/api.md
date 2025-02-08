API
===


C
-

The C API is the recommended API for binding gqlite with other languages. The main header is [gqlite-c.h](../include/gqlite-c.h), it contains API documentation. An example of us is [main.c](../examples/main.c).

C++
---

An other possibility is to use the public C++ API. The main header is [gqlite.h](../include/gqlite.h), it contains API documentation. An example of use is [main.cpp](../examples/main.cpp).

Python
------

The Python API is imported with `import gqlite` and it contains two classes:

* `gqlite.Error` an exception reported when an error occurs. The error message is in a field called `msg`.
* `gqlite.Connection` is the main class for connection to the database.
  * `gqlite.Connection(filename)` is used to create a new connection to a gqlite database stored in a redb database
  * `execute_oc_query(query, parameters)` is used to query the database, `parameters` is an optional map.
* `gqlite.connection(filename)` can be used to create a connection to a GQLite database.

An example of use is [main.py](../examples/main.py).

Ruby
----

The Ruby API is imported with `require 'gqlite'` and it contains two classes:

* `gqlite::Error` an exception reported when an error occurs. The error message is in a field called `message`.
* `gqlite::Connection` is the main class for connection to the database.
  * `gqlite::Connection.new(filename)` is used to create a new connection to a gqlite database stored in a redb database
  * `execute_oc_query(query, parameters)` is used to query the database, `parameters` is an optional map.

An example of use is [main.rb](../examples/main.rb).
