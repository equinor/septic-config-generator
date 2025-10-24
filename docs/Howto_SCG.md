# Septic config generator <!-- omit in toc -->

This is the documentation for Rust-based SCG 2.x. Documentation for the legacy Python-based 1.x version has been
removed. If you are looking for what was changed from 1.0 to 2.x, take a look in the
[changes docs](docs/Changes_1.0_2.0.md).

## Table of contents <!-- omit in toc -->

- [About](#about)
- [Introduction](#introduction)
- [Installation](#installation)
- [Usage overview](#usage-overview)
- [scg make](#scg-make)
  - [Configuration file](#configuration-file)
    - [Drawio](#drawio)
    - [Counters](#counters)
    - [Sources](#sources)
    - [Layout](#layout)
  - [Source files](#source-files)
    - [Excel source](#excel-source)
    - [CSV source](#csv-source)
    - [Combined source](#combined-source)
  - [Command-line options](#command-line-options)
  - [The template engine](#the-template-engine)
  - [Custom keywords, filters and functions](#custom-keywords-filters-and-functions)
    - [`unpack`](#unpack)
    - [`bitmask`](#bitmask)
    - [`gitcommit`](#gitcommit)
    - [`gitcommitlong`](#gitcommitlong)
    - [`now()`](#now)
    - [`scgversion`](#scgversion)
- [scg checklogs](#scg-checklogs)
- [scg update](#scg-update)
- [Howto/tutorial](#howtotutorial)
  - [The template files](#the-template-files)
  - [The source file](#the-source-file)
  - [The config file](#the-config-file)
  - [Generate a config](#generate-a-config)
- [draw.io Diagram Integration for SCG](#drawio-diagram-integration-for-scg)
  - [Prerequisites](#prerequisites)
  - [Using draw.io with SCG](#using-drawio-with-scg)
- [Coordinate and Property Extraction](#coordinate-and-property-extraction)
  - [Creating Diagrams for SCG](#creating-diagrams-for-scg)
    - [Element Types](#element-types)
    - [Properties](#properties)
    - [Adding or Editing Properties](#adding-or-editing-properties)
  - [Setting Diagram Size and Background](#setting-diagram-size-and-background)
    - [Setting Paper Size](#setting-paper-size)
    - [Creating a Fixed Size Background](#creating-a-fixed-size-background)
  - [Command Line Usage](#command-line-usage)

## About

Septic config generator (scg) is a tool to generate Septic config files based on one or more template files and one or
more Excel- or CSV-tables containing substitution values. A yaml-based config file specifies how the templates should be
combined by inserting values from the tables in locations identified by labels.

## Introduction

Upon inspecting a Septic configuration file, you will find that it can be divided into segments where some segments are
static while others are repeated for several wells (or some other entity) with only minor modifications.

For example: The initial `System` section of the Septic config is a static part that only occurs once. The following
`SopcProc` section usually contains a static header followed by the definition of a number of `SopcXvr` . The latter
part usually consists of many repeating values for all wells. Following this, you will normally have one or more
`DmmyAppl` sections that contain a mixture of common elements and per-well elements, and similarly for other sections.

By extracting these segments and placing them into separate template files, where the repeating parts are replaced by
identifier tags, this tool can recombine the templates into a fully working Septic config. Some key advantages are:

- Changes made to one well can be quickly propagated to other wells.
- Adding wells to an existing config can be is as simple as specifying some key information for the new wells and
  re-running the tool.
- Ensuring that a few templates and a table are correct is much easier than ensuring that all wells are perfectly
  specified with the correct tags in the final config. This reduces the risk of faulty configs which can lead to faulty
  operation.

## Installation

Download the latest version from the Releases-section on GitHub and extract scg.exe to somewhere in your path.

## Usage overview

The tool has four commands (or modes of operation):

- make: Generate complete config file based on templates
- checklogs: Inspect Septic log files and report errors. _(Added in 2.4)_
- diff: Simple utility to show difference between two files.
- update: Check GitHub for new release. If available, ask user whether to update. _(Added in 2.6)_

Type `scg.exe --help` to get basic help information for the tool. You can also get help for each command, e.g.
`scg.exe make --help` .

## scg make

This command is used to generate an output file based on a configuration layout `.yaml` file.

The exit status is 0 if a file was output, 1 if no file was output and 2 if there was an error.

Example:  
`scg make MyApplication.yaml`

### Configuration file

SCG uses a configuration file in the YAML format to define its behavior. The configuration file should follow the format
in the example below:

```yaml
outputfile: example.cnfg
encoding: utf-8
templatepath: templates
adjustspacing: true
verifycontent: true

drawio:
  - input: example.drawio
    pngoutput: example.png
    csvoutput: example.csv

counters:
  - name: mycounter
    value: 0

sources:
  - filename: example.xlsx
    id: wells
    sheet: Sheet1
  - filename: example.csv
    id: flowlines
    delimiter: ";"
  - filename: example_drawio_components.csv
    id: drawio_example
    delimiter: ","

layout:
  - name: 010_System.cnfg
  - name: 020_SopcProc.cnfg
  - name: 030_SopcProc_well.cnfg
    source: wells
    include:
      - D01
      - D02
  - name: 040_SopcProc_flowline.cnfg
    source: flowlines
  - name: 060_DspGroup_Overview.cnfg
    source: drawio_example
```

- `outputfile` (optional string): The file that will be generated. Writes to stdout if not provided.
- `encoding` (optional string, default: windows-1252): Specify the encoding for template files and the outputfile. Use
  any label specified in the [get an encoding](https://encoding.spec.whatwg.org/#concept-encoding-get) algorithm. The
  default may be changed to utf-8 at a later time. _(Added in v2.13)_
- `templatepath` (string): The directory that contains all template files.
- `adjustspacing` (boolean, default: true): Specifies whether to ensure exactly one newline between rendered template
  files. If `false`, then the rendering will default to
  [MiniJinja's behaviour](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#trailing-newlines).
- `verifycontent` (boolean, default: true): Whether to report differences from an already existing rendered file. Will
  ask before replacing with the new content. Set to `false` to overwrite existing file without checking for changes.
- `drawio` (optional list of `drawio` structs) Contains a list of .drawio files to extract coordinates from and convert
  to png.
- `counters` (optional list of `counter` structs): Contains a list of global auto-incrementing counter functions.
- `sources` (list of `source` structs): Contains a list of source file configurations.
- `layout` (list of `template` structs): Contains a list of templates in the order they should be rendered.

All file names and paths are relative to the location of the configuration file.

#### Drawio

_(Added in v2.13)_

OBS: for prerequisites and .drawio info, See [draw.io Diagram Integration for SCG](#drawio-diagram-integration-for-scg)
section

The `drawio` struct allows you to automate processing of `.drawio` diagram files as part of your configuration workflow.
For each entry, SCG can perform two main functions:

- **Convert the `.drawio` file to a PNG image**: This is useful for generating graphical backgrounds for displaygroups.
- **Extract coordinates and element information from the `.drawio` file to a CSV file**: This enables you to use the
  positions and properties of diagram elements in your configuration or templates, for example to place GUI elements
  accurately.

This makes it easy to keep your graphics and coordinate data in sync with your diagrams, and to integrate diagram
information directly into your config generation process.

The structure has the following fields:

- `input` (string, required): The path to the input `.drawio` file.
- `pngoutput` (string, optional): The path to the output `.png` file. If not provided, the output file will have the
  same name as the input file but with a `.png` extension.
- `csvoutput` (string, optional): The path to the output `.csv` file. If not provided, the output file will have the
  same name as the input file but with a `_components.csv` suffix.

#### Counters

_(Added in v2.7)_

The `counter` struct has the following fields:

- `name` (string): The name of the counter to create.
- `value` (optional integer, default: 0)`: The initial value to provide the counter.

For each counter that is defined, a Jinja custom function with the same name is created. It can be called in two ways:

- `countername()`: The counter is increased by 1 and the new value is returned. A counter initialized to 0 will return 1
  the first time it is called.
- `countername(somevalue)`: The counter is set to the provided value and the new value is returned.

The counters are global values: Any change to a counter value, either by incrementing or by setting to a new value in a
template, will be retained for subsequent rendered templates.

In the example above, the function will be called `mycounter()`. It is called as other Jinja custom functions by placing
it inside double braces: `{{ mycounter() }}`, `{{ mycounter(13) }}`.

To avoid printout when setting the value, try `{% do mycounter(5) %}` (see
[minijinja docs](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#-do-)) or
`{% if "" == mycounter(5) %}{% endif %}`.

#### Sources

The `source` struct represents a file that is used for replacing values in the templates. The file can be either an
Excel file or a CSV file (_since v2.5_), where the file type is identified by the extension (`.xlsx` or `.csv`). Since
v2.11, it is also possible to specify a list of `.csv` files that will be combined to a single source.

The structure has the following fields:

- `id` (string): A unique id used to reference the source file from the layout section.
- `filename` (string or list of strings): The file name that contains the substitution values. The extension must be
  either `.xlsx` or `.csv`. A third option is to specify a list of `.csv` files that will be combined into a single
  source.
- `sheet` (string): The name of the sheet where the substitution values are found. Only valid for Excel files.
- `delimiter` (optional character, default: ';'): The delimiter used in a CSV file. Only valid for `.csv` source files
  and multi-file source.

Since v2.8, each source can be accessed from within any template regardless of whether the layout item is set to iterate
over a source or not. The source is represented as a list of the rows where each row is represented as a hashmap of the
values. This makes it possible to obtain any value from any source in any template. For more on this topic, see the
[unpack](#unpack) filter.

#### Layout

The `layout` section contains one or more `template` structs that each represent a template file and how it should be
rendered to the `outputfile`. The templates will be rendered in the order they are listed. The `template` struct has the
following fields:

- `name` (string): The name of the template file. It should be located in the directory pointed to by the `templatepath`
  field.
- `source` (optional string): If provided, will iterate over each row in the source file with the provided id and render
  once per iteration.
- `include` (optional list of strings or `conditional items`): The template will be rendered only for the specified
  rows.
- `exclude` (optional list of strings or `conditional items`): The template will not be rendered for any of the
  specified rows.

Combining `include` and `exclude` will render the template for only those rows that are specified under `include` but
not specified under `exclude`. The row order in the source always determines the rendering order.

##### Including and excluding rows from sources <!-- omit in toc -->

The `include` and `exclude` sections require a bit more explanation. Both sections consist of a list that contains
either strings that reference values in the first column in the source, or `conditional items`. The latter was added in
v2.12 and follows this structure:

- `if` (string): A [MiniJinja expression](https://docs.rs/minijinja/latest/minijinja/index.html#expression-usage). All
  variables, whether defined on the command line with the `--vars` argument, global sources and rows from the current
  source can be used in the expression.
- `then` (optional list of strings): A list of items that will be included if the `if`-expression evaluates to `true`.
- `continue` (optional boolean): If not set or if `false`, stop evaluating further include (or exclude) items for this
  template. If `true`, continue evaluating following include (or exclude) items.

If `then` is not specified, the `if`-expression will be evaluated for each row in the source. Any row that causes the
expression to evaluate to `true` will be included. Use this format for including individual rows based on some property,
e.g. wells that have a downhole pressure gauge or wells that are connected to a specific flowline.

If `then` is specified, all items in the list will be included if the `if`-expression evaluates to `true`. Use this
format for including one or more rows based on a global parameter, typically specified from the command line. E.g. to
generate smaller configs with fewer wells for faster simulation times when the variable `testing` is `true`.

A contrived example of a layout item (template):

```yaml
- name: 030_SopcProc_well.cnfg
  source: wells
  include:
    - D01
    - if: "WellName is startingwith('B')"
    - if: "final == true"
      then: [D02, D03]
      continue: true
    - D04
```

Here `D01` will always be included. Next, the `startingwith()`
[MiniJinja test-function](https://docs.rs/minijinja/latest/minijinja/tests/index.html#functions) will evaluate to `true`
for any row where `WellName` starts with "B". If there is at least one match, then all matching rows will be included.
Since `continue` was not specified, the evaluation stops here and only `D01` and any `Bnn` wells are included in the
template rendering.

If the first `if`-expression had no matches, the evaluation continues to the next item. If the global variable `final`
is set to `true`, then `D02` and `D03` will be included. Since `continue` is `true`, the evaluation continues to the
next line, and `D04` is also included. The template will therefore be rendered once for each of `D01`, `D02`, `D03` and
`D04`.

Tip: By default, all rows in the source file are used for iteration, but an `include` statement effectively empties that
set and replaces it with the result of the `include` items. To have a default of including all items in case none of the
`if`-expressions match, simply add `if: "true"` as the last `conditional item`. This will evaluate to `true` for every
row and therefore re-add them.

### Source files

The source files referenced in the configuration file contain data tables used by the templates. SCG accepts two file
formats:

- Excel files with extension `.xlsx`
- CSV files with extension `.csv`

SCG will determine which file format is in use based on the extension in the `source.filename` field.

The format used by both Excel and CSV files is similar: The first row contains header labels, and the first column
contains row labels. The header labels are the values that are referenced in the templates, and are replaced by the
corresponding value found in each row. The row label identifies the row, and can be used to explicitly include or
exclude rows in the layout definition for a template.

#### Excel source

When using Excel files as source tables, multiple sheets in the same file can be used as unique sources. Cells that
contain expressions will result in the calculated value being used when rendering the template. This way all the
information about all replacement values can be contained in one single file. The disadvantage to using Excel is that it
is a binary format what is not well suited for version control. This can easily create merge conflicts that need to be
manually resolved by opening both versions of the files and comparing them visually.

Cell value types is preserved. E.g. a text cell will be treated as text when rendering templates. Excel does not
distinguish between integers and floats. Therefore a numerical value that deviates less than an epsilon from an integer
will be converted to an integer instead of to a float. Cells that contain errors will be rendered with an error text
string, e.g. "#DIV/0!" or "#N/A".

For Excel sources, the sheet must be provided. Example:

```yaml
sources:
  - filename: wells.xlsx
    sheet: well_overview
    id: wells
```

#### CSV source

The advantage to using CSV files as source is that they are plain text files and therefore very well suited for source
control. The primary disadvantage is that CSV files can only contain static values. And since they can only contain one
table, it may be necessary to use two or more CSV files, for instance one for iterating over wells and one for iterating
over flowlines, separator trains or something else. Advanced users may consider generating CSV files by e.g. crating
Python scripts to extract information from Excel files that are not part of their repository or via other sources.

Any row that starts with '#' will be ignored.

As opposed to Excel files, cell values in CSV files are always text. However, SCG will try to parse and convert the
values in the following order:

- Empty value
- Integer that starts with 0 but is not 0, e.g. `00110`, becomes string
- Integer
- Float
- Boolean

String is the fallback type. When parsing floats, both ',' and '.' are valid decimal separators.

All cell values are trimmed before parsing. This means that `a;1.0;2` and ` a ; 1.0 ; 2` are equivalent. Which again
means that "proper-looking" tables can be created: Set delimiter to e.g. `|` and maintain constant column width.

Specifying the delimiter is optional. The default value is `;`.

Example:

```yaml
sources:
  - filename: wells.csv
    delimiter: ","
    id: wells
```

#### Combined source

_(Added in v2.11)_

A number of `.csv` files can be combined into a single source. This makes it possible to keep different sections of data
separate, which can give a better overview of the data. This can also be used to semi-dynamically combine different
kinds of information, e.g. topside and subsea sections of a well for wells with reconfigurable risers.

As for single `.csv` sources, specifying the delimiter is optional. All files must use the same delimiter.

The first `.csv` file listed is the _primary_ file and will define which row labels to use (first column). All
subsequent `.csv` files are _secondary_. The _secondary_ files must as a minimum contain the same labels as those in the
_primary_ file. If a label is missing, an error message will be issued and scg will stop execution. Any row labels in
_secondary_ files that are not defined in the _primary_ file will be silently ignored.

Example: Here `wells.csv` is the _primary_ file, while `risers.csv` and `well_tuning_parameters.cvs` are _secondary_
files.

```yaml
sources:
  - filename:
      - wells.csv
      - risers.csv
      - well_tuning_parameters.csv
    delimiter: ","
    id: wells
```

This is equivalent:

```yaml
sources:
  - filename: [wells.csv, risers.csv, well_tuning_parameters.csv]
    delimiter: ","
    id: wells
```

### Command-line options

#### `--var <name> <value>` <!-- omit in toc -->

Adds a global variable that can be used by all templates in the layout. The value will be parsed to boolean, integer,
float or string, in that order.

Example:

```bat
scg.exe make --var simulation true --var size 2.3 --var version 1.2.3 example.yaml
```

The value of `simulation` will be boolean `true`, `size` will be a float with value `2.3` while `version` will be a
string with value `1.2.3`. The values can be used by inserting `{{ simulation }}` , `{{ size }}` and `{{ version }}` in
template files.

Note that these variables are not true globals: If a variable is changed within one template, the next template in the
layout will still be initialized with the original value.

#### `--ifchanged` <!-- omit in toc -->

_(Added in v2.2)_

If this argument is provided, the `outputfile` will only be built if at least one of the input files is newer than the
`outputfile` .

Input files include the layout `.yaml` file itself, all files in the `templatepath` directory, including any
subdirectories, and all source files listed under `sources`. This makes it possible to kill and restart applications
only when their config file has changed:

```bat
scg make --ifchanged MyApplication.yaml && taskkill /IM QtSeptic.exe /FI "WINDOWTITLE eq MyApplication*" > nul 2>&1
```

Here the taskkill command will only be executed if the exit status from scg is 0, which means that the config file was
updated.

### The template engine

To fully make use of all the possibilities offered by `scg make`, it is important to understand a bit about the
underlying mechanisms that are used. The parameter replacement performed by the `make` command uses the
[MiniJinja](https://crates.io/crates/minijinja) Rust crate. MiniJinja is based on the
[Jinja2](https://jinja.palletsprojects.com/) Python module which was used by scg 1.0. MiniJinja is a very powerful
templating engine that can do lots more than simply replacing variable names with values. Some examples are expressions
(e.g. calculate offsets for placing display elements based on well id number), statements for inheriting or including
other template files, conditionals, loops and filter functions. This makes scg very flexible. We can, for example,
easily handle Septic configs with non-similar wells by wrapping selected lines in conditionals.

For further information, please take a look at the
[MiniJinja documentation](https://docs.rs/minijinja/latest/minijinja/). In particular:

- [Syntax documentation](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)
- [Filter functions](https://docs.rs/minijinja/latest/minijinja/filters/index.html)
- [Test functions](https://docs.rs/minijinja/latest/minijinja/tests/index.html)

### Custom keywords, filters and functions

In addition to the built-in
[filter functions](https://docs.rs/minijinja/latest/minijinja/filters/index.html#built-in-filters) and
[global functions](https://docs.rs/minijinja/latest/minijinja/functions/index.html#functions) in MiniJinja, some custom
functionality has been added.

#### `unpack`

_(Added in v2.9)_

Filter that unpacks values for the provided keys into an array. The values in the array can then be assigned to
individual variables. This avoids having to create variables for each value with the
[set](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#-set-) statement, and also keeps the created
variables inside the scope. The filter can be applied directly to a source or a source row.

Examples:

Given a source called `wells` with this content:

| well | flowline | Pdc        |
| ---- | -------- | ---------- |
| D01  | FL1      | 13-1111-11 |
| D02  | FL2      | 13-2222-22 |

Printing selected values for all rows:

```jinja
{% for well, flowline in wells | unpack("well", "flowline") %}
{{ well }}: {{ flowline }}
{%- endfor %}
```

->

```text
D01: FL1
D02: FL2
```

Printing selected values for all wells that are connected to flowline 2, using the MiniJinja
[filter function](https://docs.rs/minijinja/latest/minijinja/filters/index.html#functions) `selectattr` and
[test function](https://docs.rs/minijinja/latest/minijinja/tests/index.html#functions) `endingwith`:

```jinja
{% for (well, pdc) in wells | selectattr("flowline", "endingwith", 2) | unpack("well", "pdc") %}
{{ well }}: {{ pdc }}
{%- endfor %}
```

-> `D02: 13-2222-22`

Printing selected values for just the "D02" row:

```jinja
{% with (flowline, pdc) = main | selectattr("well", "eq", "D02") | unpack("flowline", "pdc") %}
{{ flowline }}: {{ pdc }}
{%- endwith %}
```

-> `FL2, 13-2222-22`

#### `bitmask`

_(Added in v2.3)_

Filter that converts a non-negative integer or a sequence of non-negative integers into a bitmask. Each integer will be
translated into a 1 in the bitmask that is otherwise 0. Takes an optional argument that is the length of the bitmask
(defaults to 31, the number of available groups).

Examples:  
`{{ 2 | bitmask }}` -> `0000000000000000000000000000010`  
`{{ [1, 3, 31] | bitmask }}` -> `1000000000000000000000000000101`  
`{{ [1, 3] | bitmask(5) }}` -> `00101`

#### `gitcommit`

Global variable that inserts the Git commit hash on short form.

Example:  
`{{ gitcommit }}` -> 714e102

#### `gitcommitlong`

Global variable that inserts the Git commit hash on long form.

Example:  
`{{ gitcommitlong }}` -> 714e10261b59baf4a0257700f57c5e36a6e8c6c3

#### `now()`

Function that inserts a datestamp. The default format is `%Y-%m-%d %H:%M:%S`. The format can be customized by providing
an [strftime string](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) as function argument.

Examples:  
`{{ now() }}` -> 2023-02-23 14:18:12  
`{{ now("%a %d %b %Y %H:%M:%S") }}` -> Thu 23 feb 2023 14:18:12

#### `scgversion`

_(Added in v2.1)_

Global variable that inserts the SCG version used to create the output file.

Example:  
`{{ scgversion }}` -> 2.2.1

Try for example to add the following line at the top of the first template file:  
`// Generated with SCG v{{ scgversion }} on {{ now() }} from git commit {{ gitcommit }}`

## scg checklogs

This command is used to inspect the `.out` file and the newest (by timestamp) `.cnc` file in the specified run directory
and report any errors or warnings found. If the run directory contains a `startlogs` directory (in use since Septic
v2.85), `scg checklogs` will look there for `.cnc` files.

The exit status is 0 if everything went fine, 1 if one or more errors or warnings were found, and 2 if the check
encountered an error (e.g. unable to find or read a .cnc or .out file).

Example:

```text
scg checklogs ..\run_main
MYAPP.out[21]: No Xvr match for Pvr TestPvr
MYAPP_20230601_1415.cnc[51]: ERROR adding Item: SomeTag
```

## scg update

This command will check GitHub for the existence of a newer release. If it exists, the user will be prompted whether to
update. The update procedure replaces the called executable with the updated version.

Please note that while connected to office network, the GitHub
[rate limit](https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#rate-limiting) can
cause the update to fail with an HTTP 403 error. If that happens, you can either wait for at least 15 minutes before you
try again or connect to a different network, e.g. your home network or mobile network. Alternatively you can provide a
[GitHub authorization token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
via the hidden `--token` argument.

## Howto/tutorial

It may be easier to understand how to use the tool by example. In the docs-directory in this repository, you will find a
directory called `basic example`. This directory contains the following directories and files:

- templates: A directory containing the templates that make up a Septic config file.
- example.yaml: Defines how the template files should be combined to create example_final.cnfg
- example.xlsx: An Excel file that contains data to insert into the templates.
- example.cnfg: The resulting Septic config file.

Download and copy the entire directory called `basic example` to `C:\Appl\Septic` .

### The template files

Take a look in the templates directory. You will find a number of template files that can be combined to create a final
`example.cnfg`.

Upon inspecting the files, you will see that some of them contain text within double curly braces, e.g. `{{ Id }}` .
These are identifier tags for the parts that will be replaced.

Regarding file naming:

- It is not necessary to enumerate the files as is done here, but it may make it easier to understand the layout of the
  final config file.
- It is also a good idea, although not required, to indicate in the file names which of the files contain parameters to
  be substituted from a source file. In the example, those files end with `_well` .

### The source file

The file `example.xlsx` contains a single worksheet with a simple table. This is the file from which we will fetch
values to insert into the templates.

The first row contains the column headers which act as identifier tags. These tags correspond to the tags you saw in
curly braces in the template files. The tags are case sensitive.

Each item, in this case each well, is listed in the following rows. The value in the first column is a unique identifier
for the item, and must be a text string. The tags in the template file gets replaced with the values in the table row by
row.

Please note:

- As mentioned, the tag names (first row) are case sensitive. You must ensure that these are exactly the same as the
  tags defined in the templates. Any typo will result in an error message upon config generation, so no need to worry
  about broken configs.
- In order to ensure that id numbers such as '1' and '21' are both displayed with two digits in the resulting
  configuration, you should use strings and not numbers in Excel. Simply prepend the numbers with `'` . So if you want
  `D{{ Id }}` to become `D01` instead of `D1` , you should input `'01` instead of `1` in the Id field.
- The use of formulas and any kind of text formatting is allowed. Only the resulting unformatted value will be used by
  scg.

### The config file

Inspect the config file `example.yaml` . It starts out by defining a number of paths:

```yaml
outputfile: example.cnfg
templatepath: templates
verifycontent: true
adjustspacing: true
```

All paths are relative to the directory in which `example.yaml` is found.

The `outputfile` specifies the file which will be generated by the tool. If `outputfile` is not specified, then the
result will be output to the terminal (stdout).

SCG looks for template files in the `templatepath` directory.

When generating a config file, the default behaviour is to present any difference between a previously generated config
file and the new config as a [unified diff](https://en.wikipedia.org/wiki/Diff#Unified_format) before asking whether it
is ok to replace the original. The original config will be renamed with the extension '.bak' before being replaced. If
you don't want to be bothered with this question, you can set `verifycontent` to `false`.

If `adjustspacing` is set to `false`, then the rendering will default to
[MiniJinja's behaviour](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#trailing-newlines), which is to
remove one trailing newline from the end of the file automatically on parsing. If set to `true` , then scg will make
sure that there is exactly one newline after the last non-whitespace character in the template. This is `true` by
default since that is probably what most people want.

The next section defines the source file(s) to be used for substituting values in the template files. In this example
there is just one Excel file:

```yaml
sources:
  - filename: example.xlsx
    id: main
    sheet: Sheet1
```

The source references a worksheet called `Sheet1` in the Excel file `example.xlsx`. This is given a unique id `main`.
The path to the file is relative to the directory containing `example.yaml`.

If there are other groups of elements that you wish to create templates and substitutions for, e.g. multiple flowlines
or separator trains, or to distinguish between non-similar groups of wells such as production wells and injection wells,
simply create another sheet (in the same or a new Excel file) and define the new source similarly with a unique id.

A template file can only iterate over one source, so in some cases it may be necessary to repeat information in two or
more sources. If using Excel files, it may be a good idea to maintain one complete set of values in one sheet and
reference the corresponding cells from the other sheets.

Finally we have the layout definition:

```yaml
layout:
  - name: 01_System.cnfg
  - name: 02_SopcProc.cnfg
  - name: 03_SopcProc_well.cnfg
    source: main
  - name: 04_SmpcAppl.cnfg
  - name: 05_ExprModl_well.cnfg
    source: main
  - name: 06_DspGroupTables.cnfg
  - name: 07_DspGroupTables_well.cnfg
    source: main
    include:
      - D01
      - D02
```

This defines how the sections of the final config are created from the templates. The list of templates is processed and
output in the specified sequence. Each template reference requires at least a filename `name`. If nothing more is
specified, then the file is only written once and cannot contain tags that are defined in sources.

If a `source` is defined, then the source is used as a look-up table for substitutions into the config file. By default,
the template is rendered once per row in the source. So the template `03_SopcProc_well.cnfg` will be replicated three
times, once for each of the rows `D01`, `D02` and `D03` that are specified in the source.

It is possible to specify exactly which rows to include from the source. An example of this is shown for
`07_DspGroupTables_well.cnfg` which will only be generated for `D01` and `D03`. It is also possible to use the keyword
`exclude` in the same way to skip specific rows from the source. If both `include` and `exclude` are defined, then only
rows that are part of `include` and not in `exclude` are included.

### Generate a config

Now that you know how the files are used, let's try to generate a config. Start by making a copy of `example.cnfg`.
Rename the copy to `example_original.cnfg`. Make sure that scg.exe is somewhere in your path, open a command line and
change directory to `C:\appl\Septic\Basic Example`.

To generate example.cnfg, type:

```bat
scg.exe make example.yaml
```

Or leave out the extension:

```bat
scg.exe make example
```

Verify that the generated `example.cnfg` corresponds with the layout defined in the YAML config file and the rows in the
Excel sheet. E.g. you should have SopcCvrs and SopcMvrs defined for all three wells, but MPCTable should only list D01
and D02.

Try to make a change to one of the template files and regenerate the config. Since `verifycontent` is `true`, scg will
ask whether you want to replace the existing `example.cnfg`. Changes are shown as unified diff between the two files.

Type `scg.exe make --help` for more options to the `make` command.

## draw.io Diagram Integration for SCG

The `scg drawio` command provides utilities for processing draw.io diagram files, including converting diagrams to PNG
images and extracting coordinates and metadata for use in configuration files.

### Prerequisites

To use these features, you must have the following installed:

1. **draw.io Desktop Application**

   - Windows: Open PowerShell as Administrator and run:

     ```sh
     winget install JGraph.Draw
     ```

   - MacOS:

     ```sh
     brew install --cask drawio  # Note: requires Homebrew
     ```

   Ensure the `draw.io` executable is available in your system's PATH.

2. **VSCode Extensions**

   Two extensions are required for the best experience:

   - **draw.io Integration**: Provides draw.io diagram editing capabilities directly in VSCode
   - **Septic Extension**: Adds specialized diagram components for SCG

   These can be installed from the VSCode marketplace.

### Using draw.io with SCG

The Septic Extension enhances draw.io Integration by adding a specialized septic library with pre-configured components
designed for SCG:

- **ImageStatusLabel**: For status indicators
- **ImageXvr**: For standard XVR displays
- **ImageXvrPlot**: For plotting XVR data
- **ImageMultiXvrPlot**: For multi-series plots

Each component comes with default properties already configured for use with SCG.

## Coordinate and Property Extraction

When using the SCG tools to extract information from your diagrams:

- All components with `septic_` properties are detected and their coordinates are saved to a CSV file
- The CSV includes position data (x1, y1, x2, y2) for each component
- All properties with the `septic_` prefix are included in the CSV with the prefix removed
  - Example: `septic_name` becomes `name` in the CSV output
- If a component has a `label` attribute, it is also extracted and written to the CSV as a column named `_label`
- For multi-value fields (e.g., color lists like `"red" "blue" "green"`), a `num_values` column is included with the
  count of items

### Creating Diagrams for SCG

When creating diagrams for use with SCG, follow these guidelines:

#### Element Types

- See `docs/basic example/example.drawio` for examples
- Use the components from the Septic library in the draw.io sidebar for the easiest workflow
- Other components can be used as long as they have properties with the `septic_` prefix

#### Properties

- The Septic library components already include these recommended standard properties:
  - `septic_type`: Defines the component type (e.g., "ImageXvr")
  - `septic_name`: Defines the component name (e.g., "18PT0056")
- Common special properties for component types: `colors`, `texts`, `backgroundcolors` etc

#### Adding or Editing Properties

To modify properties:

1. Right-click on the component
2. Select "Edit Data"
3. Add or modify properties with the `septic_` prefix
4. Values will be preserved in the extracted CSV

### Setting Diagram Size and Background

Controlling the diagram size is important for consistent output when exporting to images:

#### Setting Paper Size

1. Go to **File → Page Setup** to set the desired resolution or paper size for your diagram
2. Set appropriate dimensions (e.g., 1920×1080 for HD display)

#### Creating a Fixed Size Background

By default, draw.io will automatically crop images when exporting to PNG, which may result in unexpected dimensions. To
ensure consistent sizing:

1. **Create a dedicated background layer**:

   - Open the layers panel (usually in the bottom-right)
   - Add a new layer and name it "Background"
   - Move it to the bottom of the layer stack

2. **Add a fixed-size background rectangle**:

   - Insert a rectangle on the background layer
   - Set its dimensions to match your desired output size (e.g., 1920×1082)
     - **Note**: For 1920×1080 resolution, add 2 pixels to the width (use 1922) to compensate for draw.io's sizing
       behavior
   - Set the fill color as desired for your background

3. **Lock the background layer**:
   - Click the lock icon next to the background layer
   - This prevents accidental edits to your background

This approach ensures that exported images will maintain consistent dimensions regardless of the content layout.

### Command Line Usage

The SCG tool provides the following commands for working with draw.io files:

```sh
scg drawio components  --input <file.drawio> [--output <coords.csv>]
scg drawio 2png --ipnut <file.drawio> [--output <file.png>]
```

This command extracts component coordinates and properties from a draw.io file and saves them to a CSV file. If no
output file is specified, it will create one with the same base name as the input file.
