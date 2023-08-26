project(gqlite)

set(CMAKE_MODULE_PATH ${CMAKE_SOURCE_DIR}/cmake/modules)

########################
## Define version

set(GQLITE_MAJOR_VERSION 1)
set(GQLITE_MINOR_VERSION 1)
set(GQLITE_PATCH_VERSION 0)
set(GQLITE_VERSION ${GQLITE_MAJOR_VERSION}.${GQLITE_MINOR_VERSION}.${GQLITE_PATCH_VERSION})

########################
## Define install dirs

set(INSTALL_LIB_DIR     lib${LIB_SUFFIX}/               CACHE PATH "Installation directory for libraries")
set(INSTALL_INCLUDE_DIR include/                        CACHE PATH "Installation directory for headers")
set(INSTALL_BIN_DIR     bin/                            CACHE PATH "Installation directory for executables")
set(INSTALL_SHARE_DIR   share/gqlite/                   CACHE PATH "Installation directory for data")
set(INSTALL_DOC_DIR     share/doc/gqlite/               CACHE PATH "Installation directory for documentation")

if(WIN32 AND NOT CYGWIN)
  set(DEF_INSTALL_CMAKE_DIR cmake)
else()
  set(DEF_INSTALL_CMAKE_DIR lib/cmake/gqlite)
endif()
set(INSTALL_CMAKE_DIR ${DEF_INSTALL_CMAKE_DIR} CACHE PATH "Installation directory for CMake files")

# Make relative paths absolute (needed later on)
foreach(p LIB BIN INCLUDE SHARE DOC CMAKE MODULES SPECS)
  set(var INSTALL_${p}_DIR)
  if(NOT IS_ABSOLUTE "${${var}}")
    set(${var} "${CMAKE_INSTALL_PREFIX}/${${var}}")
  endif()
endforeach()

#
# Define INSTALL_TARGETS_DEFAULT_ARGS to be used as install target for program and library.
# It will do the right thing on all platform
#
set(INSTALL_TARGETS_DEFAULT_ARGS  RUNTIME DESTINATION "${INSTALL_BIN_DIR}" COMPONENT bin
                                  LIBRARY DESTINATION "${INSTALL_LIB_DIR}" COMPONENT shlib
                                  ARCHIVE DESTINATION "${INSTALL_LIB_DIR}" COMPONENT Devel )

########################
## RPATH

SET(CMAKE_INSTALL_RPATH "${INSTALL_LIB_DIR}")
SET(CMAKE_INSTALL_RPATH_USE_LINK_PATH TRUE)

# the RPATH to be used when installing, but only if it's not a system directory
LIST(FIND CMAKE_PLATFORM_IMPLICIT_LINK_DIRECTORIES "${CMAKE_INSTALL_PREFIX}/lib" isSystemDir)
IF("${isSystemDir}" STREQUAL "-1")
   SET(CMAKE_INSTALL_RPATH "${CMAKE_INSTALL_PREFIX}/lib")
ENDIF()

########################
## enable testing
enable_testing()

########################
## Feature summary

include(FeatureSummary)

##########################
# Enable C++20

set(CMAKE_CXX_STANDARD 20)


########################
## Find SQLite

find_package(SQLite)

set_package_properties(SQLite PROPERTIES
    DESCRIPTION "SQLite is a C-language library that implements a small, fast, self-contained, high-reliability, full-featured, SQL database engine."
    URL "http://www.sqlite.org"
    TYPE OPTIONAL
    PURPOSE "Required for sqlite module.")

find_package(clog)
set_package_properties(sqlite PROPERTIES
    DESCRIPTION "clog library."
    TYPE OPTIONAL
    PURPOSE "Used for debug messages")

include_directories(include ${CMAKE_CURRENT_BINARY_DIR}/src)

if(SQLITE_FOUND)
set(GQLITE_HAVE_SQLITE 1)
endif()

if(GQLITE_FOUND)
set(GQLITE_HAVE_CLOG 1)
endif()

configure_file(src/config.h.in src/config.h)

add_library(gqlite SHARED
  src/backend.cpp
  src/database.cpp
  src/gqlite-c.cpp
  src/result.cpp
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

target_link_libraries(gqlite ${SQLITE3_LIBRARY})

install(TARGETS gqlite EXPORT gqliteTargets ${INSTALL_TARGETS_DEFAULT_ARGS} )

install(FILES include/gqlite-c.h
      DESTINATION ${INSTALL_INCLUDE_DIR}/)
  
install(DIRECTORY bindings/ruby DESTINATION ${INSTALL_LIB_DIR})