/**
 * logging file allowing optional support of clog
 */

#include "config.h"

#include <iostream>
#include "format.h"

#define gqlite_error(msg, ...) std::cerr << "ERROR: " __FILE__ ":" << __LINE__ << ": " << gqlite::format_string(msg __VA_OPT__(,) __VA_ARGS__) << std::endl;
#define gqlite_warning(msg, ...) std::cerr << "WARNING: " __FILE__ ":" << __LINE__ << ": " << gqlite::format_string(msg __VA_OPT__(,) __VA_ARGS__) << std::endl;
#define gqlite_debug(msg, ...) std::cerr << "DEBUG: " __FILE__ ":" << __LINE__ << ": " << gqlite::format_string(msg __VA_OPT__(,) __VA_ARGS__) << std::endl;
#define gqlite_info(msg, ...) std::cerr << "INFO: " __FILE__ ":" << __LINE__ << ": " << gqlite::format_string(msg __VA_OPT__(,) __VA_ARGS__) << std::endl;
#define gqlite_fatal(msg, ...) std::cerr << "FATAL ERROR" __FILE__ ":" << __LINE__ << ": " << gqlite::format_string(msg __VA_OPT__(,) __VA_ARGS__) << std::endl; std::abort()

#define gqlite_debug_vn(...) clog_debug_vn(__VA_ARGS__)

#if NDEBUG

#define gqlite_assert(assrt) do { } while ((false) && (assrt))
#define gqlite_assert_msg(assrt, msg, ...) gqlite_assert(assrt)
#define gqlite_near_equal_assert(v1, v2, tol) (void)(v1); (void)(v2); (void)(tol); do { } while (false)

#else

#define gqlite_assert(assrt)                                     \
  if( not (assrt ) )                                                \
  {                                                                 \
    std::cerr << "assertion failed: " << #assrt << std::endl;       \
    std::abort();                                                   \
  }

#define gqlite_assert_msg(assrt, msg, ...)                                    \
  if( not (assrt ) )                                                        \
  {                                                                         \
    std::cerr << "assertion failed: " << msg << " " << #assrt << std::endl; \
    std::abort();                                                           \
  }

#define gqlite_near_equal_assert(v1, v2, tol)                                                          \
  {                                                                                                       \
    auto __clog__nea_ev1_ = (v1);                                                                         \
    auto __clog__nea_ev2_ = (v2);                                                                         \
    auto __clog__nea_tol_ = (tol);                                                                        \
    auto __clog__nea_error_ = std::abs(__clog__nea_ev1_ - __clog__nea_ev2_);                              \
    if(__clog__nea_error_ > __clog__nea_tol_)                                                             \
    {                                                                                                     \
      std::cerr << "Expected near equal: '" << __clog__nea_ev1_ << "' != '" << __clog__nea_ev2_           \
                << "' error: '" << __clog__nea_error_ << "', tolerance: '" << __clog__nea_tol_ << "'");   \
    }                                                                                                     \
  }


#endif
