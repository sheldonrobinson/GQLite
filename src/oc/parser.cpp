#include "parser.h"

#include "algebra/nodes.h"

#include "../logging.h"

#include "lexer.h"

using namespace gqlite::oc;

struct parser::data
{
  lexer* lex;
  token tok;

  algebra::node_csp parse_create();
  std::unordered_map<std::string, algebra::node_csp> parse_properties();
  algebra::node_csp parse_expression();
  void get_next_token();
  void report_error(const token& _token, const std::string& _errorMsg);
  void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);

};

algebra::node_csp parser::data::parse_create()
{
  get_next_token();
  is_of_type(token_type::STARTBRACKET);
  std::vector<algebra::graph_node_csp> nodes;
  get_next_token();
  is_of_type(token_type::IDENTIFIER);
  std::string variable = tok.string;
  std::vector<std::string> labels;
  get_next_token();
  while(tok.type == token_type::COLON)
  {
    get_next_token();
    is_of_type(token_type::IDENTIFIER);
    labels.push_back(tok.string);
    get_next_token();
  }
  std::unordered_map<std::string, algebra::node_csp> properties;
  if(tok.type == token_type::STARTBRACE)
  {
    properties = parse_properties();
  }
  is_of_type(token_type::ENDBRACKET);
  get_next_token();
  return std::make_shared<algebra::create_nodes>(std::vector<algebra::graph_node_csp>{std::make_shared<algebra::graph_node>(variable, labels, properties)});
}

std::unordered_map<std::string, algebra::node_csp> parser::data::parse_properties()
{
  std::unordered_map<std::string, algebra::node_csp> p;
  is_of_type(token_type::STARTBRACE);
  get_next_token();
  while(tok.type != token_type::ENDBRACE)
  {
    if(tok.type != token_type::STRING and tok.type != token_type::IDENTIFIER)
    {
      report_unexpected(tok);
    }
    std::string key = tok.string;
    get_next_token();
    is_of_type(token_type::COLON);
    get_next_token();
    p[key] = parse_expression();
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  }
  is_of_type(token_type::ENDBRACE);
  get_next_token();
  return p;
}

algebra::node_csp parser::data::parse_expression()
{
  token t = tok;
  switch (tok.type)
  {
  case token_type::STRING:
    get_next_token();
    return std::make_shared<algebra::value>(t.string);
  default:
    report_unexpected(tok);
  }
}

void parser::data::get_next_token()
{
  tok = lex->next_token();
}

void parser::data::report_error(const token& _token, const std::string& _errorMsg)
{
  throw gqlite::exception(std::to_string(_token.line) + ":" + std::to_string(_token.column) + ":" + _errorMsg);
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

