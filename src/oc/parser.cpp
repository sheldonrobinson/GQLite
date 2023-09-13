#include "parser.h"

#include "algebra/nodes.h"

#include "../gqlite_p.h"
#include "../logging.h"
#include "../string.h"

#include "lexer.h"

using namespace gqlite::oc;

struct parser::data
{
  lexer* lex;
  token tok;

  algebra::node_csp parse_create();
  algebra::node_csp parse_match();
  algebra::node_csp parse_return();
  algebra::node_csp parse_with();
  algebra::node_csp parse_delete();
  algebra::node_csp parse_set();
  std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>> parse_patterns(bool _allow_undirected_edge, bool _allow_multiple_edge_labels, bool _require_one_label);
  std::unordered_map<std::string, algebra::node_csp> parse_properties();
  algebra::node_csp parse_expression();
  algebra::node_csp parse_unary_expression();
  algebra::node_csp parse_multiplicative_expression();
  algebra::node_csp parse_additive_expression();
  algebra::node_csp parse_relational_expression();
  algebra::node_csp parse_conditional_and_expression();
  algebra::node_csp parse_member_expression();
  algebra::node_csp parse_terminal_expression();
  algebra::node_csp parse_expression_list();
  std::vector<std::string> parse_path();
  std::vector<algebra::named_expression_csp> parse_named_expressions();
  void validate(algebra::graph_node_csp);
  void validate(algebra::graph_edge_csp);
  void get_next_token();
  template<typename... _T_>
  [[noreturn]] void report_error(const token& _token, const std::string& _errorMsg, const _T_&... _values);
  [[noreturn]] void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);

  std::unordered_map<std::string, algebra::node_csp> bounded_variables;
};

namespace gqlite::details
{
  template<>
  inline std::string to_string<token_type>(const token_type& _v)
  {
    return token_type_to_string(_v);
  }
}

void parser::data::validate(algebra::graph_node_csp _node)
{
  if(_node->get_variable().empty()) return;
  auto it = bounded_variables.find(_node->get_variable());
  if(it == bounded_variables.end())
  {
    bounded_variables[_node->get_variable()] = _node;
    return;
  }
  if(_node->get_labels().empty() and not _node->get_properties()) return;
  if(_node->equals(it->second)) return;
  report_error(tok, "Variable {} is already bound.", _node->get_variable());
}

void parser::data::validate(algebra::graph_edge_csp _node)
{
  validate(_node->get_source());
  validate(_node->get_destination());
  if(_node->get_variable().empty()) return;
  auto it = bounded_variables.find(_node->get_variable());
  if(it == bounded_variables.end())
  {
    bounded_variables[_node->get_variable()] = _node;
    return;
  }
  if(_node->get_labels().empty() and not _node->get_properties()) return;
  if(_node->equals(it->second)) return;
  report_error(tok, "Variable {} is already bound.", _node->get_variable());
}

algebra::node_csp parser::data::parse_create()
{
  get_next_token();
  return std::make_shared<algebra::create>(parse_patterns(false, false, true));
}

algebra::node_csp parser::data::parse_match()
{
  get_next_token();
  std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>> patterns = parse_patterns(true, true, false);
  algebra::node_csp where;
  if(tok.type == token_type::WHERE)
  {
    get_next_token();
    where = parse_expression();
  }
  return std::make_shared<algebra::match>(patterns, where);
}

std::vector<algebra::named_expression_csp> parser::data::parse_named_expressions()
{
  std::vector<algebra::named_expression_csp> expressions;
  std::vector<std::string> labels;
  do
  {
    std::string name;
    if(tok.type == token_type::IDENTIFIER)
    {
      name = tok.string;
    }
    algebra::node_csp node = parse_expression();
    if(tok.type == token_type::AS)
    {
      get_next_token();
      is_of_type(token_type::IDENTIFIER);
      name = tok.string;
      get_next_token();
    } else if(node->get_type() != algebra::node_type::variable)
    {
      if(node->get_type() == algebra::node_type::member_access)
      {
        algebra::member_access_csp ma = std::static_pointer_cast<const algebra::member_access>(node);
        name = name + "." + string::join(ma->get_path(), ".");
      } else {
        is_of_type(token_type::AS);
      }
    }
    if(std::find(labels.begin(), labels.end(), name) != labels.end())
    {
      report_error(tok, "Duplicate column name {} is not allowed.");
    }
    labels.push_back(name);
    expressions.push_back(std::make_shared<algebra::named_expression>(name, node));
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  } while(true);
  return expressions;
}

