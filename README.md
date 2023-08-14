# SEPTIC config generator

![Build status](https://img.shields.io/github/actions/workflow/status/equinor/septic-config-generator/ci.yml)
![Codecov](https://img.shields.io/codecov/c/github/equinor/septic-config-generator)
![GitHub all releases](https://img.shields.io/github/downloads/equinor/septic-config-generator/total)
![GitHub tag (with filter)](https://img.shields.io/github/v/tag/equinor/septic-config-generator?label=latest)
![GitHub release (by tag)](https://img.shields.io/github/downloads/equinor/septic-config-generator/latest/total)

SEPTIC config generator (scg) is a tool to generate SEPTIC configs based on one or more 
template files, one or more Excel-tables containing substitution values, and a config 
file that defines how the templates should be combined by inserting values from the
Excel tables in uniquely identified locations.

Documentation for the Rust-based 2.x version of scg can be found in [docs/Howto_SCG.md](docs/Howto_SCG.md). If you are looking for the documentation for the legacy Python-based 1.0 version, see [docs/Howto_SCG_legacy.md](docs/Howto_SCG_legacy.md). If you are interested in what was changed from the Python-based version to the first Rust-based version (2.0) and how you can amend your templates and yaml file to be used with the new version, see [docs/Changes_1.0_2.0.md](docs/Changes_1.0_2.0.md).

The legacy 1.x version will not be updated. All new development work will be on the Rust-
based 2.x version.
