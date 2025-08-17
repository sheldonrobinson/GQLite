# Gem
require 'rubygems'

# Gemspec

GEMSPEC = Gem::Specification.new do |s|
  s.name = "gqlite"
  s.version = "1.2.3"
  s.summary = "Ruby bindings for GQLite, a Graph Query library."
  s.description = <<-DESC
GQLite is a Rust-language library, with a C interface, that implements a small, fast, self-contained, high-reliability, full-featured, Graph Query database engine.
GQLite support multiple database backends, such as SQLite and redb.
This enable to achieve high performance and for application to combine Graph queries with traditional SQL queries.

GQLite source code is license under the [MIT License](LICENSE) and is free to everyone to use for any purpose. 

The official repositories contains bindings/APIs for C, C++, Python, Ruby and Crystal.

The library is still in its early stage, but it is now fully functional. Development effort has now slowed down and new features are added on a by-need basis. It supports a subset of OpenCypher, with some ISO GQL extensions.
  
Example of use
--------------

```ruby
require 'gqlite'

begin
  # Create a database on the file "test.db"
  connection = GQLite::Connection.new filename: "test.db"

  # Execute a simple query to create a node and return all the nodes
  value = connection.execute_oc_query("CREATE () MATCH (n) RETURN n")

  # Print the result
  if value.nil?
    puts "Empty results"
  else
    puts "Results are \#{value.to_s}"
  end
rescue GQLite::Error => ex
  # Report any error
  puts "An error has occured: \#{ex.message}"
end

```

The documentation for the GQL query language can found in [OpenCypher](https://auksys.org/documentation/5/libraries/gqlite/opencypher/) and for the [API](https://auksys.org/documentation/5/libraries/gqlite/api/).

DESC
  s.authors     = ["Cyrille Berger"]
  s.homepage    = "https://gitlab.com/auksys/gqlite"
  s.license     = "MIT"
  
  s.files = Dir[
    "lib/**/*.rb",
    "ext/**/.cargo/config.toml",
    "ext/**/*.{rs,toml,lock,rb,pest,sql}"
  ]
  s.require_paths = ["lib"]  
  s.extensions = %w[ext/gqliterb/extconf.rb]

  # needed until rubygems supports Rust support is out of beta
  s.add_dependency "rb_sys", "~> 0.9.39"

  # only needed when developing or packaging your gem
  s.add_development_dependency "rake-compiler", "~> 1.2.0"
end
