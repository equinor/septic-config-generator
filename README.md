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
on G-disk. Place scg.exe somewhere in your path, or perhaps better: in the directory where you place all your SEPTIC configs,
e.g. `C:\Appl\SEPTIC`.

## Preparation

It is easiest to explain how to use the tool by example. In the file-set you will find a 
directory called `basic example`. This directory contains a simplified set of files necessary 
to create a final configuration file.   
 
### Create directory structure

The tool relies on the following directory structure:
```commandline
root
root/templates
root/masters
```
The root directory should be called something that makes sense to you, e.g. the name of the asset. Here we will simply 
call it `basic example`. That gives the following directory structure:   

```commandline
C:\Appl\SEPTIC\basic example
C:\Appl\SEPTIC\basic example\templates
C:\Appl\SEPTIC\basic example\masters
```

All template files that will eventually make up the complete Septic configuration must be placed in the `templates`-directory. 

### Create substitution table

We need to create one or more tables to hold the values we wish to substitute into our 
templates in order to generate a final configuration file. In this example we assume that 
there are three wells that will be configured almost the same, except for a few parameters: 
An id number, a group mask and a group number. We therefore start by creating an Excel file 
with a single worksheet that defines these parameters. It can look something like this:

| Well | Id | GroupMask                       | GroupNo     |
|------|----|---------------------------------|-------------|
| D01  | 01 | 0000000000000000000000100000000 | GroupNo=  1 |
| D02  | 02 | 0000000000000000000001000000000 | GroupNo=  2 |
| D03  | 03 | 0000000000000000000010000000000 | GroupNo=  3 |

Some key elements: 
- The first column (well) must contain a unique identifier for the row.
- The header row must contain unique identifiers (labels) for each element that will be substituted into one or more template files.
- The header row labels are case sensitive.
- All values should be text (e.g. type `'01` instead of `1` for the D01 Id.)  
- It is a good idea to make sure that there is at least one row that contains values that are absolutely unique within each template file. This will be explained later.

If there are more groups of elements that you wish to create templates and substitutions for, 
e.g. separator trains, or to distinguish between non-similar groups of well such as production
wells and injection wells, simply create another sheet and follow the same structure. 

### Create template files

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
       MeasTag=  "MOD.D-13UIC0904/PBH_YX/PRIM"
         IdTag=  ""
         SpTag=  "MOD.D-13UIC0904/PBH_YR/PRIM"

```
We want the contents of this file to be copied into the final configuration file three
times, one for each of the wells. The value `{{ Id }}` will be substituted with the 
corresponding values from the Excel table.
 