algebra::node_csp parser::data::parse_with()
{
  get_next_token();
  if(tok.type == token_type::STAR)
  {
    get_next_token();
    return std::make_shared<algebra::with>(true, std::vector<algebra::named_expression_csp>());
  }
  return std::make_shared<algebra::with>(false, parse_named_expressions());
}

algebra::node_csp parser::data::parse_delete()
{
  bool detach = (tok.type == token_type::DETACH);
  if(detach) get_next_token();
  is_of_type(token_type::DELETE);
  get_next_token();
  std::vector<algebra::node_csp> expressions;
  while(tok.type != token_type::END_OF_FILE)
  {
    expressions.push_back(parse_expression());
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else
    {
      break;
    }
  }
  return std::make_shared<algebra::delete_statement>(detach, expressions);
}

algebra::node_csp parser::data::parse_set()
{
  std::vector<std::pair<algebra::member_access_csp, algebra::node_csp>> expressions;
  get_next_token();
  while(tok.type != token_type::END_OF_FILE)
  {
    std::string name;
    if(tok.type == token_type::STARTBRACKET)
    {
      get_next_token();
      is_of_type(token_type::IDENTIFIER);
      name = tok.string;
      get_next_token();
      is_of_type(token_type::ENDBRACKET);
    } else {
      is_of_type(token_type::IDENTIFIER);
      name = tok.string;
    }
    get_next_token();
    std::vector<std::string> path = parse_path();
    is_of_type(token_type::EQUAL);
    get_next_token();
    algebra::node_csp value = parse_expression();
    expressions.push_back({std::make_shared<algebra::member_access>(name, path), value});
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  }
  return std::make_shared<algebra::set>(expressions);
}

algebra::node_csp parser::data::parse_return()
{
  get_next_token();
  if(tok.type == token_type::STAR)
  {
    get_next_token();
    return std::make_shared<algebra::return_statement>(true, std::vector<algebra::named_expression_csp>());
  }
  return std::make_shared<algebra::return_statement>(false, parse_named_expressions());
}

namespace
{
  struct edge
  {
    bool active = false;
    algebra::edge_directivity directivity;
    algebra::graph_node_csp source, destination;
    std::string variable;
    std::vector<std::string> labels;
    algebra::map_csp properties;
  };
}

std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>> parser::data::parse_patterns(bool _allow_undirected_edge, bool _allow_multiple_edge_labels, bool _require_one_label)
{
  std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>> patterns;
  edge current_edge;
  do
  {
    is_of_type(token_type::STARTBRACKET);
    std::vector<algebra::graph_node_csp> nodes;
    get_next_token();
    // identifier
    std::string variable;
    if(tok.type == token_type::IDENTIFIER)
    { 
      variable = tok.string;
      get_next_token();
    }
    // Labels
    std::vector<std::string> labels;
    while(tok.type == token_type::COLON)
    {
      get_next_token();
      is_of_type(token_type::IDENTIFIER);
      labels.push_back(tok.string);
      get_next_token();
    }
    // Properties
    algebra::map_csp properties;
    if(tok.type == token_type::STARTBRACE)
    {
      properties = std::make_shared<algebra::map>(parse_properties());
    }
    // End of node
    is_of_type(token_type::ENDBRACKET);
    get_next_token();
    algebra::graph_node_csp gnode = std::make_shared<algebra::graph_node>(variable, labels, properties);
    // Handle, check if edge is active
    bool add_to_patterns = true;
    if(current_edge.active)
    {
      if(current_edge.source)
      {
        current_edge.destination = gnode;
      } else {
        current_edge.source = gnode;
      }
      algebra::graph_edge_csp ge = std::make_shared<algebra::graph_edge>(current_edge.variable, current_edge.source, current_edge.destination, current_edge.directivity, current_edge.labels, current_edge.properties);
      validate(ge);
      patterns.push_back(ge);
      current_edge.active = false;
      add_to_patterns = false;
    }
    // If comma, an other node statement comes after
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
      if(tok.type == token_type::MINUS)
      {
        current_edge.source = gnode;
        current_edge.directivity = algebra::edge_directivity::undirected;
      } else {
        current_edge.destination = gnode;
        current_edge.directivity = algebra::edge_directivity::directed;
      }
      // Parse
      get_next_token();
      if(tok.type == token_type::RIGHT_ARROW or tok.type == token_type::MINUS)
      {
        if(_require_one_label)
        {
          is_of_type(token_type::STARTBOXBRACKET);
        }
      } else {
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
          current_edge.labels.push_back(tok.string);
          get_next_token();
          if(_allow_multiple_edge_labels)
          {
            while(tok.type != token_type::END_OF_FILE)
            {
              if(tok.type == token_type::PIPE)
              {
                get_next_token();
                if(tok.type == token_type::COLON)
                {
                  get_next_token();
                }
                is_of_type(token_type::IDENTIFIER);
                if(std::find(current_edge.labels.begin(), current_edge.labels.end(), tok.string) == current_edge.labels.end())
                {
                  current_edge.labels.push_back(tok.string);
                }
                get_next_token();
              } else {
                break;
              }
            }
          }
        } else if(_require_one_label) {
          is_of_type(token_type::COLON); //
        }
        if(tok.type == token_type::STARTBRACE)
        {
          current_edge.properties = std::make_shared<algebra::map>(parse_properties());
        }
        is_of_type(token_type::ENDBOXBRACKET);
        get_next_token();
      }
      if(tok.type == token_type::MINUS)
      {
        if(current_edge.directivity == algebra::edge_directivity::undirected)
        {
          if(not _allow_undirected_edge)
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
        current_edge.directivity = algebra::edge_directivity::directed;
      } else {
        report_unexpected(tok);
      }
      get_next_token();
      is_of_type(token_type::STARTBRACKET);
      add_to_patterns = false;
    } else {
      if(add_to_patterns) { validate(gnode); patterns.push_back(gnode); }
      break;
    }
    if(add_to_patterns) { validate(gnode); patterns.push_back(gnode); }
  } while(true);
  if(current_edge.active)
  {
    report_error(tok, "Unfinished edge");
  }
  return patterns;
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

