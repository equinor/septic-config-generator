# SEPTIC config generator

This is the documentation for the Rust-based SCG 2.x. If you are looking for documentation 
for the legacy Python-based 1.x version, please go [here](docs/HOWTO_SCG_legacy.md).

Although the functionality of 2.x is quite similar to 1.0, there are some minor differences.
When transitioning from using 1.0 to 2.x, expect having to change a few lines in your 
templates and YAML config file. The differences between 1.0 and 2.0, and what you need to
change, are documented [here](docs/Changes_1.0_2.0.md).

## About

SEPTIC config generator (scg) is a tool to generate SEPTIC configs based on one or more 
templates files, one or more Excel-tables containing substitution values, and a config 
file that defines how the templates should be combined by inserting values from the
Excel tables in uniquely identified locations.

## Introduction

Upon inspecting a SEPTIC configuration file, you will find that it can be divided into 
segments where some segments are static while others are repeated for several wells 
(or some other entity) with only minor modifications.

For example: The initial `System` section of the SEPTIC config is a static part that only
occurs once. The following `SopcProc` section usually contains a static header followed 
by the definition of a number of `SopcXvr` . The latter part usually consists of many 
repeating values for all wells. Following this, you will normally have one or more 
`DmmyAppl` sections that contain a mixture of common elements and per-well elements, and
similarly for other sections. 

By extracting these segments and placing them into separate template files, where the
repeating parts are replaced by identifier tags, this tool can recombine the templates
into a fully working SEPTIC config. Some key advantages are:
* Changes made to one well can be quickly propagated  to other wells.
* Adding wells to an existing config can be is as simple as specifying some key information 
for the new wells and re-running the tool. 
* Ensuring that a few templates and a table are correct is much easier than ensuring that
all wells are perfectly specified with the correct tags in the final config. This reduces 
the risk of faulty configs which can lead to faulty operation.

## Installation 

Download the latest version from the Releases-section on GitHub and extract scg.exe to 
somewhere in your path.

## Basic usage

The tool has three commands (or modes of operation):
 - make: Generate complete config file based on templates
 - checklogs: Inspect SEPTIC log files and report errors. *(Added in 2.4)*
 - diff: Simple utility to show difference between two files.

Type `scg.exe --help` to get basic help information for the tool. You can also get help
for each command, e.g. `scg.exe make --help` .

### scg make

This command is used to generate an output file based on a configuration layout `.yaml` file. The exit status is 0 if a file was output, 1 if no file was output and 2 if there was an error.

`--var`: Used to add global variables that are available to all templates in the layout. Example:
```scg.exe make --var final true``` will create a variable called `final` with the boolean value `true`.

`--ifchanged`: *(Added in v2.2)* If this argument is provided, the `outputfile` will only be built if at least one of the input files is newer than the `outputfile`. 

Input files include the layout `.yaml` file itself, all files in the `templatepath` directory, including any subdirectories, and all source files listed under `sources`. 
This makes it possible to kill and restart applications only when their config file has changed:
```bat
> scg make --ifchanged MyApplication.yaml && taskkill /IM QtSeptic.exe /FI "WINDOWTITLE eq MyApplication*" > nul 2>&1
```
Here the taskkill command will only be executed if the exit status from scg is 0, which means that the config file was updated.

### scg checklogs

This command is used to inspect the .out-file and the newest (by timestamp) .cnc-file in the specified run directory and report any error or warning found. It will search for the .cnc file inside the `startlogs` directory if it exists in the run directory.

```bat
> scg checklogs ..\run_main
MYAPP.out[21]: No Xvr match for Pvr TestPvr
MYAPP.cnc[51]: ERROR adding Item: SomeTag
```

## Howto

It is easiest to explain how to use the tool by example. In the file-set you will find a 
directory called `basic example` . This directory contains the following directories and 
files:
* templates: A directory containing the templates that make up a SEPTIC config file.
* example.yaml: Defines how the template files should be combined to create 
example_final.conf
* example.xlsx: An Excel file that contains data to insert into the templates.
* example.cnfg: The resulting SEPTIC config file. 
 
Download and copy the entire directory called `basic example` to `C:\Appl\SEPTIC` . 

## The template files

Take a look in the templates directory. You will find a number of template files that
can be combined to create a final `example.cnfg` .  

