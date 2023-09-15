project(gqlite)

cmake_minimum_required(VERSION 3.10)
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
    DESCRIPTION "SQLite is a C-language library that implements a small, fast, self-contained, high-reliability, full-featured, SQL connection engine."
    URL "http://www.sqlite.org"
    TYPE OPTIONAL
    PURPOSE "Required for sqlite module.")

########################
## Find clog

find_package(clog)
set_package_properties(sqlite PROPERTIES
    DESCRIPTION "clog library."
    TYPE OPTIONAL
    PURPOSE "Used for debug messages")

########################
## Find readline

find_package(Readline)
set_package_properties(Readline PROPERTIES
    DESCRIPTION "Readline library."
    TYPE OPTIONAL
    PURPOSE "Used for to read from input")

########################
## Find python3

find_package (Python3 COMPONENTS Interpreter Development)
set_package_properties(Python3 PROPERTIES
    DESCRIPTION ""
    TYPE OPTIONAL
    PURPOSE "Used for python3 bindings.")

########################
## Find Ruby

find_package(Ruby)
set_package_properties(Ruby PROPERTIES
    DESCRIPTION ""
    TYPE OPTIONAL
    PURPOSE "Used for Ruby bindings.")

########################
## Find RSpec

find_package(RSpec)
set_package_properties(RSpec PROPERTIES
    DESCRIPTION ""
    TYPE OPTIONAL
    PURPOSE "Used for testing ruby bindings.")

########################
## Find Cucumber

find_package(Cucumber)
set_package_properties(Cucumber PROPERTIES
    DESCRIPTION ""
    TYPE OPTIONAL
    PURPOSE "Used for running the conformance test suite.")

########################
## Find cffi

if(Python3_FOUND)
  execute_process(
    COMMAND
      ${Python3_EXECUTABLE} "-c" "import cffi"
    RESULT_VARIABLE PYTHON3_CFFI_STATUS
    ERROR_QUIET
    OUTPUT_STRIP_TRAILING_WHITESPACE
    )
endif()

if(NOT PYTHON3_CFFI_STATUS)
  set(PYTHON3_CFFI_FOUND TRUE)
endif()

set_package_properties(PYTHON3_CFFI PROPERTIES
    DESCRIPTION "Python3's cffi package"
    TYPE OPTIONAL
    PURPOSE "Used for python3 bindings.")

########################
## Find Crystal

find_package(Crystal)
set_package_properties(Crystal PROPERTIES
    DESCRIPTION ""
    TYPE OPTIONAL
    PURPOSE "Used for running the crsytal test suite.")

########################
## Build library

include_directories(include ${CMAKE_CURRENT_BINARY_DIR}/src)

if(SQLITE_FOUND)
set(GQLITE_HAVE_SQLITE 1)
endif()

if(GQLITE_FOUND)
set(GQLITE_HAVE_CLOG 1)
endif()

configure_file(src/config.h.in src/config.h)

set(GQLITE_SRCS
    src/backend.cpp
    src/connection.cpp
    src/exception.cpp
    src/gqlite-c.cpp
    src/value.cpp
    src/oc/lexer.cpp
    src/oc/token.cpp
    src/oc/parser.cpp
    src/oc/algebra/abstract_node_visitor.cpp
    src/oc/algebra/node.cpp
    src/oc/algebra/node_type.cpp
    src/oc/algebra/nodes.cpp
    src/oc/algebra/visitors/print.cpp
    src/backends/functions.cpp
    src/backends/sqlite.cpp
  )

add_library(gqlite SHARED ${GQLITE_SRCS})

target_link_libraries(gqlite PRIVATE ${SQLITE3_LIBRARY})

install(TARGETS gqlite EXPORT gqliteTargets ${INSTALL_TARGETS_DEFAULT_ARGS} )

install(FILES include/gqlite-c.h
      DESTINATION ${INSTALL_INCLUDE_DIR}/)

########################
## Build python

if(Python3_FOUND AND PYTHON3_CFFI_FOUND)
  set(Python3_SHORT_VERSION "${Python3_VERSION_MAJOR}.${Python3_VERSION_MINOR}")

  set(PYTHON_SITE_PACKAGES_INSTALL_DIR ${INSTALL_LIB_DIR}/python${Python3_SHORT_VERSION}/dist-packages/)

  add_custom_target(python3_ffi_lib ALL
      COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py ${CMAKE_CURRENT_SOURCE_DIR}
      WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
      DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py
  )
  install(FILES ${CMAKE_CURRENT_BINARY_DIR}/__c_gqlite.cpython-${Python3_VERSION_MAJOR}${Python3_VERSION_MINOR}-x86_64-linux-gnu.so DESTINATION ${PYTHON_SITE_PACKAGES_INSTALL_DIR}/)
  install(DIRECTORY bindings/python/gqlite DESTINATION ${PYTHON_SITE_PACKAGES_INSTALL_DIR})
