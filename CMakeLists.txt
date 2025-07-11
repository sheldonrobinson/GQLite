cmake_minimum_required(VERSION 3.10)
project(gqlite)

set(CMAKE_MODULE_PATH ${CMAKE_SOURCE_DIR}/cmake/modules)

########################
## Define version

set(GQLITE_MAJOR_VERSION 1)
set(GQLITE_MINOR_VERSION 2)
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

##########################
# Enable all warnings

if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Werror -Wall -Wextra")
endif()

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
if(Ruby_EXECUTABLE)
set(Ruby_FOUND 1)
endif()
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

####################################
## Corrosion (build rust from CMake)

include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5 # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH Cargo.toml)

########################
## Build C++ API

set(GQLITE_CXX_SRC
    bindings/cpp/connection.cpp
    bindings/cpp/exception.cpp
    bindings/cpp/value.cpp)

add_library(gqlitecpp SHARED ${GQLITE_CXX_SRC} )
target_link_libraries(gqlitecpp PUBLIC gqlitedb)
set_property(TARGET gqlitecpp PROPERTY OUTPUT_NAME gqlite)
set_target_properties(gqlitecpp PROPERTIES VERSION ${GQLITE_VERSION} SOVERSION ${GQLITE_MAJOR_VERSION})

########################
## Install library

include_directories(BEFORE include ${CMAKE_CURRENT_BINARY_DIR}/src)

install(FILES ${CMAKE_CURRENT_BINARY_DIR}/libgqlitedb.so DESTINATION ${INSTALL_LIB_DIR})
install(TARGETS gqlitecpp EXPORT GQLiteTargets ${INSTALL_TARGETS_DEFAULT_ARGS} )
install(FILES include/gqlite-c.h  include/gqlite.h DESTINATION ${INSTALL_INCLUDE_DIR}/)

########################
## Install cmd line tool

INSTALL(FILES ${CMAKE_CURRENT_BINARY_DIR}/gqlite DESTINATION ${INSTALL_BIN_DIR} PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE)

########################
## Build python

if(Python3_FOUND AND PYTHON3_CFFI_FOUND)
  set(Python3_SHORT_VERSION "${Python3_VERSION_MAJOR}.${Python3_VERSION_MINOR}")

  set(PYTHON_SITE_PACKAGES_INSTALL_DIR ${INSTALL_LIB_DIR}/python${Python3_SHORT_VERSION}/dist-packages/)

  add_custom_target(python3_ffi_lib ALL
      COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py ${CMAKE_CURRENT_SOURCE_DIR}
      WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
      DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py gqlitecpp
  )
  install(FILES ${CMAKE_CURRENT_BINARY_DIR}/__c_gqlite.cpython-${Python3_VERSION_MAJOR}${Python3_VERSION_MINOR}-x86_64-linux-gnu.so DESTINATION ${PYTHON_SITE_PACKAGES_INSTALL_DIR}/)
  install(DIRECTORY bindings/python/gqlite DESTINATION ${PYTHON_SITE_PACKAGES_INSTALL_DIR})
endif()

########################
## Build ruby

install(DIRECTORY bindings/ruby DESTINATION ${INSTALL_LIB_DIR})

########################
## Build examples

add_executable(gqlite_c_example examples/main.c)
target_link_libraries(gqlite_c_example PRIVATE gqlitecpp)
add_executable(gqlite_cpp_example examples/main.cpp)
target_link_libraries(gqlite_cpp_example PRIVATE gqlitecpp)

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

file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite-c.h ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/ext/gqlite/gqlite-c.h)

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/lib)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/ruby/gqlite.rb ${CMAKE_CURRENT_BINARY_DIR}/dists/ruby/lib/gqlite.rb)

# For Python

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/python/gqlite)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/gqlite/__init__.py ${CMAKE_CURRENT_BINARY_DIR}/dists/python/gqlite/__init__.py)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/bindings/python/build-ffi.py ${CMAKE_CURRENT_BINARY_DIR}/dists/python/build-ffi.py)
file(CREATE_LINK ${CMAKE_CURRENT_SOURCE_DIR}/include/gqlite-c.h ${CMAKE_CURRENT_BINARY_DIR}/dists/python/src/gqlite-c.h)

########################
## Build tests

add_executable(gqlite_cpp_tests tests/cpp/catch_amalgamated.cpp tests/cpp/test_value.cpp)
target_link_libraries(gqlite_cpp_tests PRIVATE gqlitecpp)