Upon inspecting the files, you will see that some of them contain text within double 
curly braces, e.g. `{{ Id }}` . These are identifier tags for the parts that will be 
replaced. 

Regarding file naming:
* It is not necessary to enumerate the files as is done here, but it may make it easier
to understand the layout of the final config file.
* It is also a good idea, although not required, to indicate in the file names which 
of the files contain parameters to be substituted from a source file. In the example,
those files end with `_well` . 

## The Excel file

The file `example.xlsx` contains a single worksheet with a simple table. This is the file
from which we will fetch values to insert into the templates. 

The first row contains the column headers which act as identifier tags. These tags 
correspond to the tags you saw in curly braces in the template files. The tags are case 
sensitive.

Each item, in this case each well, is listed in the following rows. The value in the 
first column is a unique identifier for the item, and must be a text string. The tags in
the template file gets replaced with the values in the table row by row.

Please note:
* As mentioned, the tag names (first row) are case sensitive. You must ensure that these
are exactly the same as the tags defined in the templates. Any typo will result in an 
error message upon config generation, so no need to worry about broken configs. 
* In order to ensure that id numbers such as '1' and '21' are both displayed with two
digits in the resulting configuration, you should use strings and not numbers in Excel. 
Simply prepend the numbers with `'` . So if you want `D{{ Id }}` to become `D01` instead 
of `D1` , you should input `'01` instead of `1` in the Id field.
* The use of formulas and any kind of text formatting is allowed. Only the resulting 
unformatted value will be used by scg.

## The config file

Inspect the config file `example.yaml` . It starts out by defining a number of paths:

```yaml
outputfile: example.cnfg

templatepath: templates

verifycontent: true

adjustspacing: true
```

All paths are relative to the directory in which `example.yaml` is found.

The `outputfile` specifies the file which will be generated by the tool. If `outputfile`
is not specified, then the result will be output to the terminal (stdout).

SCG looks for template files in the `templatepath` directory. 