endif()

########################
## Build ruby

install(DIRECTORY bindings/ruby DESTINATION ${INSTALL_LIB_DIR})

########################
## Build tools

if(READLINE_FOUND)
  add_executable(gqlite_cmd tools/gqlite.cpp)
  target_link_libraries(gqlite_cmd PRIVATE gqlite ${Readline_LIBRARY})
  set_property(TARGET gqlite_cmd PROPERTY OUTPUT_NAME gqlite)

  install(TARGETS gqlite_cmd EXPORT gqliteTargets ${INSTALL_TARGETS_DEFAULT_ARGS} )
endif()

########################
## Build examples

add_executable(gqlite_c_example examples/main.c)
target_link_libraries(gqlite_c_example PRIVATE gqlite)
add_executable(gqlite_cpp_example examples/main.cpp)
target_link_libraries(gqlite_cpp_example PRIVATE gqlite)

###########################
## COPY dists to binary dir

file(GLOB_RECURSE DISTS_FILES dists/*)
list(APPEND FOUND_FILES ${DISTS_FILES})
foreach(DF ${FOUND_FILES})
  cmake_path(GET DF RELATIVE_PART TRUNCF)
  file(RELATIVE_PATH RELATIVEF ${CMAKE_CURRENT_SOURCE_DIR} ${DF})
  cmake_path(GET RELATIVEF PARENT_PATH RELATIVEF_DIR)
  file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/${RELATIVEF_DIR})
  file(CREATE_LINK ${DF} ${CMAKE_CURRENT_BINARY_DIR}/${RELATIVEF})
endforeach()

# For Ruby

file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite.h ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/ext/gqlite/gqlite.h)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite-c.h ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/ext/gqlite/gqlite-c.h)

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/lib)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/ruby/gqlite.rb ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/lib/gqlite.rb)

# For Python

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/python/gqlite)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/gqlite/__init__.py ${CMAKE_CURRENT_BINARY_DIR}/dists/python/gqlite/__init__.py)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py ${CMAKE_CURRENT_BINARY_DIR}/dists/python/build-ffi.py)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite.h ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src/gqlite.h)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite-c.h ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src/gqlite-c.h)

##########################
# Generate amalgamate
if(Ruby_EXECUTABLE)
  add_custom_target(gqlite_amalgamate ALL
    COMMAND ${Ruby_EXECUTABLE} ${CMAKE_CURRENT_BINARY_DIR}/dists/create_amalgamate.rb ${CMAKE_CURRENT_SOURCE_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/dists/gqlite-amalgamate.cpp ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/ext/gqlite/gqlite-amalgamate.cpp
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/dists/gqlite-amalgamate.cpp ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src/gqlite-amalgamate.cpp
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/
    DEPENDS ${GQLITE_SRCS})
endif()

########################
## Build tests

add_executable(gqlite_cpp_tests tests/cpp/catch_amalgamated.cpp tests/cpp/test_table.cpp)
target_link_libraries(gqlite_cpp_tests PRIVATE gqlite)

if(Crystal_FOUND)
  add_custom_target(gqlite-cr-spec ALL
  COMMAND ${CMAKE_COMMAND} -E env CRYSTAL_PATH=${CMAKE_CURRENT_SOURCE_DIR}/bindings/crystal/:${Crystal_PATH} ${Crystal_EXECUTABLE} build --link-flags "-L${CMAKE_CURRENT_BINARY_DIR}" ${CMAKE_CURRENT_SOURCE_DIR}/tests/cr/gqlite-spec.cr -o gqlite-cr-spec
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    DEPENDS gqlite ${CMAKE_CURRENT_SOURCE_DIR}/tests/cr/gqlite-spec.cr ${CMAKE_CURRENT_SOURCE_DIR}/bindings/crystal/gqlite.cr
  )
endif()

########################
## Run tests

add_test(NAME gqlite-cpp COMMAND ${CMAKE_CURRENT_BINARY_DIR}/gqlite_cpp_tests)

if(RSPEC_FOUND)
  add_test(NAME gqlite-rb-rspec COMMAND ${RSPEC_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/gqlite-rspec.rb  )
endif()

if(CUCUMBER_FOUND)
  add_test(NAME gqlite-rb-tck COMMAND ${Ruby_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/gqlite-tck.rb WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/)
endif()

if(Python3_FOUND AND PYTHON3_CFFI_FOUND)
  add_test(NAME gqlite-py COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/py/python-unittest.py)
endif()

if(Crystal_FOUND)
  add_test(NAME gqlite-cr-spec COMMAND ${CMAKE_CURRENT_BINARY_DIR}/gqlite-cr-spec )
endif()
