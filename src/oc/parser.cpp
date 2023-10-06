#include "parser.h"

#include "algebra/nodes.h"
#include "algebra/visitors/stringify.h"

#include "../gqlite_p.h"
#include "../logging.h"


#include "lexer.h"

using namespace gqlite::oc;

struct parser::data
{
  lexer* lex;
  value_map bindings;
  token tok;
  int id = 0;
  algebra::node_csp parse_call();
  algebra::node_csp parse_create();
  algebra::node_csp parse_match();
  algebra::node_csp parse_return();
  algebra::node_csp parse_with();
  algebra::node_csp parse_unwind();
  algebra::node_csp parse_delete();
  algebra::node_csp parse_set();
  algebra::node_csp parse_remove();
  algebra::modifiers_csp parse_modifiers();
  std::vector<algebra::alternative<algebra::graph_node, algebra::graph_edge>> parse_patterns(bool _allow_undirected_edge, bool _allow_multiple_edge_labels, bool _require_one_label);
  std::unordered_map<std::string, algebra::node_csp> parse_properties();
  algebra::node_csp parse_expression();
  algebra::node_csp parse_unary_expression();
  algebra::node_csp parse_multiplicative_expression();
  algebra::node_csp parse_additive_expression();
  algebra::node_csp parse_relational_expression();
  algebra::node_csp parse_conditional_xor_expression();
  algebra::node_csp parse_conditional_or_expression();
  algebra::node_csp parse_conditional_and_expression();
  algebra::node_csp parse_indexed_access_expression();
  algebra::node_csp parse_member_expression();
  algebra::node_csp parse_terminal_expression();
  algebra::node_csp parse_expression_list();
  std::vector<std::string> parse_path();
  std::vector<algebra::named_expression_csp> parse_named_expressions();
  void validate(algebra::graph_node_csp);
  void validate(algebra::graph_edge_csp);
  std::string generate_anonymous_variable();
  std::unordered_map<std::string, algebra::node_csp> bounded_variables;
  void get_next_token(int _flags = lexer::mode::normal);
  template<typename... _T_>
  [[noreturn]] void report_error(const token& _token, exception_code _code, const std::string& _errorMsg, const _T_&... _values);
  [[noreturn]] void report_unexpected(const token& _token);
  bool is_of_type(const token& _token, token_type _type);
  bool is_of_type(token_type _type);
  int64_t string_to_integer(std::string _string);

};

namespace gqlite
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
  report_error(tok, exception_code::variable_already_bound, "Variable {} is already bound", _node->get_variable());
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
  report_error(tok, exception_code::variable_already_bound, "Variable {} is already bound.", _node->get_variable());
}

std::string parser::data::generate_anonymous_variable()
{
  return "__gqlite_anon_" + std::to_string(id++);
}

int64_t parser::data::string_to_integer(std::string _string)
{
  if(_string == "0x" or _string == "-0x" or _string == "0o" or _string == "-0o")
  {
    throw_exception(exception_stage::compiletime, exception_code::invalid_number_literal, "'{}' is not a valid string literal", _string);
  }
  try
  {
    int base = 10;
    if(_string.size() > 2 and (_string[1] == 'x' or (_string[0] == '-' and _string[2] == 'x')))
    {
      base = 16;
    } else if(_string.size() > 2 and (_string[1] == 'o' or (_string[0] == '-' and _string[2] == 'o')))
    {
      base = 8;
      if(_string[0] == '-') _string = '-' + _string.substr(3);
      else _string = _string.substr(2);
    }
    long long i = std::stoll(_string, 0, base);
    if(i > std::numeric_limits<int64_t>::max() or i < std::numeric_limits<int64_t>::min())
    {
      throw_exception(exception_stage::compiletime, exception_code::integer_overflow, "'{}' is too large.", _string);
    }
    return i;
  } catch(const std::out_of_range&)
  {
    throw_exception(exception_stage::compiletime, exception_code::integer_overflow, "'{}' is too large.", _string);
  }
}

