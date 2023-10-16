#include <gqlite.h>

#include "gqlite_p.h"

using namespace gqlite;

struct exception::data
{
  exception_stage stage;
  exception_code code;
  std::string error;
  mutable std::string full_error;
  const char* c_error;
};

struct gqlite::exception_reader
{
  const exception& ex;
  exception_code get_code() const
  {
    return ex.d->code;
  }
  std::string get_message() const
  {
    return ex.d->error;
  }
};

exception_code gqlite::get_execption_code(const exception& _exception)
{
  return exception_reader{_exception}.get_code();
}

std::string gqlite::get_execption_message(const exception& _exception)
{
  return exception_reader{_exception}.get_message();
}


struct gqlite::exception_maker
{
  exception operator()(exception_stage _stage, exception_code _code, const std::string& _error)
  {
    return exception(new exception::data{_stage, _code, _error, std::string(), nullptr});
  }
  exception operator()(exception_stage _stage, exception_code _code, const char* _error)
  {
    return exception(new exception::data{_stage, _code, std::string(), std::string(), _error});
  }
  exception operator()(exception_stage _stage, exception _ex)
  {
    _ex.d->stage = _stage;
    return _ex;
  }
};

[[noreturn]] void gqlite::throw_exception(exception_stage _stage, exception_code _code, const std::string& _error)
{
  throw exception_maker()(_stage, _code, _error);
}

[[noreturn]] void gqlite::throw_exception(exception_stage _stage, exception_code _code, const char* _error)
{
  throw exception_maker()(_stage, _code, _error);
}

[[noreturn]] void gqlite::rethrow_exception(exception_stage _stage, const exception& _ex)
{
  throw exception_maker()(_stage, _ex);
}

exception::exception(data* _d) : d(_d)
{
}

exception::~exception()
{
  delete d;
}

exception::exception(exception&& _rhs) : d(_rhs.d)
{
  *(const_cast<data**>(&_rhs.d)) = nullptr;
}

exception::exception(const exception& _rhs) : d(new data(*_rhs.d))
{
}

const char* exception::what() const throw()
{
  if(d->full_error.empty())
  {
    switch(d->stage)
    {
      using enum exception_stage;
      case unspecified:
        break;
      case runtime:
        d->full_error = "RunTime: ";
        break;
      case compiletime:
        d->full_error = "CompileTime: ";
        break;
    }
    switch(d->code)
    {
      using enum exception_code;
      case unspecified:
        break;
      case internal_error:
        d->full_error += "InternalError: "; break;
      case invalid_json:
        d->full_error += "InvalidJson: "; break;
      case table_error:
        d->full_error += "TableError: "; break;
      case value_error:
        d->full_error += "ValueError: "; break;
      case unimplemented_error:
        d->full_error += "UnimplementedError: "; break;
      case column_name_conflict:
        d->full_error += "ColumnNameConflict: "; break;
      case integer_overflow:
        d->full_error += "IntegerOverflow: "; break;
      case invalid_argument_type:
        d->full_error += "InvalidArgumentType: "; break;
      case invalid_number_literal:
        d->full_error += "InvalidNumberLiteral: "; break;
      case invalid_number_of_arguments:
        d->full_error += "InvalidNumberOfArguments: "; break;
      case invalid_parameter_use:
        d->full_error += "InvalidParameterUse: "; break;
      case negative_integer_argument:
        d->full_error += "NegativeIntegerArgument: "; break;
      case non_constant_expression:
        d->full_error += "NonConstantExpression: "; break;
      case parse_error:
        d->full_error += "ParseError: "; break;
      case requires_directed_relationship:
        d->full_error += "RequiresDirectedRelationship: "; break;
      case undefined_variable:
        d->full_error += "UndefinedVariable: "; break;
      case unknown_function:
        d->full_error += "UnknownFunction: "; break;
      case unexpected_syntax:
        d->full_error += "UnexpectedSyntax: "; break;
      case variable_already_bound:
        d->full_error += "VariableAlreadyBound: "; break;
      case variable_type_conflict:
        d->full_error += "VariableTypeConflict: "; break;
    }
    if(d->c_error)
    {
      d->full_error += d->c_error;
    } else {
      d->full_error += d->error;
    }
  }
  return d->full_error.c_str();
}

exception& exception::operator=(const exception& _rhs)
{
  *d = *_rhs.d;
  return *this;
}
