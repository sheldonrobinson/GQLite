#include "lexer.h"

#include <assert.h>
#include <iostream>
#include <vector>

#include "token.h"

using namespace gqlite::oc;

struct lexer::data {
  std::istream* stream;
  int lastChar;
  int col;
  int line;
  int followingnewline;
  std::vector<token> ungotten;
};

lexer::lexer(std::istream* sstream) : d(new data)
{
  d->col = 1;
  d->line = 1;
  d->followingnewline = 1;
  d->stream = sstream;
}

lexer::~lexer()
{
  delete d;
}

#define IDENTIFIER_IS_KEYWORD( tokenname, tokenid) \
  if( identifierStr == tokenname) \
  { \
    return token(token_type::tokenid, line(), initial_col); \
  }

#define CHAR_IS_TOKEN( tokenchar, tokenid ) \
  if( lastChar == tokenchar ) \
  { \
    return token(token_type::tokenid, line(), initial_col); \
  }

#define CHAR_IS_TOKEN_OR_TOKEN( tokenchar, tokendecidechar, tokenid_1, tokenid_2 ) \
  if( lastChar == tokenchar  ) \
  { \
    if( get_next_char() == tokendecidechar ) \
    { \
      return token(token_type::tokenid_2, line(), initial_col); \
    } else { \
      unget(); \
      return token(token_type::tokenid_1, line(), initial_col); \
    } \
  }

#define CHAR_IS_TOKEN_OR_TOKEN_OR_TOKEN( tokenchar_1, tokenchar_2, tokenchar_3, tokenid_1, tokenid_2, tokenid_3 ) \
  if( lastChar == tokenchar_1 ) \
  { \
    int nextChar = getNextChar(); \
    if( nextChar == tokenchar_2 ) \
    { \
      return token(token_type::tokenid_2, line(), initial_col); \
    } else if( nextChar  == tokenchar_3 ) \
    { \
      return token(token_type::tokenid_3, line(), initial_col); \
    } else { \
      unget(); \
      return token(token_type::tokenid_1, line(), initial_col); \
    } \
  }

void lexer::unget(const token& _token)
{
  d->ungotten.push_back(_token);
}

token lexer::next_token()
{
  if(not d->ungotten.empty())
  {
    token t = d->ungotten.back();
    d->ungotten.pop_back();
    return t;
  }
  int lastChar = get_next_non_separator_char();
  int initial_line = line() - 1;
  int initial_col = column() - 1;
  if( eof() ) return token(token_type::END_OF_FILE, line(), initial_col);
  std::string identifierStr;
  // Test for comment
  token commenttoken;
  if( ignore_comment( commenttoken, lastChar ) )
  {
    return commenttoken;
  }
  // if it is alpha, it's an identifier or a keyword
  if(std::isalpha(lastChar) or lastChar == '_')
  {
    identifierStr = get_identifier(lastChar);
    
#define TOKEN_KEYWORD(_K_) IDENTIFIER_IS_KEYWORD(# _K_, _K_)
    #include "token_keywords.h"
#undef TOKEN_KEYWORD

    return token(token_type::IDENTIFIER, identifierStr, line(), initial_col);
  } else if(lastChar == '"' or lastChar == '\'' ) {
    return get_string(lastChar);
  } else {
    CHAR_IS_TOKEN(';', SEMI );
    CHAR_IS_TOKEN('.', DOT );
    CHAR_IS_TOKEN( '{', STARTBRACE );
    CHAR_IS_TOKEN( '}', ENDBRACE );
    CHAR_IS_TOKEN( '(', STARTBRACKET );
    CHAR_IS_TOKEN( ')', ENDBRACKET );
    CHAR_IS_TOKEN( '[', STARTBOXBRACKET );
    CHAR_IS_TOKEN( ']', ENDBOXBRACKET );
    CHAR_IS_TOKEN( ',', COMMA );
    CHAR_IS_TOKEN( '=', EQUAL );
    CHAR_IS_TOKEN_OR_TOKEN( ':', ':', COLON, COLONCOLON);
    CHAR_IS_TOKEN_OR_TOKEN( '-', '>', MINUS, RIGHT_ARROW);
    CHAR_IS_TOKEN_OR_TOKEN( '<', '-', UNKNOWN, LEFT_ARROW);
  }
  if( lastChar > 128 ) return next_token();
  identifierStr = lastChar;
  std::cerr << "Unknown token : " << lastChar << " '" << identifierStr << "' at " << initial_line << "," << initial_col << " eof is" << eof() << std::endl;
  assert( not isspace(lastChar));
  return token(token_type::UNKNOWN, initial_line, initial_col);
}

int lexer::get_next_non_separator_char()
{
  int lastChar = ' ';
  while( not eof() and isspace(lastChar = get_next_char() )  )
  { // Ignore space
  }
  return lastChar;
}

int lexer::get_next_char()
{
  d->lastChar = d->stream->get();
  if( d->lastChar == '\n' )
  {
    ++d->line;
    ++d->followingnewline;
    d->col = 1;
  } else {
    ++d->col;
    d->followingnewline = 0;
  }
  return d->lastChar;
}

void lexer::unget()
{
  --d->col;
  d->stream->unget();
  if( d->followingnewline > 0 )
  {
    --d->followingnewline;
    --d->line;
  }
}

bool lexer::eof() const
{
  return !d->stream->good();
}

int lexer::line() const
{
  return d->line;
}
int lexer::column() const
{
  return d->col;
}

bool lexer::ignore_comment(token& _token, int _lastChar )
{
  if( _lastChar == '/' )
  {
    int initial_line = line();
    int initial_col = column();
    int nextChar = get_next_char();
    if( nextChar == '/' )
    { // Mono line comment
      while( not eof() and get_next_char() != '\n' )
      {
      }
      _token = next_token();
      return true;
    } else {
      unget();
    }
  }
  return false;
}

std::string lexer::get_identifier(int lastChar)
{
  std::string identifierStr;
  if( lastChar != 0) {
    identifierStr = lastChar;
  }
  while (not eof() )
  {
    lastChar = get_next_char();
    if( std::isalnum(lastChar) or lastChar == '_')
    {
      identifierStr += lastChar;
    } else {
      unget();
      break;
    }
  }
  return identifierStr;
}

token lexer::get_string(int lastChar)
{
  int initial_col = column();
  int previousChar = lastChar;
  std::string identifierStr = "";
  while( not eof() )
  {
    int nextChar = get_next_char();
    if( nextChar == lastChar and previousChar != '\\')
    {
      return token(token_type::STRING, identifierStr, line(), initial_col );
    } else {
      previousChar = nextChar;
      identifierStr += nextChar;
    }
  }
  return token(token_type::UNFINISHED_STRING, line(), initial_col);
}
