GQL Query Language
==================

GQL is an ISO standard for a graph query language. This document covers the implementation of GQL in GQLite.

*Note:* the GQL specification is hidden behind a paywall, and is not available to the authors of GQLite. The implementation of GQL is based on secondary sources.


Difference with ISO GQL
-----------------------

GQLite currently only support a subset of GQL, and has the following difference:

* integer range is limited to 64 bits, aka, -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807, or from −(2^63) to 2^63 − 1.

Supported features
------------------

List of supported features:

* *CREATE*
* *MATCH*
* *RETURN*
* *REMOVE*
* *DELETE*
* *SET*
* *WITH*
* *ORDER BY*
* *LIMIT*
* *SKIP*
* *OPTIONAL*

The list of features under development or planned for future release can be found in [gitlab issues](https://gitlab.com/gqlite/GQLite/-/issues/?label_name%5B%5D=gql%3Afeature)

List of features with low likelyhood of implementation (external contributions are welcome):

* *recursive path*: due to the complexity of executing such queries.

Create Query
------------

Match Query
-----------

Optional Query
--------------

The support for `OPTIONAL` query is partial and does not fully conform to the specification. As such, results might change in future version.

With Statement
--------------

Delete Query
------------

Remove Query
------------

Set Query
---------

Return Statement
----------------

Create Graph
------------

```gql
CREATE GRAPH <identifier>
```

This query will create the graph `<identifier>`, and set the current graph to `<identifier>`. If the graph already exists, it will return an error.

Example:

```gql
CREATE GRAPH new_graph
CREATE (:Label)
```

This create a graph `new_graph` with a single node of label `Label`.

Use
---

```gql
USE <identifier>
```

This query switch the current graph to the one called `<identifier>`. If no such graph exists, the query execution returns an error.

Example:

```gql
MATCH (a)
USE new_graph
MATCH (b)
```

The first `MATCH` is executed against the `default` graph. While the second is executed against `new_graph`.

Functions
---------

Aggregations
------------

* `avg`
* `collect`
* `count`
* `max`
* `min`
* `sum`
