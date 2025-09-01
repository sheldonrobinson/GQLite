<div align="center">

[![stable pipeline](https://gitlab.com/gqlite/gqlite/badges/stable/pipeline.svg?key_text=stable)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=stable)
[![dev/1 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/1/pipeline.svg?key_text=dev/1)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/1)
[![docs](https://docs.rs/gqlitedb/badge.svg)](https://docs.rs/gqlitedb)
[![crates.io](https://img.shields.io/crates/v/gqlitedb.svg)](https://crates.io/crates/gqlitedb)
</div>

![GQLite logo](logo.png) GQLite
===============================

GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine.
GQLite support multiple database backends, such as SQLite and redb.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for C, C++, Python, Ruby and Crystal.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of ISO GQL.

Installation
------------

To build from source, GQLite requires the [cmake](https://cmake.org/) build system. For development, it might be necesserary to install [ruby](https://www.ruby-lang.org/en/) (for the test suite and updating SQL queries) and [ptc](https://www.ruby-lang.org/en/) (for updating SQL queries).

Specific installation instructions can be found in the [installation](docs/installation.md) section of the documentation.

Compatibilities
---------------

This table summarizes the versions used by GQLite. The database version refers to the schema version, with the version in parentheses indicating which version can be opened. The crate version corresponds to the GQLite release that supports that database schema.

| Database Version | GQLite Version | Crate Version |
|------------------|----------------|---------------|
| 1.3 (1.2-1.0)    | 1.3            | 0.6.x         |
| 1.2 (1.1-1.0)    | 1.2            | 0.5.x         |
| 1.2              | -              | 0.4.x         |
| -                | -              | 0.1.x-0.3.x   |
| 1.1 (1.0)        | 1.1            | -             |
| 1.0              | 1.0            | -             |

Project Guarantees
------------------

- **Forward-Compatible Database Format**  
  Databases created with older versions of GQLite can be opened by newer versions without requiring manual migration. This guarantee applies strictly to the on-disk format managed by GQLite. Compatibility is maintained across patch releases within the same minor version.

- **Query Language Stability**  
  Valid queries are expected to remain functional across minor releases. Specifically, OpenCypher queries will be supported throughout the entire 1.x series. If the introduction of GQL-specific features results in breaking changes, these will be deferred to the 2.x series or later.

- **Semantic Versioning Compliance**  
  The `gqlitedb` crate follows [Semantic Versioning](https://semver.org/), and its public API stability is checked using the [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks) tool, subject to its limitations. Note that `gqlitedb` has its own versioning and release cycle, independent of the main `GQLite` binary. 

  For maximum compatibility across `GQLite` versions, the recommended interface is the C API. The Ruby and Python APIs are also designed to remain stable across GQLite versions.


Documentation
-------------

* [GQLite official documentation](https://auksys.org/documentation/5/libraries/gqlite/)

Storage Backends
----------------

* [redb](https://redb.rs) store. This is the current default store for `gqlite 2.x`.
* [sqlite](https://sqlite.org) store. Pre-planning for a sqlite store, which the fasted and most used SQL database. This enable to achieve high performance and for application to combine Graph queries with traditional SQL queries. This is required for `gqlite 2.0`.
* [postgresql](https://postgresql.org) work-in-progress.

Contributions
-------------

Contributions are very welcome. They should be submited as merge requests in [gitlab](https://gitlab.com/gqlite/GQLite/-/merge_requests). Submited code should be formated with rustfmt, using the `rustfmt.toml` in the root.