algebra::node_csp parser::data::parse_call()
{
  get_next_token();
  std::string name;
  while(tok.type != token_type::END_OF_FILE)
  {
    is_of_type(token_type::IDENTIFIER);
    name += tok.string;
    get_next_token();
    if(tok.type == token_type::DOT)
    {
      name += ".";
      get_next_token();
    } else {
      break;
    }
  }
  std::vector<algebra::node_csp> arguments;
  is_of_type(token_type::STARTBRACKET);
  get_next_token();
  while(tok.type != token_type::ENDBRACKET)
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
  get_next_token();
  std::vector<std::string> yields;
  if(tok.type == token_type::YIELD)
  {
    get_next_token();
    while(tok.type != token_type::END_OF_FILE)
    {
      is_of_type(token_type::IDENTIFIER);
      yields.push_back(tok.string);
      if(tok.type == token_type::COMMA)
      {
        get_next_token();
      } else {
        break;
      }
    }
  }
  return std::make_shared<algebra::call>(name, arguments, yields);
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
      name = algebra::visitors::stringify().start(node);
    }
    if(std::find(labels.begin(), labels.end(), name) != labels.end())
    {
      report_error(tok, exception_code::column_name_conflict, "Duplicate column name {} is not allowed", name);
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
    return std::make_shared<algebra::with>(true, std::vector<algebra::named_expression_csp>(), parse_modifiers());
  }
  std::vector<algebra::named_expression_csp> named_expressions = parse_named_expressions();
  return std::make_shared<algebra::with>(false, named_expressions, parse_modifiers());
}

algebra::node_csp parser::data::parse_unwind()
{
  get_next_token();
  algebra::node_csp expr = parse_expression();
  is_of_type(token_type::AS);
  get_next_token();
  is_of_type(token_type::IDENTIFIER);
  std::string name = tok.string;
  get_next_token();
  return std::make_shared<algebra::unwind>(name, expr);
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
  std::vector<algebra::node_csp> nodes;
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

    switch(tok.type)
    {
      case token_type::EQUAL:
      {
        get_next_token();
        algebra::node_csp value = parse_expression();
        nodes.push_back(std::make_shared<algebra::set_property>(std::make_shared<algebra::member_access>(name, path), value));
        break;
      }
      case token_type::PLUS_EQUAL:
      {
        get_next_token();
        algebra::node_csp value = parse_expression();
        nodes.push_back(std::make_shared<algebra::add_property>(std::make_shared<algebra::member_access>(name, path), value));
        break;
      }
      case token_type::COLON:
      {
        if(not path.empty())
        {
          report_unexpected(tok);
        }
        std::vector<std::string> labels;
        while(tok.type == token_type::COLON)
        {
          get_next_token();
          is_of_type(token_type::IDENTIFIER);
          labels.push_back(tok.string);
          get_next_token();
        }
        nodes.push_back(std::make_shared<algebra::edit_labels>(name, labels));
        break;
      }
      default:
        report_unexpected(tok);
    }

    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  }
  return std::make_shared<algebra::set>(nodes);
}

algebra::node_csp parser::data::parse_remove()
{
  std::vector<algebra::node_csp> nodes;
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

    switch(tok.type)
    {
      case token_type::COLON:
      {
        if(not path.empty())
        {
          report_unexpected(tok);
        }
        std::vector<std::string> labels;
        while(tok.type == token_type::COLON)
        {
          get_next_token();
          is_of_type(token_type::IDENTIFIER);
          labels.push_back(tok.string);
          get_next_token();
        }
        nodes.push_back(std::make_shared<algebra::edit_labels>(name, labels));
        break;
      }
      default:
        nodes.push_back(std::make_shared<algebra::remove_property>(std::make_shared<algebra::member_access>(name, path)));
        break;
    }

    if(tok.type == token_type::COMMA)
    {
      get_next_token();
    } else {
      break;
    }
  }
  return std::make_shared<algebra::remove>(nodes);
}

