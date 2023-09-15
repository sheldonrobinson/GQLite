Installation
============

Compile from source
--------------------

From source, by default, git clone fetch the *stable* branch of GQLite:

```bash
git clone https://gitlab.com/cyloncore/GQLite.git
cd GQLite
mkdir build
cd build
cmake ..
make -j4
make install
```

Amalgamate
----------

From the [releases page](https://gitlab.com/cyloncore/GQLite/-/releases) it is possible to download an amalgamate variant of GQLite, which contains a single cpp file and two headers. This allow to easilly embedd GQLite in your own application build.

Using lrs-pkg
-------------

Using [lrs-pkg](https://gitlab.com/cyloncore/lrs-pkg) build environment:

```bash
lrs-pkg get --repo cyloncore-core https://gitlab.com/cyloncore/lrs-pkg-core-repositories.git
lrs-pkg get gqlite
lrs-pkg build
```

Using pip (Python)
------------------

The pip package currently contains a copy of GQLite which is built during installation: 

```bash
pip install gqlitedb
```

Using gem (Ruby)
----------------

The gem package currently contains a copy of GQLite which is built during installation: 

```bash
gem install gqlite
```
