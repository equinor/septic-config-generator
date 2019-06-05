# SEPTIC config generator

SEPTIC config generator (scg) is a tool to generate SEPTIC configs based on one or more templates,
one or more Excel-tables containing substitution values, and a config file that defines how the templates
should be combined.

## Introduction

The tool has three modes of operation:
 - make: Generate complete config file based on templates
 - revert: Regenerate one or more template files based on parts of config files.
 - diff: Simply utility to show difference between two files.  

## Installation 

Although some may prefer to run the Python script directly, it is highly recommended to
download a precompiled executable from [\\\\statoil.net\dfs\common\TPD\RD_Data\Process Control\Software Tools\SepticConfigGenerator](file://statoil.net/dfs/common/TPD/RD_Data/Process%20Control/Software%20Tools/SepticConfigGenerator)
on G-disk. Place scg.exe somewhere in your path, preferably in the directory where you place all your SEPTIC configs.
In the following, this directory is assumed to be `C:\Appl\SEPTIC`.

## Preparation

It is easiest to explain how to use the tool by example. In the file-set you will find a 
directory called `basic example`. This directory contains the following directories and files:
- templates: A directory containing the templates that make up a SEPTIC config file.
- example.yaml: Defines how the template files should be combined to create example_final.conf
- example.xlsx: An Excel file that contains data to insert into the templates.
- masters: A directory containing segments of the SEPTIC config file that can be reverted into templates.
- example.cnfg The resulting SEPTIC config file. 
 
Download and copy the entire directory called `basic example` to `C:\Appl\SEPTIC`  

## The template files

Take a look in the templates directory. You will find a number of template files that
can be combined to create a final `example.cnfg`.  

Upon inspecting the files, you will see that some of them contain text within double 
curly braces, e.g. `{{ Id }}`. These are the parts that will be replaced.

The files that do not contain tags are static, and will normally be used only once in 
the final config. The files that contain tags are dynamic and will by default be replicated
once for each row that is defined in the Excel file.

Regarding file naming:
- It is not necessary to enumerate the files as is done here, but it may make it easier
to stay on top of the layout of the final config file.
- It is also a good idea to indicate in the file names which of the files contain 
parameters for substitution. In the example, those files end with `_well`. This is 
of course not required.
  
## The Excel file

The file `example.xlsx` contains a single worksheet with a simple table. This is the file
from which we will pick values to insert into the templates. 

The first row contains the substitution tags. These tags correspond to the tags you saw
in curly braces in the template files. Please note that the are case sensitive.

Each item, in this case well, is listed in the following rows. The value in the first
column is a unique identifier for the item, and is not available for substitution. The
following columns contain the values that will be substituted for the item into the
tags that correspond with the values in the first row. 

Please note:
- As mentioned, the tag names (first row) are case sensitive. You must ensure that these
are exactly the same as the tags defined in the templates. However, any typo here will
result in an error message upon config generation.
- All values must be formatted exactly the way you want them to appear in the final config.
In most cases, this means you have to ensure the values are strings, not numbers. In Excel,
this is done by prepending numbers with `'`. So if you want `D{{ Id }}` to become 
`D01` instead of `D1`, you should input `'01` instead of `1` in the Id field.     

## The config file

Inspect the config file `example.yaml`. It starts out by defining a number of paths:
```yaml
outputfile: example.cnfg

templatepath: templates
masterpath: masters

masterkey: D02
verifycontent: yes

```
All paths are relative to the directory in which `example.yaml` is found. 

Ignore `masterkey` for now.

When generating a config file, the default behaviour is to present any difference
between a previously generated config file and the new config before asking whether 
it is ok to replace the original. The original config will be renamed with the extension 
'.bak' before being replaced. If, for any reason, you don't want to be bothered with this
question, you can set `verifycontent` to `no`. 
  
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

If there are more groups of elements that you wish to create templates and substitutions 
for, e.g. two separator trains, or to distinguish between non-similar groups of well such 
as production wells and injection wells, simply create another sheet (in the same or
a new Excel sheet) and define the new source similarly with a unique id.

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
of templates is processed in sequence. Each template reference requires at least a filename 
`name`. If nothing more is specified, the template is a simply inserted into the config file.

If a `source` is defined, then the source is used as a look-up table for substitions
into the config file. By default, the template is generated once per row in the source. 
So the template `03_SopcProc_well.cnfg` will be replicated three times, one for each well 
`D01`, `D02` and `D03`.
It is possible to specify which rows to include. An example of this is shown for `07_DspGroupTables_well.cnfg`
which will only be generated for `D01` and `D03`. It is also possible to use the keyword
`exclude` to skip specific rows from the source.

## Reverting templates

What do you do if you have made a modification to the final config on the server, and wish
to update your templates to reflect the modifications so that you can continue to use
this tool? In some cases it may be easiest to perform the same modifications to the 
templates as what was done on the server, but this method requires meticulous care to ensure
that each modification is copied perfectly over.

There is another method that may be handy in many cases: Reverting a segment back into a
template. *Todo: Describe* 

 

## * * * Scratch * * *

The template files will be combined according to a rule-set to generate the completed config.
Each template file can either contain place-holders for inserting values from the table in the previous step, or it can be a static file.
- Static templates will normally be included only once per complete config file
- Dynamic templates (templates containing place-holders) will normally be generated once per well with the place-holders filled inn accordingly. 

Upon inspecting a complete Septic configuration, you will find that it can be segmented into parts where some parts are static while others are repeated for several wells (or some other entity) with only minor modifications.

For example: The initial `System` section of the Septic config is a static part, and should be separated into one template file.
The following `SopcProc` section usually contains a static header followed by the definition of a number of `SopcXvr`. The static header should be placed in one template, while the rest should be made into a dynamic template.
Following this, you will normally have one or more `DmmyAppl` sections that contain a mixture of common elements and per-well elements. These should be separated similarly to the `SopcProc` section.

If you already have a Septic configuration that you wish to convert into templates to be used with this tool, you should begin by restructuring the file so that static and dynamic parts are clearly and logically separated.
 
Place all template files into the templates-directory. Make sure they have the regular SEPTIC config file extension `.cnfg`.

Wherever you want to insert a value for substitution, enter the column name from the substitution table in double curly braces, e.g. ```{{ Id }}```. These identifiers are case-sensitive, so make sure you use the exact same label as in the table.

Here is an example of a dynamic template file:
```
  SopcMvr:       D{{ Id }} Zpc
         Text1=  "D{{ Id }}: Production choke"
         Text2=  ""
        MvrTag=  "D{{ Id }}Zpc"
       MeasTag=  "MOD.D-13UIC{{ Id }}01/ZPC_YXSP/PRIM"
         PVTag=  "MOD.D-13UIC{{ Id }}01/ZPC_YX/PRIM"

  SopcCvr:       D{{ Id }}Pbh
         Text1=  "D{{ Id }}: Bottom hole pressure"
         Text2=  ""
        CvrTag=  "D{{ Id }}Pbh "
       MeasTag=  "MOD.D-13UIC{{ Id }}04/PBH_YX/PRIM"
         IdTag=  ""
         SpTag=  "MOD.D-13UIC{{ Id }}04/PBH_YR/PRIM"

```
We want the contents of this file to be copied into the final configuration file three
times, one for each of the wells. The value `{{ Id }}` will be substituted by the 
corresponding values from the Excel table.
 




