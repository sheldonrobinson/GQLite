
build:
  cargo build

unittest:
  cargo test

[working-directory: 'tests/rb/']
tck junit:
  ./gqlite-tck.rb {{if junit == "on" { "--output-junit --output-dir {{justfile_directory()}}/Testing/junit" } else { "" } }}

test: unittest (tck "off")
  @echo "All tests have passed!"

coverage:
  scripts/coverage