if(Crystal_FOUND)
  add_custom_target(gqlite-cr-spec ALL
  COMMAND ${CMAKE_COMMAND} -E env CRYSTAL_PATH=${CMAKE_CURRENT_SOURCE_DIR}/bindings/crystal/:${Crystal_PATH} ${Crystal_EXECUTABLE} build --link-flags "-L${CMAKE_CURRENT_BINARY_DIR}" ${CMAKE_CURRENT_SOURCE_DIR}/tests/cr/gqlite-spec.cr -o gqlite-cr-spec
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    DEPENDS gqlitecpp ${CMAKE_CURRENT_SOURCE_DIR}/tests/cr/gqlite-spec.cr ${CMAKE_CURRENT_SOURCE_DIR}/bindings/crystal/gqlite.cr
  )
endif()

########################
## Run tests

option(GQLITE_TESTS_OUTPUT_JUNIT "Make GQLite test output junit" OFF)
option(GQLITE_TESTS_DISABLE_RUST_TESTS "Disable rust unittest (intended for CI)" OFF)

if(GQLITE_TESTS_OUTPUT_JUNIT)
  make_directory(${CMAKE_BINARY_DIR}/Testing/junit)
  set(GQLITE_CPP_EXTRA_ARGS -r junit -o ${CMAKE_BINARY_DIR}/Testing/junit/TEST-gqlite-cpp.xml )
  set(GQLITE_RSPEC_EXTRA_ARGS --format RspecJunitFormatter --out ${CMAKE_BINARY_DIR}/Testing/junit/TEST-gqlite-rb-rspec.xml)
  set(GQLITE_TCK_EXTRA_ARGS --output-junit)
  set(GQLITE_PY_EXTRA_ARGS -m pytest --junitxml Testing/junit/TEST-gqlite-py-unittest.xml)
endif()

add_test(NAME gqlite-cpp COMMAND ${CMAKE_CURRENT_BINARY_DIR}/gqlite_cpp_tests ${GQLITE_CPP_EXTRA_ARGS})

if(NOT GQLITE_TESTS_DISABLE_RUST_TESTS)
  add_test(NAME gqlite-rs COMMAND
      ${CMAKE_COMMAND} -E env
      CARGO_TARGET_DIR=${CMAKE_BINARY_DIR}/cargo/build
      ${Rust_CARGO_CACHED} test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/)
endif()

if(RSPEC_FOUND)
  add_test(NAME gqlite-rb-rspec COMMAND ${RSPEC_EXECUTABLE} ${GQLITE_RSPEC_EXTRA_ARGS} ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/gqlite-rspec.rb  )
endif()

if(CUCUMBER_FOUND)
  add_test(NAME gqlite-rb-tck COMMAND ${Ruby_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/gqlite-tck.rb ${GQLITE_TCK_EXTRA_ARGS} --output-dir ${CMAKE_BINARY_DIR}/Testing/junit WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tests/rb/)
endif()

if(Python3_FOUND AND PYTHON3_CFFI_FOUND)
  add_test(NAME gqlite-py COMMAND ${Python3_EXECUTABLE} ${GQLITE_PY_EXTRA_ARGS} ${CMAKE_CURRENT_SOURCE_DIR}/tests/py/python-unittest.py)
endif()

if(Crystal_FOUND)
  add_test(NAME gqlite-cr-spec COMMAND ${CMAKE_CURRENT_BINARY_DIR}/gqlite-cr-spec )
endif()

########################
## Config files

# export(TARGETS ${PROJECT_EXPORTED_TARGETS}
# FILE "${PROJECT_BINARY_DIR}/GQLiteTargets.cmake")

include(CMakePackageConfigHelpers)
configure_package_config_file(GQLiteConfig.cmake.in "${PROJECT_BINARY_DIR}/GQLiteConfig.cmake"
  INSTALL_DESTINATION ${INSTALL_CMAKE_DIR}
  PATH_VARS INSTALL_INCLUDE_DIR
  )
write_basic_package_version_file("${PROJECT_BINARY_DIR}/GQLiteConfigVersion.cmake" VERSION ${GQLITE_VERSION} COMPATIBILITY SameMajorVersion)

# Install the GQLiteConfig.cmake and GQLiteConfigVersion.cmake
install(FILES
"${PROJECT_BINARY_DIR}/GQLiteConfig.cmake"
"${PROJECT_BINARY_DIR}/GQLiteConfigVersion.cmake"
DESTINATION "${INSTALL_CMAKE_DIR}" COMPONENT dev)

# Install the export set for use with the install-tree
# install(EXPORT GQLiteTargets DESTINATION "${INSTALL_CMAKE_DIR}" COMPONENT dev)

########################
## Feature summary

feature_summary(WHAT ALL FATAL_ON_MISSING_REQUIRED_PACKAGES)
