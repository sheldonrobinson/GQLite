#include "../../src/table.h"

#include <iostream>

#include "catch_amalgamated.hpp"

template<typename _T_>
std::ostream& operator<<(std::ostream& _os, const gqlite::table<_T_>& _t)
{
  _os << "(" << _t.get_rows_count() << "x" << _t.get_columns_count() << "){\n";
  for(std::size_t r = 0; r < _t.get_rows_count(); ++r)
  {
    typename gqlite::table<_T_>::const_row_view rv = _t.get_row(r);
    _os << " ";
    for(std::size_t c = 0; c < _t.get_columns_count(); ++c)
    {
      _os << " " << rv[c]; 
    }
    _os << "\n";
  }
  _os << "}";
  return _os;
}

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

TEST_CASE( "Table Sortering", "[table]" )
{
  using table = gqlite::table<int>;
  {
    table t;
    t.add_columns({"a", "b", "c"});
    t.set_rows({1, 2, 3,
                1, 3, 4,
                1, 1, 2,
                1, 2, 2,
                1, 2, 4,
                2, 2, 3,
                0, 2, 3,
                0, 1, 3,
                0, 3, 3});
    
    t.sort({1,2});
    for(std::size_t r = 1; r < t.get_rows_count(); ++r)
    {
      table::const_row_view r0 = t.get_row(r-1);
      table::const_row_view r1 = t.get_row(r);
      CHECK( (r0[1] < r1[1] or (r0[1] == r1[1] and r0[2] <= r1[2] )) );
    }
    t.reverse_sort({0,2});
    for(std::size_t r = 1; r < t.get_rows_count(); ++r)
    {
      table::const_row_view r0 = t.get_row(r-1);
      table::const_row_view r1 = t.get_row(r);
      CHECK( (r0[0] > r1[0] or (r0[0] == r1[0] and r0[2] >= r1[2] )) );
    }
  }
  // random tables
  for(std::size_t i = 0; i < 100; ++i)
  {
    table t;

    std::size_t rows = 1 + rand() % 100;
    std::size_t cols = 1 + rand() % 10;
    for(std::size_t c = 0; c < cols; ++c)
    {
      t.add_column(std::to_string(c));
    }
    for(std::size_t r = 0; r < rows; ++r)
    {
      std::vector<int> row;
      for(std::size_t c = 0; c < cols; ++c)
      {
        row.push_back(50 - rand() % 100);
      }
      t.add_row(row);
    }
    std::vector<std::size_t> sv;
    for(std::size_t c = 0; c < rand() % cols; ++c)
    {
      sv.push_back(rand() % cols);
    }
    t.sort(sv);
    for(std::size_t r = 1; r < t.get_rows_count(); ++r)
    {
      table::const_row_view r0 = t.get_row(r-1);
      table::const_row_view r1 = t.get_row(r);
      for(std::size_t ri : sv)
      {
        CHECK(r0[ri] <= r1[ri]);
        if(r0[ri] <= r1[ri]) break;
      }
    }
    sv.clear();
    for(std::size_t c = 0; c < rand() % cols; ++c)
    {
      sv.push_back(rand() % cols);
    }
    t.reverse_sort(sv);
    for(std::size_t r = 1; r < t.get_rows_count(); ++r)
    {
      table::const_row_view r0 = t.get_row(r-1);
      table::const_row_view r1 = t.get_row(r);
      for(std::size_t ri : sv)
      {
        CHECK(r0[ri] >= r1[ri]);
        if(r0[ri] >= r1[ri]) break;
      }
    }
  }
}

TEST_CASE( "Table Resizing", "[table]" )
{
  using table = gqlite::table<int>;
  table t;
  t.add_columns({"a", "b", "c"});
  t.set_rows({1, 2, 3,
              1, 3, 4,
              1, 1, 2,
              1, 2, 2,
              1, 2, 4,
              2, 2, 3,
              0, 2, 3,
              0, 1, 3,
              0, 3, 3});
  t.limit(5);
  table tc;
  tc.add_columns({"a", "b", "c"});
  tc.set_rows({1, 2, 3,
              1, 3, 4,
              1, 1, 2,
              1, 2, 2,
              1, 2, 4});
  CHECK( t == tc );
  t.offset(2);

  tc.set_rows({1, 1, 2,
              1, 2, 2,
              1, 2, 4});
  CHECK( t == tc );
}
