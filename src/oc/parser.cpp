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
  algebra::node_csp parse_matches();
  algebra::node_csp parse_return();
  std::tuple<std::vector<algebra::graph_node_csp>, std::vector<algebra::graph_edge_csp>> parse_graph_nodes_edges(bool _creation, bool _match);
  std::unordered_map<std::string, algebra::node_csp> parse_properties();
  algebra::node_csp parse_expression();
  void get_next_token();
  [[noreturn]] void report_error(const token& _token, const std::string& _errorMsg);
  [[noreturn]] void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);

};

algebra::node_csp parser::data::parse_create()
{
  get_next_token();
  auto const&[nodes, edges] = parse_graph_nodes_edges(true, false);
  return std::make_shared<algebra::create>(nodes, edges);
}

algebra::node_csp parser::data::parse_matches()
{
  get_next_token();
  auto const&[nodes, edges] = parse_graph_nodes_edges(false, true);
  return std::make_shared<algebra::match>(nodes, edges);
}

algebra::node_csp parser::data::parse_return()
{
  get_next_token();
  std::vector<std::string> variables;
  do
  {
    is_of_type(token_type::IDENTIFIER);
    variables.push_back(tok.string);
    get_next_token();
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  } while(true);
  return std::make_shared<algebra::return_statement>(variables);
}

namespace
{
  struct edge
  {
    bool active = false;
    algebra::edge_directivity directivity;
    algebra::graph_node_csp source, destination;
    std::string variable;
    std::string label;
    std::unordered_map<std::string, algebra::node_csp> properties;
  };
}

std::tuple<std::vector<algebra::graph_node_csp>, std::vector<algebra::graph_edge_csp>> parser::data::parse_graph_nodes_edges(bool _creation, bool _match)
{
  std::vector<algebra::graph_node_csp> graph_nodes;
  std::vector<algebra::graph_edge_csp> graph_edges;
  edge current_edge;
  do
  {
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
    algebra::graph_node_csp gnode = std::make_shared<algebra::graph_node>(variable, labels, properties);
    graph_nodes.push_back(gnode);
    if(current_edge.active)
    {
      if(current_edge.source)
      {
        current_edge.destination = gnode;
      } else {
        current_edge.source = gnode;
      }
      // graph_edges.push_back(std::make_shared<algebra::graph_edge>(current_edge.source, current_edge.destination, current_edge.directivity, current_edge.label, current_edge.properties));
      current_edge.active = false;
    }
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    }
    else if(tok.type == token_type::MINUS or tok.type == token_type::LEFT_ARROW)
    {
      // Reset current edge
      current_edge = edge{};
      current_edge.active = true;
      // Handle directivity
      bool node_swaped = false;
      if(tok.type == token_type::MINUS)
      {
        current_edge.directivity = algebra::edge_directivity::undirected;
      } else {
        node_swaped = true;
        current_edge.directivity = algebra::edge_directivity::directed;
      }
      // Parse
      get_next_token();
      is_of_type(token_type::STARTBOXBRACKET);
      get_next_token();
      // parse edge
      if(tok.type == token_type::IDENTIFIER)
      { // identifier
        current_edge.variable = tok.string;
        get_next_token();
      }
      if(tok.type == token_type::COLON)
      { // label
        get_next_token();
        is_of_type(token_type::IDENTIFIER);
        current_edge.label = tok.string;
        get_next_token();
      }
      if(tok.type == token_type::STARTBRACE)
      {
        current_edge.properties = parse_properties();
      }
      is_of_type(token_type::ENDBOXBRACKET);
      get_next_token();
      if(tok.type == token_type::MINUS)
      {
        if(current_edge.directivity == algebra::edge_directivity::undirected)
        {
          if(_creation)
          {
            report_error(tok, "Edge must be directed during creation,");
          }
        }
      } else if(tok.type == token_type::RIGHT_ARROW)
      {
        if(current_edge.directivity == algebra::edge_directivity::directed)
        {
          report_error(tok, "Edge cannot have both direction.");
        }
      } else {
        report_unexpected(tok);
      }
      get_next_token();
      is_of_type(token_type::STARTBRACKET);
    } else {
      break;
    }
  } while(true);
  if(current_edge.active)
  {
    report_error(tok, "Unfinished edge");
  }
  return {graph_nodes, graph_edges};
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
  std::vector<algebra::node_csp> nodes;
  d->get_next_token();
  while(d->tok.type != token_type::END_OF_FILE)
  {
    switch(d->tok.type)
    {
    case token_type::CREATE:
      nodes.push_back(d->parse_create());
      break;
    case token_type::MATCHES:
      nodes.push_back(d->parse_matches());
      break;
    case token_type::RETURN:
      nodes.push_back(d->parse_return());
      d->is_of_type(token_type::END_OF_FILE);
      break;
    default:
      d->report_unexpected(d->tok);
    }
  }
  switch(nodes.size())
  {
  case 0:
    d->report_error(token(), "Empty query.");
  case 1:
    return nodes.front();
  default:
    return std::make_shared<algebra::statements>(nodes);
  }
}

