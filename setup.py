#!/usr/bin/env python

from setuptools import setup

version = {}
with open('scg/version.py') as f:
    exec(f.read(), version)

setup(name='scg',
      description='Utility for generating SEPTIC config files based on templates',
      version=version['__version__'],
      author='Einar S. Idso',
      author_email="eiids@equinor.com",
      packages=['scg'],
      platforms=['Windows'],
      package_data={'': ['*.md']},
      python_requires='~=3.6',
      install_requires=['jinja2',
                        'strictyaml',
                        'openpyxl']
      )
