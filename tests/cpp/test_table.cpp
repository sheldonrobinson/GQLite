#include "../../src/table.h"

#include "catch_amalgamated.hpp"

TEST_CASE( "Table", "[table]" )
{
  using table = gqlite::table<int>;

  table t;
  REQUIRE( t.get_columns_count() == 0);
  REQUIRE( t.get_columns_names().empty() );
  REQUIRE( t.get_rows_count() == 0);

  t.add_column("a");

  REQUIRE( t.get_columns_count() == 1);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a"});
  REQUIRE( t.get_rows_count() == 0);

  t.add_row({1});

  REQUIRE( t.get_columns_count() == 1);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a"});
  REQUIRE( t.get_rows_count() == 1);
  REQUIRE( t.get_value(0,0) == 1 );
  REQUIRE( t.get_value("a",0) == 1 );
  REQUIRE( t.get_row(0) == std::vector<int>{1} );

  t.add_row({2});

  REQUIRE( t.get_columns_count() == 1);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a"});
  REQUIRE( t.get_rows_count() == 2);
  REQUIRE( t.get_value(0,0) == 1 );
  REQUIRE( t.get_value(0,1) == 2 );
  REQUIRE( t.get_value("a",0) == 1 );
  REQUIRE( t.get_value("a",1) == 2 );
  REQUIRE( t.get_row(0) == std::vector<int>{1} );
  REQUIRE( t.get_row(1) == std::vector<int>{2} );

  t.add_columns({"b", "c"}, -1);

  REQUIRE( t.get_columns_count() == 3);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a", "b", "c"});
  REQUIRE( t.get_rows_count() == 2);
  REQUIRE( t.get_value(0,0) == 1 );
  REQUIRE( t.get_value(1,0) == -1 );
  REQUIRE( t.get_value(2,0) == -1 );
  REQUIRE( t.get_value(0,1) == 2 );
  REQUIRE( t.get_value(1,1) == -1 );
  REQUIRE( t.get_value(2,1) == -1 );
  REQUIRE( t.get_value("a",0) == 1 );
  REQUIRE( t.get_value("b",0) == -1 );
  REQUIRE( t.get_value("c",0) == -1 );
  REQUIRE( t.get_value("a",1) == 2 );
  REQUIRE( t.get_value("b",1) == -1 );
  REQUIRE( t.get_value("c",1) == -1 );
  REQUIRE( t.get_row(0) == std::vector<int>{1, -1, -1} );
  REQUIRE( t.get_row(1) == std::vector<int>{2, -1, -1} );

  t.add_row({3, 4, 5});

  REQUIRE( t.get_columns_count() == 3);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a", "b", "c"});
  REQUIRE( t.get_rows_count() == 3);
  REQUIRE( t.get_value(0,0) == 1 );
  REQUIRE( t.get_value(1,0) == -1 );
  REQUIRE( t.get_value(2,0) == -1 );
  REQUIRE( t.get_value(0,1) == 2 );
  REQUIRE( t.get_value(1,1) == -1 );
  REQUIRE( t.get_value(2,1) == -1 );
  REQUIRE( t.get_value(0,2) == 3 );
  REQUIRE( t.get_value(1,2) == 4 );
  REQUIRE( t.get_value(2,2) == 5 );
  REQUIRE( t.get_value("a",0) == 1 );
  REQUIRE( t.get_value("b",0) == -1 );
  REQUIRE( t.get_value("c",0) == -1 );
  REQUIRE( t.get_value("a",1) == 2 );
  REQUIRE( t.get_value("b",1) == -1 );
  REQUIRE( t.get_value("c",1) == -1 );
  REQUIRE( t.get_value("a",2) == 3 );
  REQUIRE( t.get_value("b",2) == 4 );
  REQUIRE( t.get_value("c",2) == 5 );
  REQUIRE( t.get_row(0) == std::vector<int>{1, -1, -1} );
  REQUIRE( t.get_row(1) == std::vector<int>{2, -1, -1} );
  REQUIRE( t.get_row(2) == std::vector<int>{3, 4, 5} );

  t.add_columns({"d"}, 6);

  REQUIRE( t.get_columns_count() == 4);
  REQUIRE( t.get_columns_names() == std::vector<std::string>{"a", "b", "c", "d"});
  REQUIRE( t.get_rows_count() == 3);
  REQUIRE( t.get_value(0,0) == 1 );
  REQUIRE( t.get_value(1,0) == -1 );
  REQUIRE( t.get_value(2,0) == -1 );
  REQUIRE( t.get_value(3,0) == 6 );
  REQUIRE( t.get_value(0,1) == 2 );
  REQUIRE( t.get_value(1,1) == -1 );
  REQUIRE( t.get_value(2,1) == -1 );
  REQUIRE( t.get_value(3,1) == 6 );
  REQUIRE( t.get_value(0,2) == 3 );
  REQUIRE( t.get_value(1,2) == 4 );
  REQUIRE( t.get_value(2,2) == 5 );
  REQUIRE( t.get_value(3,2) == 6 );
  REQUIRE( t.get_value("a",0) == 1 );
  REQUIRE( t.get_value("b",0) == -1 );
  REQUIRE( t.get_value("c",0) == -1 );
  REQUIRE( t.get_value("d",0) == 6 );
  REQUIRE( t.get_value("a",1) == 2 );
  REQUIRE( t.get_value("b",1) == -1 );
  REQUIRE( t.get_value("c",1) == -1 );
  REQUIRE( t.get_value("d",1) == 6 );
  REQUIRE( t.get_value("a",2) == 3 );
  REQUIRE( t.get_value("b",2) == 4 );
  REQUIRE( t.get_value("c",2) == 5 );
  REQUIRE( t.get_value("d",2) == 6 );
  REQUIRE( t.get_row(0) == std::vector<int>{1, -1, -1, 6} );
  REQUIRE( t.get_row(1) == std::vector<int>{2, -1, -1, 6} );
  REQUIRE( t.get_row(2) == std::vector<int>{3, 4, 5, 6} );


}