algebra::node_csp parser::data::parse_return()
{
  get_next_token();
  if(tok.type == token_type::STAR)
  {
    get_next_token();
    return std::make_shared<algebra::return_statement>(true, std::vector<algebra::named_expression_csp>(), parse_modifiers());
  }
  std::vector<algebra::named_expression_csp> named_expressions = parse_named_expressions();
  return std::make_shared<algebra::return_statement>(false, named_expressions, parse_modifiers());
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

algebra::modifiers_csp parser::data::parse_modifiers()
{
  algebra::node_csp skip, limit;
  algebra::order_by_csp order_by;

  while(tok.type == token_type::SKIP or tok.type == token_type::LIMIT or tok.type == token_type::ORDER)
  {
    switch(tok.type)
    {
      case token_type::SKIP:
        if(skip) report_unexpected(tok);
        get_next_token();
        skip = parse_expression();
        break;
      case token_type::LIMIT:
        if(limit) report_unexpected(tok);
        get_next_token();
        limit = parse_expression();
        break;
      case token_type::ORDER:
      {
        if(order_by) report_unexpected(tok);
        get_next_token();
        is_of_type(token_type::BY);
        get_next_token();
        std::vector<algebra::order_by_expression_csp> expressions;
        while(true)
        {
          bool asc = true;
          algebra::node_csp expression = parse_expression();
          if(tok.type == token_type::ASC)
          {
            get_next_token();
          } else if(tok.type == token_type::DESC)
          {
            asc = false;
            get_next_token();
          }
          expressions.push_back(std::make_shared<algebra::order_by_expression>(asc, expression));
          if(tok.type == token_type::COMMA)
          {
            get_next_token();
          } else {
            break;
          }
        }
        order_by = std::make_shared<algebra::order_by>(expressions);
        break;
      }
      default:
        report_unexpected(tok);
    }
  }

  if(skip or limit or order_by)
  {
    return std::make_shared<algebra::modifiers>(skip, limit, order_by);
  } else {
    return nullptr;
  }
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
    } else {
      variable = generate_anonymous_variable();
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
    if(tok.type == token_type::STARTBRACE or tok.type == token_type::PARAMETER)
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
        } else {
          current_edge.variable = generate_anonymous_variable();
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
        if(tok.type == token_type::STARTBRACE or tok.type == token_type::PARAMETER)
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
            report_error(tok, exception_code::requires_directed_relationship, "Edge must be directed during creation");
          }
        }
      } else if(tok.type == token_type::RIGHT_ARROW)
      {
        if(current_edge.directivity == algebra::edge_directivity::directed)
        {
          report_error(tok, exception_code::requires_directed_relationship, "Edge cannot have both direction");
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
    report_error(tok, exception_code::parse_error, "Unfinished edge");
  }
  return patterns;
}

std::unordered_map<std::string, algebra::node_csp> parser::data::parse_properties()
{
  switch(tok.type)
  {
    case token_type::STARTBRACE:
    {
      std::unordered_map<std::string, algebra::node_csp> p;
      get_next_token(lexer::mode::disable_keywords);
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
          get_next_token(lexer::mode::disable_keywords);
        } else {
          break;
        }
      }
      is_of_type(token_type::ENDBRACE);
      get_next_token();
      return p;
    }
    case token_type::PARAMETER:
    {
      auto it = bindings.find(tok.string);
      if(it == bindings.end())
      {
        report_error(tok, exception_code::invalid_parameter_use, "Unknown parameter '{}'", tok.string);
      }
      get_next_token();
      std::unordered_map<std::string, algebra::node_csp> p;
      
      for(auto const& [k, v] : it->second.to_map())
      {
        p[k] = std::make_shared<algebra::value>(v);
      }
      return p;
    }
    default:
      report_unexpected(tok);
  }
}

