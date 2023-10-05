#include <gqlite.h>

#include "catch_amalgamated.hpp"

TEST_CASE( "value_map", "[value]" )
{
  gqlite::value_map vm;
  vm["name"] = "Dan dong.";
  CHECK( gqlite::value(vm).to_json() == R"({"name":"Dan dong."})");

  gqlite::value val = gqlite::value::from_json(R"({"num":2,"name":"Ann Darrow"})");
  vm = val.to_map();
  CHECK(vm["num"].to_integer() == 2);
  CHECK(vm["name"].to_string() == "Ann Darrow");
  CHECK( gqlite::value(vm).to_json() == R"({"name":"Ann Darrow","num":2})");
}
