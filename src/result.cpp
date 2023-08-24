#include "gqlite.h"

using namespace gqlite;

struct result::data
{
  status st;
  std::string error;
  std::any value;
};

result::result() : d(new data)
{}

result::result(const result& _rhs) : d(_rhs.d)
{
}

result& result::operator=(const result& _rhs)
{
  d = _rhs.d;
  return *this;
}

result::~result()
{}

result result::from_error(const std::string& _error)
{
  result r;
  r.d->st = status::failure;
  r.d->error = _error;
  return r;
}

result::status result::get_status() const
{
  return d->st; 
}

std::string result::get_error() const
{
  return d->error;
}

std::any result::value()
{
  return d->value;
}
