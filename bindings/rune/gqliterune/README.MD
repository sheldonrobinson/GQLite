<div align="center">

[![stable pipeline](https://gitlab.com/gqlite/gqlite/badges/stable/pipeline.svg?key_text=stable)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=stable)
[![dev/1 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/1/pipeline.svg?key_text=dev/1)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/1)
[![crates.io](https://img.shields.io/crates/v/gqliterune.svg)](https://crates.io/crates/gqliterune)
</div>

![GQLite logo](../../logo.png) GQLite Rune Bindings
===================================================

GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine.
This crate provides bindings to execute queries in a GQLite database, using the [Rune programming language](https://rune-rs.github.io/).

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

Installation
------------

In an existing project, configured to run `Rune` programs:

```
cargo add gqliterune
```

Example of Rune programming that executes OpenCypher queries:

```Rune
let connection = gqlite::Connection::create({}).unwrap();
connection.execute_oc_query("CREATE (n)", {}).unwrap();
let r = connection.execute_oc_query("MATCH (n) RETURN n", {});
println!("Results: {r}");
```

Contributions
-------------

Contributions are very welcome. They should be submitted as merge requests in [gitlab](https://gitlab.com/gqlite/GQLite/-/merge_requests). Submitted code should be formatted with rustfmt, using the `rustfmt.toml` in the root.
