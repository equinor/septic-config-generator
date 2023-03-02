# Changes from Python-based 1.0 to Rust-based 2.0

Although the functionality of 2.x is quite similar to 1.0, there are some minor differences.
When transitioning from using 1.0 to 2.x, expect having to change a few lines in your 
templates and YAML config file.

This document describes the differences between the last Python-based version (1.0) and the first Rust-based version (2.0). Further changes for 2.1 etc. will be documented in [CHANGELOG.md](../CHANGELOG.md) and in the release notes.

The primary difference is, of course, that scg version 2.x is written in Rust instead of Python. This should reduce execution times significantly. Furthermore, scg 2.x uses Rust's [MiniJinja](https://docs.rs/minijinja) templating engine instead of Python's [Jinja2](https://jinja.palletsprojects.com/). 

MiniJinja is based on the Jinja2 engine and supports a range of features from Jinja2, but there are differences and you may experience that some filters and expressions no longer work. If you experience this, then please let me know and we can either find a work-around or we can implement [custom filters](https://docs.rs/minijinja/0.30.4/minijinja/filters/index.html#custom-filters) or [custom functions](https://docs.rs/minijinja/0.30.4/minijinja/functions/index.html#custom-functions) to solve the issue. An overview of the differences between MiniJinja and Jinja2 is available [here](https://github.com/mitsuhiko/minijinja/blob/main/COMPATIBILITY.md)

## Removed

- Reverting config files with `scg revert` has been removed since it is no longer in use by anyone.
- The command line argument `--no-verify` has been removed. It can easily be re-added if anyone needs it.
- The diffing method (`scg diff`) has not yet been reimplemented in 2.0. It is scheduled to be added in the next feature release.

## Changed

- The expression `now() 'local', 'format-string'` has been replaced with a custom function `now()` which takes an optional [strftime string](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) as argument. The new function will always use local time.
- The keyword `gitcommit` is no longer an expression but a global variable. It inserts the short-form of the hash instead of the long-form.
- If both `include` and `exclude` are provided for a template layout entry, then only items that are included and not excluded will be iterated over. Previously the `exclude` option was ignored if `include` was provided.
- SCG version 1 used YAML spec 1.1, which provided [multiple ways](https://yaml.org/type/bool.html) to specify boolean values (`true`/`yes`/`on`, `false`/`no`/`off`). SCG 2.x uses the [YAML spec 1.2](https://yaml.org/spec/1.2.2/#10212-boolean), which requires the use of `true` and `false`.

## New

- [Trailing newlines](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#trailing-newlines) can be somewhat frustrating both with Jinja2 and MiniJinja. The new config option `adjustspacing` tries to solve this. Set it to `true` to force scg to leave exactly one blank line after the last non-whitespace character in each template. Newlines at the beginning of the templates are not affected.
- The global variable `gitcommitlong` has been added. It does the same as the `gitcommit` expression in 1.0.

# How to make your project compatible with scg version 2.x

## YAML config
1. Remove the keys `masterpath` and `masterkey`.
2. Ensure that `verifycontent` is either `true` or `false` (not `yes` or `no`, `on` or `off`).

## Template files
1. Replace `{% gitcommit %}` with `{{ gitcommitlong }}` or alternatively `{{ gitcommit }}`
2. Replace `{% now 'local', '%a %d %b %Y %H:%M:%S' %}` with `{{ now("%a %d %b %Y %H:%M:%S") }}` or simply `{{ now() }}`.
