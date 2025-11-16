<div align="center">

[![stable pipeline](https://gitlab.com/gqlite/gqlite/badges/stable/pipeline.svg?key_text=stable)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=stable)
[![dev/1 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/1/pipeline.svg?key_text=dev/1)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/1)
[![crates.io](https://img.shields.io/crates/v/gqlitebrowser.svg)](https://crates.io/crates/gqlitebrowser)
</div>

![GQLite logo](https://gqlite.org/assets/images/logo-88x88.png) GQLite Browser
==============================================================================

GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine.
This crate provides a graphical user interface (GUI) to execute queries in a GQLite database.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

Installation
------------

```
cargo install gqlitebrowser
```

Run with:

```
gqlitebrowser [filename]
```

Contributions
-------------

Contributions are very welcome. They should be submitted as merge requests in [gitlab](https://gitlab.com/auksys/GQLite/-/merge_requests). Submitted code should be formatted with rustfmt, using the `rustfmt.toml` in the root.
