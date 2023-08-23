#include "parser.h"

#include "../logging.h"

#include "lexer.h"

using namespace gqlite::oc;

struct parser::data
{
  lexer* lex;
  token tok;
  std::string error;

  void get_next_token();
  void report_error(const token& _token, const std::string& _errorMsg);
  void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);

};

void parser::data::get_next_token()
{
  tok = lex->next_token();
}

void parser::data::report_error(const token& _token, const std::string& _errorMsg)
{
  gqlite_assert(error.empty());
  error = std::to_string(_token.line) + ":" + std::to_string(_token.column) + ":" + _errorMsg;
}

void parser::data::report_unexpected(const token& _token)
{
  if(_token.string.empty())
  {
    report_error(_token, std::string("Unexpected token ") + token_type_to_string(_token.type));
  } else {
    report_error(_token, std::string("Unexpected token ") + token_type_to_string(_token.type) + " (" + _token.string + ")");
  }
}

bool parser::data::is_of_type(const token& _token, token_type _type)
{
  if(_token.type == _type) return true;
  if(_token.string.empty())
  {
    report_error(_token, std::string("Expected token ") + token_type_to_string(_type) + " got " + token_type_to_string(_token.type));
  } else {
    report_error(_token, std::string("Expected token ") + token_type_to_string(_type) + " got " + token_type_to_string(_token.type) + " (" + _token.string + ")");
  }
  return false;
}

bool parser::data::is_of_type(token_type _type)
{
  return is_of_type(tok, _type);
}

parser::parser(lexer* _lexer) : d(new data)
{
  d->lex = _lexer;
}

parser::~parser()
{
  delete d;
}

algebra::node_csp parser::parse()
{
  d->error = "unimplemented";
  return nullptr;
}

std::string parser::get_error() const
{
  return d->error;
}

