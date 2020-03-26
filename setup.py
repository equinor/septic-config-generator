#!/usr/bin/env python

from setuptools import setup

setup(name='scg',
      description='Utility for generating SEPTIC config files based on templates',
      author='Einar S. Idso',
      author_email="eiids@equinor.com",
      packages=['scg'],
      platforms=['Windows'],
      package_data={'': ['*.md']},
      python_requires='~=3.6',
      setup_requires=['setuptools_scm'],
      use_scm_version={'write_to': 'scg/version.py'},
      install_requires=['jinja2',
                        'strictyaml',
                        'openpyxl']
      )
