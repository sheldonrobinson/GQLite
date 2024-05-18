opencypher query language
=========================

[OpenCypher](https://opencypher.org/) is the currently only supported query language for GQLite. The main documentation for the language is [openCypher9.pdf](https://s3.amazonaws.com/artifacts.opencypher.org/openCypher9.pdf). This document covers some basic of cypher, the difference with opencypher and the list of what is implemented.

Difference with opencypher
--------------------------

GQLite currently only support a subset of OpenCypher, but al

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

The list of features under development or planned for future release can be found in [gitlab issues](https://gitlab.com/gqlite/GQLite/-/issues/?label_name%5B%5D=cypher%3Afeature)

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
