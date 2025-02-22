<div align="center">

[![stable pipeline](https://gitlab.com/gqlite/gqlite/badges/stable/pipeline.svg?key_text=stable)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=stable)
[![dev/1 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/1/pipeline.svg?key_text=dev/1)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/1)
[![dev/2 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/2/pipeline.svg?key_text=dev/2)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/2)
[![crates.io](https://img.shields.io/crates/v/gqlitecli.svg)](https://crates.io/crates/gqlitecli)
</div>

![GQLite logo](../../logo.png) GQLite (CLI)
=====================================

GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine. The `dev/2` branch contains the new implementation for `GQLite 2.x`, currently released as `gqlitedb 0.x` on (crates.io)[https://crates.io/crates/gqlitedb]. This crate is currently *experimental* and it is recommended to stick with the `GQLite 1.x` branch until further notice.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for C, Python and Ruby.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of ISO GQL.

Installation
------------

```
cargo install gqlitecli
```

Run with:

```
gqlite [filename]
```

Contributions
-------------

Contributions are very welcome. They should be submited as merge requests in [gitlab](https://gitlab.com/gqlite/GQLite/-/merge_requests). Submited code should be formated with rustfmt, using the `rustfmt.toml` in the root.
