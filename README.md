<div align="center">

[![stable pipeline](https://gitlab.com/gqlite/gqlite/badges/stable/pipeline.svg?key_text=stable)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=stable)
[![dev/1 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/1/pipeline.svg?key_text=dev/1)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/1)
[![dev/2 pipeline](https://gitlab.com/gqlite/gqlite/badges/dev/2/pipeline.svg?key_text=dev/2)](https://gitlab.com/gqlite/gqlite/-/pipelines?ref=dev/2)
</div>

![GQLite logo](logo.png) GQLite
===============================

GQLite is a C++-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine. The data is stored in a [persy](https://persy.rs) store, which the fasted and most used SQL database. This enable to achieve high performance and for application to combine Graph queries with traditional SQL queries.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for C, C++, Python and Ruby.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of ISO GQL.

Installation
------------

To build from source, GQLite requires the [cmake](https://cmake.org/) build system. For development, it might be necesserary to install [ruby](https://www.ruby-lang.org/en/) (for the test suite and updating SQL queries) and [ptc](https://www.ruby-lang.org/en/) (for updating SQL queries).

Specific installation instructions can be found in the [installation](docs/installation.md) section of the documentation.

Documentation
-------------

* [GQLite official documentation](docs/main.md)

Contributions
-------------

Contributions are very welcome. They should be submited as merge requests in [gitlab](https://gitlab.com/gqlite/GQLite/-/merge_requests). The code based does not follow strict coding style rules, but the indentation shoulbe two spaces, and the code should be aerated and readable.
