![GQLite logo](../../logo.png) GQB (Graph Query Builder)
========================================================

`GQB` is part of the [GQLite project](https://gqlite.org) project, it an API for building OpenCypher query with a high-level interface. It supports for creating graphs and querying existing graphs.

Usage
-----

To use `GQB` in a project, add it to your `Cargo.toml` using:

```bash
cargo add gqb
```

Create Graph
------------

```rust
use gqb::{Builder, labels, value_map, ValueMap, prelude::*};

let mut builder = Builder::default();

// Create a single node with label "a"

let n1 = builder.create_node(labels!("a"), ValueMap::default());

// Create two nodes with respective label "b" and "c". The first node has
// a propery "name" with value "b label".
let (n2, n3) = builder.create_nodes((
    (labels!("b"), value_map!{ "name" => "b label" }),
    (labels!("c"), ValueMap::default()),
));

// Create an edge between `n1` and `n2`.
let _ = builder.create_edge(n1, labels!("d"), ValueMap::default(), n2);
// Create an edge between `n1` and `n3`, `n2` and `n3`, the second edge has
// a property "key" with value "b -> c".
let (_, _) = builder.create_edges((
    (n1, labels!("e"), ValueMap::default(), n3),
    (n2, labels!("f"), value_map!{ "key" => "b -> c" }, n3),
));

// Generate OpenCypher query and bindings.
let (query, bindings) = builder.into_oc_query().unwrap();

// Execute the query in a GQLite database.
let connection = gqlitedb::Connection::builder().create().unwrap();
let _ = connection.execute_oc_query(query, bindings).unwrap();
```

Match a Graph
-------------


```rust
use gqb::{Builder, labels, value_map, ValueMap, prelude::*};

let mut builder = Builder::default();

// Match for a node with label "a" and a property "id" with value `3`.
// The node can have other properties.
let n1 = builder.match_node(labels!("a"), value_map!("id" => 3));

// Create an anonymous variable for a node, aka, it will match any node.
let n2 = builder.node_variable();

// Match edges between any node that matches n1 and n2, and has label "b"
// and a property "id" with value `2`. The edge can have other properties.
let e1 = builder.match_edge(n1, labels!("b"), value_map!("id" => 2), n2);

// Return in the result, variable n1, e1 and n2.
builder.return_variable(n1, "n1");
builder.return_variable(e1, "e1");
builder.return_variable(n2, "n2");

// Generate OpenCypher query and bindings.
let (query, bindings) = builder.into_oc_query().unwrap();

// Execute the query in a GQLite database.
let connection = gqlitedb::Connection::builder().create().unwrap();
let _ = connection.execute_oc_query(query, bindings).unwrap();
```