algebra::node_csp parser::data::parse_terminal_expression()
{
  token t = tok;
  switch (tok.type)
  {
  case token_type::NULL_TOKEN:
    get_next_token();
    return std::make_shared<algebra::value>(gqlite::value());
  case token_type::IDENTIFIER:
    get_next_token();
    switch(tok.type)
    {
      case token_type::STARTBRACKET:
      {
        std::vector<algebra::node_csp> arguments;
        get_next_token();
        if(tok.type != token_type::ENDBRACKET)
        {
          while(tok.type != token_type::END_OF_FILE)
          {
            arguments.push_back(parse_expression());
            if(tok.type == token_type::COMMA)
            {
              get_next_token();
            } else {
              break;
            }
          }
          is_of_type(token_type::ENDBRACKET);
        }
        get_next_token();
        return std::make_shared<algebra::function_call>(t.string, arguments);
      }
      case token_type::COLON:
      {
        std::vector<std::string> labels;
        get_next_token();
        while(tok.type == token_type::IDENTIFIER)
        {
          labels.push_back(tok.string);
          get_next_token();
          if(tok.type == token_type::COLON)
          {
            get_next_token();
          } else {
            break;
          }
        }
        return std::make_shared<algebra::has_labels>(t.string, labels);
      }
      default:
        return std::make_shared<algebra::variable>(t.string);
    }
  case token_type::STRING:
    get_next_token();
    return std::make_shared<algebra::value>(t.string);
  case token_type::INTEGER:
    get_next_token();
    try
    {
      return std::make_shared<algebra::value>(std::stoi(t.string));
    } catch(std::out_of_range)
    {
      throw gqlite::exception("Integer {} is too large.", t.string);
    }
  case token_type::FLOATING_POINT:
    get_next_token();
    return std::make_shared<algebra::value>(std::stod(t.string));
  case token_type::TRUE:
    get_next_token();
    return std::make_shared<algebra::value>(true);
  case token_type::FALSE:
    get_next_token();
    return std::make_shared<algebra::value>(false);
  case token_type::STARTBRACE:
  {
    return std::make_shared<algebra::map>(parse_properties());
  }
  case token_type::STARTBOXBRACKET:
  {
    return parse_expression_list();
  }
  default:
    report_unexpected(tok);
  }
}

algebra::node_csp parser::data::parse_expression_list()
{
  std::vector<algebra::node_csp> nodes;
  get_next_token();
  while(tok.type != token_type::END_OF_FILE)
  {
    nodes.push_back(parse_expression());
    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      is_of_type(token_type::ENDBOXBRACKET);
      get_next_token();
      return std::make_shared<algebra::array>(nodes);
    }
  }
  report_unexpected(tok);
}

algebra::node_csp parser::data::parse_conditional_and_expression()
{
  algebra::node_csp node = parse_relational_expression();
  if(tok.type == token_type::AND)
  {
    get_next_token();
    return std::make_shared<algebra::logical_and>(node, parse_expression());
  } else {
    return node;
  }
}