When generating a config file, the default behaviour is to present any difference
between a previously generated config file and the new config as a 
[unified diff](https://en.wikipedia.org/wiki/Diff#Unified_format) before asking whether 
it is ok to replace the original. The original config will be renamed with the extension 
'.bak' before being replaced. If you don't want to be bothered with this question, you 
can set `verifycontent` to `false`.

If `adjustspacing` is set to `false`, then the rendering will default to [MiniJinja's 
behaviour](https://docs.rs/minijinja/latest/minijinja/syntax/index.html#trailing-newlines), which is to remove one trailing newline from the end of the file automatically on parsing. If set to 
`true`, then scg will make sure that there is exactly one newline after the last non-whitespace 
character in the template. This is `true` by default since that is probably what most people want.

The next section defines an Excel file that shall be used to substitute values in the
template files:

```yaml
sources:
  - filename: example.xlsx
    id: main
    sheet: Sheet1
```

The source references a worksheet called `Sheet1` in the Excel file `example.xlsx`. This is 
given a unique id `main`. The path to the file is relative to the directory containing 
`example.yaml` .

If there are other groups of elements that you wish to create templates and substitutions 
for, e.g. multiple flowlines or separator trains, or to distinguish between non-similar groups 
of wells such as production wells and injection wells, simply create another sheet (in the 
same or a new Excel sheet) and define the new source similarly with a unique id.

A template file can only use one source, so in some cases it may be necessary to repeat
information on two or more sheets. To ensure consistency, it may be a good idea to maintain
one set of values in one sheet and reference the corresponding cells from the other sheets.  

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

This defines how the sections of the final config are created from the templates. The list
of templates is processed and output in the specified sequence. Each template reference 
requires at least a filename `name` . If nothing more is specified, then the file is only
written once and cannot contain tags that are defined in sources.

If a `source` is defined, then the source is used as a look-up table for substitutions
into the config file. By default, the template is generated once per row in the source. 
So the template `03_SopcProc_well.cnfg` will be replicated three times, once for each 
of the rows `D01` , `D02` and `D03` that are specified in the source.

It is possible to specify exactly which rows to include. An example of this is shown for 
`07_DspGroupTables_well.cnfg` which will only be generated for `D01` and `D03` . It is 
also possible to use the keyword `exclude` in the same way to skip specific rows from 
the source. If both `include` and `exclude` are defined, then only rows that are part
of `include` and not in `exclude` are included.

## Generate a config

Now that you know how the files are used, let's try to generate a config. Start by making a 
copy of `example.cnfg` . Rename the copy to `example_original.cnfg`. Make sure that scg.exe 
is somewhere in your path, open a command line and change directory to `C:\appl\SEPTIC\Basic Example` .

To generate example.cnfg, type:

```
scg.exe make example.yaml
```

You can also type simply

```
scg.exe make example
```

Verify that the generated `example.cnfg` corresponds with the layout defined in the YAML 
config file and the rows in the Excel sheet. E.g. you should have SopcCvrs and SopcMvrs 
defined for all three wells, but MPCTable should only list D01 and D02.

Try to make a change to one of the template files and regenerate the config. Since 
`verifycontent` is `true`, scg will ask whether you want to replace the existing `example.cnfg`.
Changes are shown as unified diff between the two files.  

Type `scg.exe make --help` for more options to the `make` command.

### Global variables

It is possible to define global replacement variables on the command line using the `--var` 
option. The option takes two arguments: Name and value. Global variables can be used in 
any template file, also files without a defined source, using the same format as for variables 
defined in the Excel sheet. SCG will try to convert the value to boolean, integer or float 
and fall back to string as default type.

Example: 

```
scg.exe make --var simulation true --var size 2.3 --var version 1.2.3 example.yaml
```

Here `{{ simulation }}` , `{{ size }}` and `{{ version }}` will be available for use in 
all template files. The value of `simulation` will be boolean `true` , `size` will be a 
float with value `2.3` while `version` will be a string with value `1.2.3` .


## The template engine

The parameter replacement performed by the `make` command uses the [MiniJinja](https://crates.io/crates/minijinja) Rust crate. MiniJinja is based on the 
[Jinja2](https://jinja.palletsprojects.com/) Python module which was used by scg 1.0. 
MiniJinja is a very powerful templating engine that can do lots more than what has been 
described above, such as expressions (e.g. calculate offsets for placing display elements based 
on well id number), statements for inheriting or including other template files, conditionals and loops etc. This makes scg very flexible and we can, for example, easily handle SEPTIC configs with non-similar wells. 

For further information, please take a look at the 
[MiniJinja documentation](https://docs.rs/minijinja/latest/minijinja/). In particular:
* [Syntax documentation](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)
* [Filter functions](https://docs.rs/minijinja/latest/minijinja/filters/index.html)
* [Test functions](https://docs.rs/minijinja/latest/minijinja/tests/index.html)

### Custom keywords, filters and functions

In addition to the built-in [filter functions](https://docs.rs/minijinja/latest/minijinja/filters/index.html#built-in-filters) and [global functions](https://docs.rs/minijinja/latest/minijinja/functions/index.html#functions) in MiniJinja, some custom functionality has been added.

#### `now()`
Function that inserts a datestamp. The default format is `%Y-%m-%d %H:%M:%S"`. The format can be modified by providing an [strftime string](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) as function argument to customize the datestamp

Examples: <br />
`{{ now() }}` -> 2023-02-23 14:18:12 <br />
`{{ now("%a %d %b %Y %H:%M:%S") }}` -> Thu 23 feb 2023 14:18:12

#### `gitcommit`
Global variable that inserts the Git commit hash on short form.

Example: <br />
`{{ gitcommit }}` -> 714e102

#### `gitcommitlong`
Global variable that inserts the Git commit hash on long form.

Example: <br />
`{{ gitcommitlong }}` -> 714e10261b59baf4a0257700f57c5e36a6e8c6c3

#### `scgversion`
Global variable that inserts the SCG version used to create the output file.

Example: <br />
`{{ scgversion }}` -> 2.2.1

Try for example to add the following line at the top of the first template file:<br />
`// Generated with SCG v{{ scgversion }} on {{ now() }} from git commit {{ gitcommit }}`

#### `bitmask`
Filter that converts a non-negative integer or a sequence of non-negative integers into a bitmask. Each integer will be translated into a 1 in the bitmask that is otherwise 0. Takes an optional argument that is the length of the bitmask (defaults to 31).

Examples:<br />
`{{ 2 | bitmask }}` -> `0000000000000000000000000000010`<br />
`{{ [1, 3, 31] | bitmask }}` -> `1000000000000000000000000000101`<br />
`{{ [1, 3] | bitmask(5) }}` -> `00101`




