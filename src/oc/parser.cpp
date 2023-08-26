#include "parser.h"

#include "algebra/nodes.h"

#include "../logging.h"

#include "lexer.h"

using namespace gqlite::oc;

struct parser::data
{
  lexer* lex;
  token tok;
  std::string error;

  algebra::node_csp parse_create();
  void get_next_token();
  void report_error(const token& _token, const std::string& _errorMsg);
  void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);

};

#define CHECK_IS_OF_TYPE(...) \
  if(not is_of_type(__VA_ARGS__)) return nullptr

algebra::node_csp parser::data::parse_create()
{
  get_next_token();
  CHECK_IS_OF_TYPE(token_type::STARTBRACKET);
  std::vector<algebra::graph_node_csp> nodes;
  get_next_token();
  CHECK_IS_OF_TYPE(token_type::IDENTIFIER);
  std::string variable = tok.string;
  std::vector<std::string> labels;
  get_next_token();
  while(tok.type == token_type::COLON)
  {
    get_next_token();
    CHECK_IS_OF_TYPE(token_type::IDENTIFIER);
    labels.push_back(tok.string);
    get_next_token();
  }
  CHECK_IS_OF_TYPE(token_type::ENDBRACKET);
  get_next_token();
  std::unordered_map<GQLITE_LIST(std::string, std::any)> properties;
  return std::make_shared<algebra::create_nodes>(std::vector<algebra::graph_node_csp>{std::make_shared<algebra::graph_node>(variable, labels, properties)});
}

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
  d->get_next_token();
  switch(d->tok.type)
  {
    case token_type::CREATE:
      return d->parse_create();
    default:
      d->report_unexpected(d->tok);
      return nullptr;
  }
}

std::string parser::get_error() const
{
  return d->error;
}