algebra::node_csp parser::data::parse_terminal_expression()
{
  token t = tok;
  switch (tok.type)
  {
  case token_type::STARTBRACKET:
  {
    get_next_token();
    algebra::node_csp no = parse_expression();
    is_of_type(token_type::ENDBRACKET);
    get_next_token();
    return no;
  }
  case token_type::NULL_CAPS:
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
    return std::make_shared<algebra::value>(string_to_integer(t.string));
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
  case token_type::PARAMETER:
  {
    auto it = bindings.find(tok.string);
    if(it == bindings.end())
    {
      report_error(tok, exception_code::invalid_parameter_use, "Unknown parameter '{}'", tok.string);
    }
    get_next_token();
    return std::make_shared<algebra::value>(it->second);
  }
  default:
    report_unexpected(tok);
  }
}

algebra::node_csp parser::data::parse_expression_list()
{
  std::vector<algebra::node_csp> nodes;
  get_next_token();
  if(tok.type == token_type::ENDBOXBRACKET)
  {
    get_next_token();
    // empty list
    return std::make_shared<algebra::array>(nodes);
  }
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

algebra::node_csp parser::data::parse_conditional_xor_expression()
{
  algebra::node_csp node = parse_conditional_or_expression();
  if(tok.type == token_type::XOR)
  {
    get_next_token();
    return std::make_shared<algebra::logical_xor>(node, parse_conditional_xor_expression());
  } else {
    return node;
  }
}

algebra::node_csp parser::data::parse_conditional_or_expression()
{
  algebra::node_csp node = parse_conditional_and_expression();
  if(tok.type == token_type::OR)
  {
    get_next_token();
    return std::make_shared<algebra::logical_or>(node, parse_conditional_or_expression());
  } else {
    return node;
  }
}

algebra::node_csp parser::data::parse_conditional_and_expression()
{
  algebra::node_csp node = parse_relational_expression();
  if(tok.type == token_type::AND)
  {
    get_next_token();
    return std::make_shared<algebra::logical_and>(node, parse_conditional_and_expression());
  } else {
    return node;
  }
}

algebra::node_csp parser::data::parse_relational_expression()
{
  algebra::node_csp node = parse_additive_expression();
  switch(tok.type)
  {
    case token_type::IS:
    {
      get_next_token();
      if(tok.type == token_type::NOT)
      {
        get_next_token();
        is_of_type(token_type::NULL_CAPS);
        get_next_token();
        return std::make_shared<algebra::is_not_null>(node);
      } else {
        is_of_type(token_type::NULL_CAPS);
        get_next_token();
        return std::make_shared<algebra::is_null>(node);
      }
    }
    case token_type::EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_equal>(node, parse_relational_expression());
    case token_type::DIFFERENT:
      get_next_token();
      return std::make_shared<algebra::relational_different>(node, parse_relational_expression());
    case token_type::INFERIOR:
      get_next_token();
      return std::make_shared<algebra::relational_inferior>(node, parse_relational_expression());
    case token_type::SUPERIOR:
      get_next_token();
      return std::make_shared<algebra::relational_superior>(node, parse_relational_expression());
    case token_type::INFERIOR_EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_inferior_equal>(node, parse_relational_expression());
    case token_type::SUPERIOR_EQUAL:
      get_next_token();
      return std::make_shared<algebra::relational_superior_equal>(node, parse_relational_expression());
    case token_type::IN:
      get_next_token();
      return std::make_shared<algebra::relational_in>(node, parse_expression());
    case token_type::NOT:
      get_next_token();
      if(is_of_type(tok, token_type::IN))
      {
        get_next_token();
      }
      return std::make_shared<algebra::relational_not_in>(node, parse_expression());
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
      return std::make_shared<algebra::addition>(node, parse_additive_expression());
    case token_type::MINUS:
      get_next_token();
      return std::make_shared<algebra::substraction>(node, parse_additive_expression());
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
      return std::make_shared<algebra::multiplication>(node, parse_multiplicative_expression());
    case token_type::DIVIDE:
      get_next_token();
      return std::make_shared<algebra::division>(node, parse_multiplicative_expression());
    case token_type::PERCENT:
      get_next_token();
      return std::make_shared<algebra::modulo>(node, parse_multiplicative_expression());
    default:
      return node;
  }
}

algebra::node_csp parser::data::parse_unary_expression()
{
  switch(tok.type)
  {
    case token_type::NOT:
    case token_type::EXCLAMATION:
      get_next_token();
      return std::make_shared<algebra::logical_negation>(parse_unary_expression());
    case token_type::MINUS:
    {
      get_next_token();
      token t = tok;
      switch(tok.type)
      {
        // This needs to be here to properly parse -9223372036854775808
      case token_type::INTEGER:
        get_next_token();
        return std::make_shared<algebra::value>(string_to_integer("-" + t.string));
      case token_type::FLOATING_POINT:
        get_next_token();
        return std::make_shared<algebra::value>(std::stod("-" + t.string));
      default:
        return std::make_shared<algebra::negation>(parse_indexed_access_expression());
      }
    }
    case token_type::PLUS:
      get_next_token();
      [[fallthrough]];
    default:
      return parse_indexed_access_expression();
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

algebra::node_csp parser::data::parse_indexed_access_expression()
{
  algebra::node_csp left = parse_member_expression();
  while(tok.type == token_type::STARTBOXBRACKET)
  {
    get_next_token();
    algebra::node_csp index;
    if(tok.type == token_type::DOTDOT)
    {
      index = std::make_shared<algebra::value>(int64_t(0));
    } else {
      index = parse_expression();
    }
    algebra::node_csp end = nullptr;
    if(tok.type == token_type::DOTDOT)
    {
      get_next_token();
      if(tok.type == token_type::ENDBOXBRACKET)
      {
        end = std::make_shared<algebra::end_of_list>();
      } else {
        end = parse_expression();
      }
    }
    is_of_type(token_type::ENDBOXBRACKET);
    get_next_token();
    left = std::make_shared<algebra::indexed_access>(left, index, end);
  }
  return left;
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
  return parse_conditional_xor_expression();
}

void parser::data::get_next_token(int _flags)
{
  tok = lex->next_token(_flags);
}

template<typename... _T_>
void parser::data::report_error(const token& _token, exception_code _code, const std::string& _errorMsg, const _T_&... _values)
{
  throw_exception(exception_stage::compiletime, _code, _errorMsg + " at ({}, {}).", _values..., _token.line, _token.column);
}

void parser::data::report_unexpected(const token& _token) 
{
  if(_token.string.empty())
  {
    report_error(_token, exception_code::unexpected_syntax, "Unexpected token '{}'", _token.type);
  } else {
    report_error(_token, exception_code::unexpected_syntax, "Unexpected token '{}' ('{}')", _token.type, _token.string);
  }
}

bool parser::data::is_of_type(const token& _token, token_type _type)
{
  if(_token.type == _type) return true;
  if(_token.string.empty())
  {
    report_error(_token, exception_code::unexpected_syntax, "Expected token {} got {}", _type, _token.type);
  } else {
    report_error(_token, exception_code::unexpected_syntax, "Expected token {} got {} ({})", _type, _token.type, _token.string);
  }
  return false;
}

bool parser::data::is_of_type(token_type _type)
{
  return is_of_type(tok, _type);
}

parser::parser(lexer* _lexer, const value_map& _bindings) : d(new data)
{
  d->lex = _lexer;
  d->bindings = _bindings;
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
    case token_type::UNWIND:
      nodes.push_back(d->parse_unwind());
      break;
    case token_type::DELETE:
    case token_type::DETACH:
      nodes.push_back(d->parse_delete());
      break;
    case token_type::SET:
      nodes.push_back(d->parse_set());
      break;
    case token_type::REMOVE:
      nodes.push_back(d->parse_remove());
      break;
    case token_type::RETURN:
      nodes.push_back(d->parse_return());
      d->is_of_type(token_type::END_OF_FILE);
      break;
    case token_type::CALL:
      nodes.push_back(d->parse_call());
      break;
    default:
      d->report_unexpected(d->tok);
    }
  }
  switch(nodes.size())
  {
  case 0:
    d->report_error(token(), exception_code::parse_error, "Empty query.");
  case 1:
    return nodes.front();
  default:
    return std::make_shared<algebra::statements>(nodes);
  }
}

