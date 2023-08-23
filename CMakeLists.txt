project(gqlite)

set(CMAKE_CXX_STANDARD 20)


find_package(sqlite)
find_package(clog)

include_directories(include ${CMAKE_CURRENT_BINARY_DIR}/src)

if(SQLITE_FOUND)
set(GQLITE_HAVE_SQLITE 1)
endif()

if(GQLITE_FOUND)
set(GQLITE_HAVE_CLOG 1)
endif()

configure_file(src/config.h.in src/config.h)

add_library(gqlite
  src/database.cpp
  src/gqlite-c.cpp
  src/oc/lexer.cpp
  src/oc/token.cpp
  src/oc/parser.cpp
  src/oc/algebra/abstract_node_visitor.cpp
  src/oc/algebra/node.cpp
  src/oc/algebra/node_type.cpp
  src/oc/algebra/nodes.cpp
  src/oc/algebra/visitors/print.cpp
  src/backends/sqlite.cpp
  )
