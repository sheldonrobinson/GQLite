How to release gqlite
=====================

- Check version numbers:
  - CMakeLists.txt
  - Cargo.toml
  - crates/gqb crates/gqls crates/gqls-macro crates/gqlparser
  - bindings/python/gqlitepy/pyproject.toml
  - bindings/ruby/gqliterb.gemspec
- Make sure there is a test database in test/rb/data for the new version, and that it is called used in gqlite-rspec

- Release crates:

```bash
cargo release
```

- Ruby, in build directory

```bash
  cd bindings/ruby;
  yet build_gem
```

  Test gem:
```bash
docker run -it -v `pwd`/pkg:/pkg cyloncore/ci:ruby bash
gem install /pkg/gqlite-*gem
irb
require 'gqlite'
c = GQLite::Connection.new filename: "testdb"
c.execute_oc_query "CREATE (n) RETURN n"
``````

Alternatively, using any ruby image:

```bash
docker run -it -v `pwd`/pkg:/pkg ruby:3.1 bash
apt update
apt install libclang-dev
gem install /pkg/gqlite-*gem
irb
require 'gqlite'
c = GQLite::Connection.new filename: "testdb"
c.execute_oc_query "CREATE (n) RETURN n"
``````

  Publish gem:
```bash
  gem push gqlite-1.x.x.gem
```

- Python, from source directory:

```bash
  cd bindings/python/gqlitepy
  yte make_sdist
  yte build_many_wheel
```
  Usefull documentation:
   - https://betterscientificsoftware.github.io/python-for-hpc/tutorials/python-pypi-packaging/

  Testing (building):
```
docker run -it -v `pwd`/wheels:/wheels python:3.9 bash
pip3 install /wheels/gqlite*

python3
import gqlite
c = gqlite.Connection("testdb")
c.execute_oc_query("CREATE (n) RETURN n")
```


  Testing (wheel):
```
docker run -it -v `pwd`/wheels:/wheels python:3.9 bash
pip3 install /wheels/wheelhouse/gqlite*cp39-*

python3
import gqlite
c = gqlite.Connection("testdb")
c.execute_oc_query("CREATE (n) RETURN n")
```

  Publish package:
```bash
twine upload wheels/*.tar.gz
twine upload wheels/wheelhouse/*whl
```