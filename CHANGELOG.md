# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

- Add possibility to create Rune connection from arc connection.

### gqb

- Add support for set statements.
- Add graph management, use, create and drop graph.

## [0.6.1] - 2025-10-28

- Add `id` function.
- Fix aliasing in return/with statements.
- Improve query status reporting in gqlbrowser.

### gqb

- Add `where` statement and expressions.
- Add expressions to `return` statement.
- Add `delete` to query builder (added in gqb 0.2.1).

## [0.6.0] - 2025-09-01

- Add `gqb` a crate for generating queries using a high-level API.
- Upgrade to redb 3.
- API cleanup.

## [0.5.3] - 2025-08-17
- Add egui-based, GQLite browser.
- Add support for WASM (only redb backend supported)
- Add support for in-memory databases.
- Add `exponent` (^) operator.
- Add Rune bindings.

## [0.5.2] - 2025-07-26
- New Ruby/Python bindings.
- Add `head` function, and `avg` aggregator.
- Add support for executing multiple independent queries separated by a ';'.
- Add execution of multiple independent queries separated by a ';'.
- Add reading file as input in the CLI.
- Add support for using connection in multi-threaded environment.

## [0.5.1] - 2025-07-15
- Add experimental connection server, for use in multi-threaded environment.
- Export Node/Edge/Path.

## [0.5.0] - 2025-07-13
- Support for USING and multiple graphs.
- Fix aggregation, with grouping.
- Add max/min aggregations.
- Add upgrade from 1.0/1.1.

## [0.4.0] - 2025-07-09
- SQLite backend.
- Refactored handling of variables in the executor.
- Improved Rust API.

## [0.3.4] - 2025-04-18
- Improve support of comparison and maps.
- Add support for string as key in map, and allow to use operator [] for map.
- Add properties, toString functions.

## [0.3.3] - 2025-03-31

- Improve support for lists, float, strings and for graph patterns.
- Fix support for boolean, conditional, null, integer literals.
- Add ceil, floor, rand, toInteger functions.
- Add collect aggregator.

## [0.3.2] - 2025-02-22

- Fix order of arrays.
- Support for array concatenation.
- Improve support of UNWIND.
- Add support for ORDER BY, LIMIT and SKIP

## [0.3.1] - 2025-02-16

- Add support for += in SET.
- Add support for REMOVE.

## [0.3.0] - 2025-02-09

- Change in backend serialization.
- Support for SET to add properties and labels.

## [0.2.0] - 2025-02-08

- `persy` backend is removed.
- `redb` backend.

## [0.1.1] - 2025-02-06

- Support for aggregation (count).
- Improved support for WHERE and WITH.
- Most binary/unary expressions are supported.

## [0.1.0] - 2025-02-01

- Basic support Create/Match.
- `persy` backend.

<!-- next-url -->
[Unreleased]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.6.1...dev/1
[0.6.1]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.6.0...gqlitedb-v0.6.1
[0.6.0]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.5.3...gqlitedb-v0.6.0
[0.5.3]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.5.2...gqlitedb-v0.5.3
[0.5.2]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.5.1...gqlitedb-v0.5.2
[0.5.1]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.5.0...gqlitedb-v0.5.1
[0.5.0]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.4.0...gqlitedb-v0.5.0
[0.4.0]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.3.4...gqlitedb-v0.4.0
[0.3.4]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.3.3...gqlitedb-v0.3.4
[0.3.3]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.3.2...gqlitedb-v0.3.3
[0.3.2]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.3.1...gqlitedb-v0.3.2
[0.3.1]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.3.0...gqlitedb-v0.3.1
[0.3.0]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.2.0...gqlitedb-v0.3.0
[0.2.0]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.1.1...gqlitedb-v0.2.0
[0.1.1]: https://gitlab.com/gqlite/gqlite/compare/gqlitedb-v0.1.0...gqlitedb-v0.1.1
[0.1.0]: https://gitlab.com/gqlite/gqlite/compare/17d12...gqlitedb-v0.1.0
