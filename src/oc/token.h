#include <string>

namespace gqlite::oc
{
  enum class token_type
  {
  // Not really token
    UNFINISHED_STRING = -4,
    UNFINISHED_COMMENT = -3,
    END_OF_FILE = -2,
    UNKNOWN = - 1,
  // Special characters
    SEMI = 0, ///< ;
    STARTBRACE, ///< {
    ENDBRACE, ///< }
    STARTBRACKET, ///< (
    ENDBRACKET, ///< )
    COMMA, ///< ,
    COLON, ///< :
    COLONCOLON, ///< :
    EQUAL, ///< =
    DOT, ///< .
    LEFT_ARROW, ///< ->
    RIGHT_ARROW, ///< <-
  // Constants
    IDENTIFIER,
    STRING,
  // Keywords
    CREATE,
  };
  static const char* token_type_to_string(token_type _type );
  struct token
  {
    /// type of the token
    token_type type;
    /// line of the token
    int line;
    /// Column of the token
    int column;
    /// String or identifier name
    std::string string;
    
    token();
    /**
      * Creates a token of the given type
      */
    token(token_type _type, int _line, int _column);
    /**
      * Creates an identifier or a string constant
      */
    token(token_type _type, const std::string& _string, int _line, int _column);
  };
} // namespace gqlite::oc
