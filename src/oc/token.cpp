#include "token.h"

#include <assert.h>
#include <iostream>

using namespace gqlite::oc;

const char* gqlite::oc::token_type_to_string(token_type _type)
{
  switch(_type)
  {
    using enum token_type;
    case UNFINISHED_STRING:
      return "unfinished string";
    case UNFINISHED_COMMENT:
      return "unfinished comment";
    case END_OF_FILE:
      return "end of file";
    case UNKNOWN:
      return "unknown";
    case SEMI:
      return ";";
    case STARTBRACE:
      return "{";
    case ENDBRACE:
      return "}";
    case STARTBRACKET:
      return "(";
    case ENDBRACKET:
      return ")";
    case STARTBOXBRACKET:
      return "[";
    case ENDBOXBRACKET:
      return "]";
    case PIPE:
      return "|";
    case MINUS:
      return "-";
    case COMMA:
      return ",";
    case DOT:
      return ".";
    case LEFT_ARROW:
      return "<-";
    case RIGHT_ARROW:
      return "->";
    case COLON:
      return ":";
    case COLONCOLON:
      return "::";
    case EQUAL:
      return "=";
    case DIFFERENT:
      return "!=";
    case SUPERIOR:
      return ">";
    case INFERIOR:
      return "<";
    case SUPERIOR_EQUAL:
      return ">=";
    case INFERIOR_EQUAL:
      return "<=";
    case PLUS:
      return "+";
    case STAR:
      return "*";
    case DIVIDE:
      return "/";
    case EXCLAMATION:
      return "!";
    case PLUS_EQUAL:
      return "+=";
    case IDENTIFIER:
      return "identifier";
    case STRING:
      return "string";
#define TOKEN_KEYWORD(_K_)  \
    case _K_:               \
      return # _K_;
#define TOKEN_KEYWORD2(_K_, _S_)  \
    case _K_:                     \
      return _S_;
    #include "token_keywords.h"
#undef TOKEN_KEYWORD
#undef TOKEN_KEYWORD2
  }
  std::cerr << "add the token" << (int)_type << std::endl;
  std::abort();
}

token::token() : type(token_type::UNKNOWN)
{

}

token::token(token_type _type, int _line, int _column) : type(_type), line(_line), column(_column)
{
}

token::token(token_type _type, const std::string& _string, int _line, int _column) : type(_type), line(_line), column(_column), string(_string)
{
}
