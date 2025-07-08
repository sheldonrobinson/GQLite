#include "gqlite.h"
#include <format>
#include <string>

namespace gqlite
{
  /**
   * Represent the value type.
   */
  enum class value_type
  {
    invalid,
    boolean,
    integer,
    floating_point,
    string,
    map,
    vector
  };

  struct base_formatter
  {
    template<class ParseContext>
    constexpr ParseContext::iterator parse(ParseContext& ctx)
    {
      auto it = ctx.begin(), end = ctx.end();

      if(it != end && *it != '}')
        throw std::format_error("invalid format");

      return it;
    }
  };

} // namespace gqlite

template<>
struct std::formatter<gqlite::value_type> : std::formatter<const char*>
{

  static const char* to_string(gqlite::value_type _type)
  {
    switch(_type)
    {
      using enum gqlite::value_type;
    case invalid:
      return "invalid";
    case boolean:
      return "boolean";
    case integer:
      return "integer";
    case floating_point:
      return "number";
    case string:
      return "string";
    case map:
      return "map";
    case vector:
      return "vector";
    }
    return "unknown value type";
  }
  template<typename FormatContext>
  auto format(const gqlite::value_type& p, FormatContext& ctx) const
  {
    return std::formatter<const char*>::format(to_string(p), ctx);
  }
};

template<>
struct std::formatter<gqlite::value> : std::formatter<std::string>
{
  template<typename FormatContext>
  auto format(const gqlite::value& p, FormatContext& ctx) const
  {
    return std::formatter<std::string>::format(p.to_string(), ctx);
  }
};