algebra::node_csp parser::data::parse_relational_expression()
{
  algebra::node_csp node = parse_additive_expression();
  switch(tok.type)
  {
    case token_type::EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_equal>(node, parse_additive_expression());
    case token_type::DIFFERENT:
      get_next_token();
      return std::make_shared<algebra::relational_different>(node, parse_additive_expression());
    case token_type::INFERIOR:
      get_next_token();
      return std::make_shared<algebra::relational_inferior>(node, parse_additive_expression());
    case token_type::SUPERIOR:
      get_next_token();
      return std::make_shared<algebra::relational_superior>(node, parse_additive_expression());
    case token_type::INFERIOR_EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_inferior_equal>(node, parse_additive_expression());
    case token_type::SUPERIOR_EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_superior_equal>(node, parse_additive_expression());
    case token_type::IN:
      get_next_token();
      return std::make_shared<algebra::relational_in>(node, parse_expression_list());
    case token_type::NOT:
      get_next_token();
      if(is_of_type(tok, token_type::IN))
      {
        get_next_token();
      }
      return std::make_shared<algebra::relational_not_in>(node, parse_expression_list());
    default:
      return node;
  }
}

algebra::node_csp parser::data::parse_additive_expression()
{
  algebra::node_csp node = parse_multiplicative_expression();
  switch(tok.type)
  {
    case token_type::PLUS:
      get_next_token();
      return std::make_shared<algebra::addition>(node, parse_multiplicative_expression());
    case token_type::MINUS:
      get_next_token();
      return std::make_shared<algebra::substraction>(node, parse_multiplicative_expression());
    default:
      return node;
  }
}

algebra::node_csp parser::data::parse_multiplicative_expression()
{
  algebra::node_csp node = parse_unary_expression();
  switch(tok.type)
  {
    case token_type::STAR:
      get_next_token();
      return std::make_shared<algebra::multiplication>(node, parse_unary_expression());
    case token_type::DIVIDE:
      get_next_token();
      return std::make_shared<algebra::division>(node, parse_unary_expression());
    default:
      return node;
  }
}

algebra::node_csp parser::data::parse_unary_expression()
{
  switch(tok.type)
  {
    case token_type::EXCLAMATION:
      get_next_token();
      return std::make_shared<algebra::logical_negation>(parse_member_expression());
    case token_type::MINUS:
      get_next_token();
      return std::make_shared<algebra::negation>(parse_member_expression());
    case token_type::PLUS:
      get_next_token();
      [[fallthrough]];
    default:
      return parse_member_expression();
  }
}

std::vector<std::string> parser::data::parse_path()
{
  std::vector<std::string> path;
  while (tok.type == token_type::DOT)
  {
    get_next_token();
    is_of_type(token_type::IDENTIFIER);
    path.push_back(tok.string);
    get_next_token();
  }
  return path;
}

algebra::node_csp parser::data::parse_member_expression()
{
  algebra::node_csp left = parse_terminal_expression();
  if(tok.type == token_type::DOT)
  {
    if(left->get_type() != algebra::node_type::variable)
    {
      report_unexpected(tok);
    }
    std::string left_identif = std::static_pointer_cast<const algebra::variable>(left)->get_identifier();
    return std::make_shared<algebra::member_access>(left_identif, parse_path());
  }
  return left;
}

algebra::node_csp parser::data::parse_expression()
{
  algebra::node_csp node = parse_conditional_and_expression();
  if(tok.type == token_type::OR)
  {
    get_next_token();
    return std::make_shared<algebra::logical_or>(node, parse_expression());
  } else {
    return node;
  }
}

void parser::data::get_next_token()
{
  tok = lex->next_token();
}

template<typename... _T_>
void parser::data::report_error(const token& _token, const std::string& _errorMsg, const _T_&... _values)
{
  throw gqlite::exception("{}:{}:" + _errorMsg, _token.line, _token.column, _values...);
}

void parser::data::report_unexpected(const token& _token) 
{
  if(_token.string.empty())
  {
    report_error(_token, "Unexpected token {}", _token.type);
  } else {
    report_error(_token, "Unexpected token {} ({})", _token.type, _token.string);
  }
}

bool parser::data::is_of_type(const token& _token, token_type _type)
{
  if(_token.type == _type) return true;
  if(_token.string.empty())
  {
    report_error(_token, "Expected token {} got {}", _type, _token.type);
  } else {
    report_error(_token, "Expected token {} got {} ({})", _type, _token.type, _token.string);
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
    case token_type::MATCH:
      nodes.push_back(d->parse_match());
      break;
    case token_type::WITH:
      nodes.push_back(d->parse_with());
      break;
    case token_type::DELETE:
    case token_type::DETACH:
      nodes.push_back(d->parse_delete());
      break;
    case token_type::SET:
      nodes.push_back(d->parse_set());
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

