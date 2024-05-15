#pragma once

namespace gqlite::oc::algebra
{
  enum class expression_type
  {
    empty, value, boolean, integer, floating_point, string, map, vector, node, edge, path
  };
  struct expression_info
  {
    expression_type type = expression_type::empty;
    bool constant = true;
    bool aggregation_result;
  };
}
