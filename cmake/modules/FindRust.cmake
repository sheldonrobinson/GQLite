
find_program(Cargo_EXECUTABLE cargo)

if(Cargo_EXECUTABLE)
  set(Rust_FOUND TRUE)

  FIND_PACKAGE_HANDLE_STANDARD_ARGS(Rust DEFAULT_MSG Cargo_EXECUTABLE )

endif()

mark_as_advanced(
  Rust_FOUND
  Cargo_EXECUTABLE
)
