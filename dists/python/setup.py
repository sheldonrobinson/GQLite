#!/usr/bin/env python

import os
import sys

from setuptools import setup, find_packages, Extension
from setuptools.command.build_clib import build_clib
from distutils.ccompiler import new_compiler

# # os.chdir(os.path.dirname(sys.argv[0]) or ".")

# # Create compiler with default options
# c = new_compiler()

# # Optionally add include directories etc.
# c.add_include_dir("src/")

# # Compile into .o files
# objects = c.compile(['src/gqlite-amalgamate.cpp'], extra_postargs=['-std=c++20', '-fPIC'], output_dir="build/")
# c.link_shared_lib(objects, "gqlite", output_dir="build/")


setup(
    name="gqlite",
    version="0.9",
    description="Python bindings for GQLite, a Graph Query library",
    long_description=open("README.md", "rt").read(),
    url="https://gitlab.com/cyloncore/gqlite",
    author="Cyrille Berger",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
    ],
    packages=find_packages(),
    libraries = [ ('gqlite', {'sources': ['src/gqlite-amalgamate.cpp'], 'cflags': ['-std=c++20', '-fPIC', '-Isrc'], 'build-clib': 'build/'})],
    cmdclass = {'build_clib': build_clib},
    cffi_modules=[
        "./build-ffi.py:build_for_pip",
    ]
)