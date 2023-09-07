#include <gqlite.h>

using namespace gqlite;

exception::exception(const exception& _rhs)
{
  operator=(_rhs);
}

exception& exception::operator=(const exception& _rhs)
{
  if(_rhs.m_error.empty())
  {
    m_c_error = _rhs.m_c_error;
  } else {
    m_error = _rhs.m_error;
    m_c_error = m_error.c_str();
  }
  return *this;
}
