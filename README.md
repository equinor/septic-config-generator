![Build](https://github.com/equinor/septic-config-generator/workflows/Build/badge.svg)

# SEPTIC config generator

SEPTIC config generator (scg) is a tool to generate SEPTIC configs based on one or more 
templates files, one or more Excel-tables containing substitution values, and a config 
file that defines how the templates should be combined by inserting values from the
Excel tables in uniquely identified locations.

The tool is ready for use, but should be used with care. Ensure you do not overwrite
the only copy of your perfectly working config.

## Introduction

Upon inspecting a SEPTIC configuration file, you will find that it can be divided into 
segments where some segments are static while others are repeated for several wells 
(or some other entity) with only minor modifications.

For example: The initial `System` section of the SEPTIC config is a static part that only
occurs once. The following `SopcProc` section usually contains a static header followed 
by the definition of a number of `SopcXvr`. The latter part usually consists of many 
repeating values for all wells. Following this, you will normally have one or more 
`DmmyAppl` sections that contain a mixture of common elements and per-well elements, and
similarly for other sections. 

By extracting these segments and placing them into separate template files, where the
repeating parts are replaced by identifier tags, this tool can recombine the templates
into a fully working SEPTIC config. Some key advantages are:
- Changes made to one well can be quickly propagated  to other wells.
- Adding wells to an existing config is as simple as specifying some key information 
for the new wells and re-running tool. 
- Ensuring that a few templates and a table are correct is much easier than ensuring that
all wells are perfectly specified with the correct tags in the final config. This reduces 
the risk of faulty configs which can lead to faulty operation.

## Installation 

Although some may prefer to run the Python script directly, it is highly recommended to
download the last released precompiled executable. Place scg.exe somewhere in your path,
preferably in the directory where you place all your SEPTIC configs. In the following, 
this directory is assumed to be `C:\Appl\SEPTIC`.

## Basic usage

The tool has three commands (or modes of operation):
 - make: Generate complete config file based on templates
 - revert: Regenerate one or more template files based on parts of config files.
 - diff: Simply utility to show difference between two files.  

Type `scg.exe --help` to get basic help information for the tool. You can also get help
for each command, e.g. `scg.exe make --help`.

## Preparation

It is easiest to explain how to use the tool by example. In the file-set you will find a 
directory called `basic example`. This directory contains the following directories and 
files:
- templates: A directory containing the templates that make up a SEPTIC config file.
- example.yaml: Defines how the template files should be combined to create 
example_final.conf
- example.xlsx: An Excel file that contains data to insert into the templates.
- masters: A directory containing segments of the SEPTIC config file that can be reverted 
into templates.
- example.cnfg: The resulting SEPTIC config file. 
 
Download and copy the entire directory called `basic example` to `C:\Appl\SEPTIC`. 

## The template files

Take a look in the templates directory. You will find a number of template files that
can be combined to create a final `example.cnfg`.  

Upon inspecting the files, you will see that some of them contain text within double 
curly braces, e.g. `{{ Id }}`. These are identifier tags for the parts that will be 
replaced.

The files that do not contain tags are static, and will normally be used only once in 
the final config. The files that contain tags are dynamic and will by default be replicated
once for each row that is defined in the Excel file.

Regarding file naming:
- It is not necessary to enumerate the files as is done here, but it may make it easier
to understand the layout of the final config file.
- It is also a good idea, although not required, to indicate in the file names which 
of the files contain parameters for substitution. In the example, those files end 
with `_well`. 
  
## The Excel file

The file `example.xlsx` contains a single worksheet with a simple table. This is the file
from which we will fetch values to insert into the templates. 

The first row contains the identifier tags. These tags correspond to the tags you saw
in curly braces in the template files. The tags are case sensitive.

Each item, in this case each well, is listed in the following rows. The value in the 
first column is a unique identifier for the item, and is not available for substitution 
in the template. The following columns contain the values that will be substituted for 
the item into the tags that correspond with the values in the first row. 

Please note:
- As mentioned, the tag names (first row) are case sensitive. You must ensure that these
are exactly the same as the tags defined in the templates. Any typo will result in an 
error message upon config generation, so no need to worry about broken configs. 
- In order to ensure that id numbers such as '1' and '21' both are displayed with two
digits in the resulting configuration, you should use strings and not numbers in Excel. 
Simply prepend the numbers with by prepending numbers with `'`. So if you want `D{{ Id }}` 
to become `D01` instead of `D1`, you should input `'01` instead of `1` in the Id field.
- The use of formulas and any kind of text formatting is allowed.

## The config file

Inspect the config file `example.yaml`. It starts out by defining a number of paths:
```yaml
outputfile: example_original.cnfg

templatepath: templates
masterpath: masters

masterkey: D02
verifycontent: yes

```
All paths are relative to the directory in which `example.yaml` is found.

The `outputfile` specifies the file which will be generated by the tool. The tool looks
for templates in the `templatepath` directory. 

Ignore `masterkey` and `masterpath` for now - they will be explained later.

When generating a config file, the default behaviour is to present any difference
between a previously generated config file and the new config as a 
[unified diff](https://en.wikipedia.org/wiki/Diff#Unified_format) before asking whether 
it is ok to replace the original. The original config will be renamed with the extension 
'.bak' before being replaced. If, for any reason, you don't want to be bothered with this
question, you can set `verifycontent` to `no` or override by adding the option 
`--no-verify` on the command line. 
  
The next section defines an Excel file that should be used to substitute values in the
template files:
```yaml
sources:
  - filename: example.xlsx
    id: main
    sheet: Sheet1
```
The source references a worksheet called `Sheet1` in the Excel file `example.xlsx`, and
is given a unique id `main`. The path to the file is relative to the directory containing 
`example.yaml`.

If there are other groups of elements that you wish to create templates and substitutions 
for, e.g. two separator trains, or to distinguish between non-similar groups of well such 
as production wells and injection wells, simply create another sheet (in the same or
a new Excel sheet) and define the new source similarly with a unique id.

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
requires at least a filename `name`. If nothing more is specified, the template is assumed
to be static, and simply inserted into the config file.

If a `source` is defined, then the source is used as a look-up table for substitutions
into the config file. By default, the template is generated once per row in the source. 
So the template `03_SopcProc_well.cnfg` will be replicated three times, once for each 
of the rows `D01`, `D02` and `D03` that are specified in the source.

It is possible to specify exactly which rows to include. An example of this is shown for 
`07_DspGroupTables_well.cnfg` which will only be generated for `D01` and `D03`. It is 
also possible to use the keyword `exclude` in the same way to skip specific rows from 
the source.

## Generate a config

Now that you know how the files are used, let's try to generate a config. Start by making a 
copy of `example.cnfg`. Rename the copy to `example_original.cnfg`. Copy scg.exe into your
`Basic Example` directory, open a command line and change directory to `C:\appl\SEPTIC\Basic Example`.

To generate example.cnfg, type:
```
scg.exe make example.yaml
```
You can also type simply
```
scg.exe make example
```
Verify that the generated `example.cnfg` corresponds with the layout defined in yaml and
the rows in the Excel sheet. E.g. you should have SopcCvrs and SopcMvrs defined for all 
three wells, but MPCTable should only list D01 and D02.

Try to make a change to one of the template files and regenerate the config. `Scg` will 
detect that there is already a file called `example.cnfg` and will ask whether you want
to replace it. Changes are shown as unified diff between the two files.  

Type `scg.exe make --help` for more options to the `make` command.

### Global variables

Since version 1.0.0, it is possible to define global replacement variables on the command
line using the `--var` option. The option takes two arguments: Name and value. The name 
can be used in any template file, also files without a defined source, using the same 
format as for variables defined in the Excel sheet (in fact, global variables are simply 
added to set of variables (tags) defined in the source, which may be empty/non-existent.
and replace them if they have the same name). `scg` will try to convert value to boolean, 
integer or float with string as default type.

Example: 
```
scg.exe make --var simulation true --var size 2.3 --var version 1.2.3 example.yaml
```
Here `{{ simulation }}`, `{{ size }}` and `{{ version }}` will be available for use in 
all template files. The value of `simulation` is boolean `True`, `size` is a float with
value `2.3` while `version` is a string with value `1.2.3`.

## Reverting templates

What do you do if you have made a modification to the final config on the server, and 
wish to update your templates to reflect the modifications so that you can continue to 
use this tool? In some cases it can be easiest to manually repeat the same modifications 
to the affected templates as what was done on the server, but this method requires 
meticulous care to ensure that each modification is copied perfectly over. This may
be assisted by modifying templates, running the tool, comparing the output with the
modified config and repeating this process until it matches. 

This tool has another command called `revert` that can be used as an alternative to 
manual patching. It is based on regenerating each affected template based on its 
corresponding segment from the modified config file. 

*Please note that reverting can only convert static values in the master file to corresponding 
identifier tags found in the source file. This means that it is not possible to combine
reverting with more advanced [Jinja2 functionality](#the-template-engine) such as if/then/else, 
calculations etc.* 

For static templates, you should simply copy the entire corresponding content over from
the modified config file to the template. 
 
For dynamic templates, it is possible to revert a segment of the modified config file 
back into a template. In this method we search for the field values from one specific 
row in the source file, and replace the values with the corresponding identifier tags. 

This is done as follows:
- Identify a template which covers the modifications performed on the full config. Copy
a segment from the full config covering the entire template just once into a file in the 
directory specified by `masterpath` in the yaml config. The filename should be the same 
as that of the corresponding template. This is called the master file. If one template 
doesn't cover all modifications, repeat for other templates until all modifications are 
covered. 
- Inspect the source file used for the template, and find a row that contains values
that only exist in places where you wish to have a tag in the template file.
- Declare `masterkey` in the yaml config to identify the row you found in the previous
point. In many cases this can be done globally for all templates by declaring `masterkey`
as shown in the example.yaml file. However, if necessary then you can also declare 
`masterkey` per template to override the global masterkey.
- Run scg with the `revert` command (see `scg.exe revert --help`)
- The template(s) will be regenerated in the templates directory. If `verifycontent` is set
to `yes` in the yaml file, you will be presented with the unified diff between the old and 
the new template file and asked whether you wish to overwrite or keep the old version. A 
backup will be made if you choose to overwrite.

This method requires that the values to be searched for and replaced with identifier tags 
exist only in places in the config file where we wish to have those tags. For example: 
If we wish to regenerate the template file `05_ExprModl_well.cnfg`, then we cannot 
copy-paste the corresponding segment from the config file that was generated for 'D01'. 
Reverting such a master with `masterkey: D01` will replace all occurrences of '01' with
'{{ Id }}', including parts of the strings for `GrpType`, `fulfRescale`, `MeasHighLimit` 
and `MeasLowLimit`. If, on the other hand, we copy-paste the segment that corresponds to
'D02', we see that the string '02' only exist in those places where we wish to insert
'{{ Id }}'. Performing the same consideration for the other identifier tags (`TagId` 
and `GroupMask`), we see that the same is true for those, and it is therefore safe 
to specify `masterkey: D02` in the yaml file.

In some cases you may be unable to find a candidate since each row has at least one value
that exists in places where it shouldn't. This can be circumvented by changing the value
so it includes more of the line where it is used. For instance: It is clearly not safe
to substitute '3' with '{{ Group }}' in the following:
```
  DisplayGroup:  D03Calcs
       GroupNo=  3
      HistSize=  360
```
since that will result in 
```
  DisplayGroup:  D0{{ Group }}Calcs
       GroupNo=  {{ Group }}
      HistSize=  {{ Group }}60
```
The trick is to change the values in the source file from '1', '2', '3', etc, to include 
more of the surrounding text to make them unique. For instance, we can use
'GroupNo=  1', 'GroupNo=  2' and 'GroupNo=  3'. When performing a revert with these
values, you will get
```
  DisplayGroup:  D03Calcs
       {{ Group }}
      HistSize=  360
```
Or, if "=  3" doesn't occur anywhere else in the master file, we could instead have
chosen "=  1", "=  2" and "=  3", which would have resulted in
```
  DisplayGroup:  D03Calcs
       GroupNo{{ GroupNo }}
      HistSize=  360
```

While the reverting method may seem daunting and scary, it is perfectly safe to play 
around with it as long as you have `verifycontent: yes` in your yaml file.

## Special keywords
The following two keywords are available:
 - ```now```: Inserts a datestamp. For info on formatting, see [Jinja2-time](https://pypi.org/project/jinja2-time/)
 - ```gitcommit```: Inserts the GIT commit hash. More info at [Jinja2-git](https://github.com/sobolevn/jinja2-git)

By placing the following line at the top of the first template file, a nicely 
formatted timestamp is inserted at the top of the generated config:
```
// Generated on {% now 'local', '%a %d %b %Y %H:%M:%S' %}
```

## The template engine
The parameter replacement performed by the `make` command uses the 
[Jinja2](https://jinja.palletsprojects.com/) Python module. Jinja2 is a very powerful templating
engine that can do lots more than what has been described above, such as expressions
(e.g. calculate offsets for placing display elements based on well id number), statements
for inheriting or including other template files, conditionals and loops etc. This 
makes it possible to use scg even for SEPTIC configs with non-similar wells, for example. 
For further information, please take a look at the 
[Jinja2 Template Designer Documentation](https://jinja.palletsprojects.com/en/2.11.x/templates/)

