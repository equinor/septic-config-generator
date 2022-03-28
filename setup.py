#!/usr/bin/env python

from setuptools import find_packages, setup


def parse_requirements(filename):
    """Load requirements from a pip requirements file"""
    try:
        lineiter = (line.strip() for line in open(filename))
        return [line for line in lineiter if line and not line.startswith("#")]
    except IOError:
        return []


setup(
    name="scg",
    description="Utility for generating SEPTIC config files based on templates. Don't install, instead download or generate exe file.",
    author="Einar S. Idso",
    author_email="eiids@equinor.com",
    packages=["scg", "scg.helpers"],
    platforms=["Windows"],
    python_requires="~=3.6",
    setup_requires=["setuptools_scm"],
    use_scm_version={"write_to": "scg/version.py"},
    entry_points={"console_scripts": ["scg=scg.scg:main"]},
    install_requires=parse_requirements("requirements.txt"),
)
