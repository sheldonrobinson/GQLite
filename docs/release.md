How to release gqlite
=====================

- Ruby
```bash
  cd dists/ruby;
  rake package
  gem push gqlite-1.x.x.gem
```

- Python 

```bash
  cd dists/python
  python setup.py check
  python setup.py sdist
```
  Usefull documentation:
   - https://betterscientificsoftware.github.io/python-for-hpc/tutorials/python-pypi-packaging/