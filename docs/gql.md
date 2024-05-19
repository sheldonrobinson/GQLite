GQL Query Language
==================

GQL is an ISO standard for a graph query language, it is a derivative of the [OpenCypher](opencypher.md) language. This document covers the implementation of GQL in GQLite.

*Note:* the GQL specification is hidden behind a paywall, and is not available to the authors of GQLite. The implementation of GQL is based on secondary sources.


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
