GQLite
======

GQLite aims to be the equivalent to SQLite for graph database: an easy to embedd query engine, with the data stored in a single file. To achieve that, GQLite uses SQLite to store data, and come as a library implementing a translation between graph query languages and SQL. The library is written in C++, but also provides a C API 

The official repositories contains bindings/APIs for C, C++, Python and Ruby.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of [OpenCypher](https://opencypher.org/), and the intent is to also support ISO GQL in the future when it become available.

Documentation
-------------

* [GQLite official documentation](docs/main.md)
* [OpenCypher](https://opencypher.org/) is the main query language for GQLite

License
-------

The GQLite library is released under the [MIT License](LICENSE).

Contributions
-------------

Contributions are very welcome. They should be submited as merge requests in [gitlab](https://gitlab.com/cyloncore/GQLite/-/merge_requests). The code based does not follow strict coding style rules, but the indentation shoulbe two spaces, and the code should be aerated and readable.
