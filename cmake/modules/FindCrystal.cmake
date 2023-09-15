
find_program(Crystal_EXECUTABLE crystal)

if(Crystal_EXECUTABLE)
  set(Crystal_FOUND TRUE)

  FIND_PACKAGE_HANDLE_STANDARD_ARGS(Crystal DEFAULT_MSG Crystal_EXECUTABLE )
  execute_process(COMMAND ${Crystal_EXECUTABLE} env CRYSTAL_PATH OUTPUT_VARIABLE Crystal_PATH OUTPUT_STRIP_TRAILING_WHITESPACE)

endif()

mark_as_advanced(
  Crystal_FOUND
  Crystal_EXECUTABLE
  Crystal_PATH
)
