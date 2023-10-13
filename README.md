# SEPTIC config generator

![Build status](https://img.shields.io/github/actions/workflow/status/equinor/septic-config-generator/ci.yml)
![Codecov](https://img.shields.io/codecov/c/github/equinor/septic-config-generator)
![GitHub all releases](https://img.shields.io/github/downloads/equinor/septic-config-generator/total)
![GitHub tag (with filter)](https://img.shields.io/github/v/tag/equinor/septic-config-generator?label=latest)
![GitHub release (by tag)](https://img.shields.io/github/downloads/equinor/septic-config-generator/latest/total)

SEPTIC config generator (scg) is a tool to generate SEPTIC config files based on one or more template files and one or
more Excel- or CSV-tables containing substitution values. A yaml-based config file specifies how the templates should be
combined by inserting values from the tables in uniquely identified locations identified by labels.

Scg uses [MiniJinja](https://docs.rs/minijinja/latest/minijinja/syntax/index.html) as template engine.

Although scg was written specifically to make it easier to create config files for SEPTIC, it can be used to merge any
kind of text files where one or more files needs to be repeated while iterating over table rows.

Documentation can be found in [docs/Howto_SCG.md](docs/Howto_SCG.md).

Note: Scg was originally developed as a Python script converted to an executable with PyInstaller. Release 2.0 was a
complete rewrite in Rust. The legacy Python 1.x version has since been removed from the repository along with its
documentation. If you are interested in what was changed from the 1.x version to 2.0 and how you can amend your old
templates and yaml file to be used with the 2.x series, see [docs/Changes_1.0_2.0.md](docs/Changes_1.0_2.0.md).
