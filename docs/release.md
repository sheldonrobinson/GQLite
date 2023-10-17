How to release gqlite
=====================

- Test amalgamate

```bash
g++ -std=c++20 -I../../../src/gqlite/include/ gqlite-amalgamate.cpp -lsqlite3 -c
```

- Ruby
```bash
  cd dists/ruby;
  rake package
```

  Test gem:
```bash
docker run -it -v `pwd`/pkg:/pkg ruby:3.1 bash
gem install /pkg/gqlite-*gem
irb
require 'gqlite'
c = GQLite::Connection.new sqlite_filename: "testdb"
c.execute_oc_query "CREATE (n) RETURN n"
``````

  Publish gem:
```bash
  gem push gqlite-1.x.x.gem
```

- Python 

```bash
  cd dists/python
  python3 setup.py check
  python3 setup.py sdist
```
  Usefull documentation:
   - https://betterscientificsoftware.github.io/python-for-hpc/tutorials/python-pypi-packaging/

  Testing:
```
docker run -it -v `pwd`/dist:/dist python:3.7 bash
pip3 install /dist/gqlite...

python3
import gqlite
c = gqlite.Connection("testdb")
c.execute_oc_query("CREATE (n) RETURN n")
```

  Publish package:
```bash
twine upload dist/*
```