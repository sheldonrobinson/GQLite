#pragma once

#define GQLITE_UNUSED (void*)

//BEGIN gqlite Foreach

// The following macros are for internal use by gqlite, no warranty of behavior is guaranteed

#define __GQLITE_INC(I) __GQLITE_INC_## I
#define __GQLITE_INC_0 1
#define __GQLITE_INC_1 2
#define __GQLITE_INC_2 3
#define __GQLITE_INC_3 4
#define __GQLITE_INC_4 5
#define __GQLITE_INC_5 6
#define __GQLITE_INC_6 7
#define __GQLITE_INC_7 8
#define __GQLITE_INC_8 9
#define __GQLITE_INC_9 10
#define __GQLITE_INC_10 11
#define __GQLITE_INC_11 12
#define __GQLITE_INC_12 13
#define __GQLITE_INC_13 14
#define __GQLITE_INC_14 15
#define __GQLITE_INC_15 16
#define __GQLITE_INC_16 17
#define __GQLITE_INC_17 18
#define __GQLITE_INC_18 19
#define __GQLITE_INC_19 20
#define __GQLITE_INC_20 21

#define __GQLITE_FOREACH__0(WHAT, SEP, I, ...)
#define __GQLITE_FOREACH__1(WHAT, SEP, I, X, ...) WHAT(X, I)
#define __GQLITE_FOREACH__2(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__1(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__3(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__2(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__4(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__3(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__5(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__4(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__6(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__5(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__7(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__6(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__8(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__7(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__9(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__8(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__10(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__9(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__11(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__10(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__12(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__11(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__13(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__12(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__14(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__13(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__15(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__14(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__16(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__15(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__17(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__16(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__18(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__17(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__19(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__18(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__20(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__19(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__21(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__20(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__22(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__21(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__23(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__22(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__24(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__23(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__25(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__24(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__26(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__25(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__27(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__26(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__28(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__27(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__29(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__28(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)
#define __GQLITE_FOREACH__30(WHAT, SEP, I, X, ...) WHAT(X, I) GQLITE_UNPACK SEP __GQLITE_FOREACH__29(WHAT, SEP, __GQLITE_INC(I), __VA_ARGS__)

#define __GQLITE_GET_MACRO(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, NAME, ...) NAME

/**
 * Execute action(X, I) on each variadic arguments.
 * X will contains the argument and I the index.
 * And add sep between each element
 * 
 * @code
 * #define PRINT(X, I) std::cout << " " # I ": " << X;
 * GQLITE_FOREACH(PRINT, do();, a, b, c)
 * @endcode
 * 
 * Will expand into:
 * 
 * @code
 * std::cout << " " "0" ": " << a; do();
 * std::cout << " " "1" ": " << b; do();
 * std::cout << " " "2" ": " << c;
 * @endcode
 */
#define GQLITE_FOREACH_SEP(action, sep, ...) \
  __GQLITE_GET_MACRO(void, __VA_ARGS__ __VA_OPT__(,) \
  __GQLITE_FOREACH__29, __GQLITE_FOREACH__28, __GQLITE_FOREACH__27, __GQLITE_FOREACH__26, __GQLITE_FOREACH__25, __GQLITE_FOREACH__24, __GQLITE_FOREACH__23, __GQLITE_FOREACH__22, __GQLITE_FOREACH__21, __GQLITE_FOREACH__20, \
  __GQLITE_FOREACH__19, __GQLITE_FOREACH__18, __GQLITE_FOREACH__17, __GQLITE_FOREACH__16, __GQLITE_FOREACH__15, __GQLITE_FOREACH__14, __GQLITE_FOREACH__13, __GQLITE_FOREACH__12, __GQLITE_FOREACH__11, __GQLITE_FOREACH__10, \
  __GQLITE_FOREACH__9, __GQLITE_FOREACH__8, __GQLITE_FOREACH__7, __GQLITE_FOREACH__6, __GQLITE_FOREACH__5, __GQLITE_FOREACH__4, __GQLITE_FOREACH__3, __GQLITE_FOREACH__2, __GQLITE_FOREACH__1, __GQLITE_FOREACH__0)(action, (sep), 0, __VA_ARGS__)

/**
 * Execute action(X, I) on each variadic arguments.
 * X will contains the argument and I the index.
 * 
 * @code
 * #define PRINT(X, I) std::cout << " " # I ": " << X;
 * GQLITE_FOREACH(PRINT, a, b, c)
 * @endcode
 * 
 * Will expand into:
 * 
 * @code
 * std::cout << " " "0" ": " << a;
 * std::cout << " " "1" ": " << b;
 * std::cout << " " "2" ": " << c;
 * @endcode
 */
#define GQLITE_FOREACH(action, ...) GQLITE_FOREACH_SEP(action, , __VA_ARGS__)

//END gqlite Foreach

//BEGIN gqlite List

#define __GQLITE_LIST_ACTION(__X__, __I__) , __X__

/**
 * Concatenate the arguments of the macro in a comma seperated list.
 * Usefull to enable multiple template parameters in a macro.
 * 
 * @code
 * SOME_MACRO(some_template<GQLITE_LIST(A,B)>)
 * @endcode
 */
#define GQLITE_LIST(__A__, ...) __A__ GQLITE_FOREACH(__GQLITE_LIST_ACTION, __VA_ARGS__)

//END gqlite List

namespace gqlite::traits
{
  /**
   * Define a trait that test if \ref T is in the set \ref Ts.
   * 
   * \code
   * contains_v<int, std::string, double> == false;
   * contains_v<std::string, std::string, double> == true;
   * \endcode
   */
  template<typename T, typename... Ts>
  struct contains : std::disjunction<std::is_same<T, Ts>...>
  {};
  template<typename T, typename... Ts>
  inline constexpr bool contains_v = contains<T, Ts...>::value;
}
