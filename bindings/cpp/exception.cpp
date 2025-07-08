#include "exception.h"
#include "gqlite.h"

using namespace gqlite;

struct exception::data
{
  std::string error;
  const char* c_error;
};

struct gqlite::exception_maker
{
  exception operator()(const std::string& _error)
  {
    return exception(new exception::data{_error, nullptr});
  }
  exception operator()(const char* _error)
  {
    return exception(new exception::data{std::string(), _error});
  }
};

void gqlite::throw_exception(const std::string& _error) { throw exception_maker()(_error); }

void gqlite::throw_exception(const char* _error) { throw exception_maker()(_error); }

exception::exception(data* _d) : d(_d) {}

exception::~exception() { delete d; }

exception::exception(exception&& _rhs) : d(_rhs.d) { *(const_cast<data**>(&_rhs.d)) = nullptr; }

exception::exception(const exception& _rhs) : d(new data(*_rhs.d)) {}

const char* exception::what() const throw()
{
  return d->c_error == nullptr ? d->error.c_str() : d->c_error;
}

exception& exception::operator=(const exception& _rhs)
{
  *d = *_rhs.d;
  return *this;
}