import os
import re
import sys
import logging
import click
import difflib
from jinja2 import Environment, FileSystemLoader, StrictUndefined
from helpers.config_parser import parse_config, patch_config
from helpers.helpers import get_all_source_data, diff_backup_and_replace
from version import version as __version__


@click.group()
@click.version_option(version=__version__)
def main():
    pass


@main.command()
@click.option(
    "--output", help='name of output file (overrides config option "outputfile")'
)
@click.option(
    "--no-verify",
    is_flag=True,
    default=False,
    help="do not prompt for verification of output file before overwriting original (overrides config option).",
)
@click.option(
    "--silent", is_flag=True, default=False, help="only output warnings or errors."
)
@click.argument("config_file")
def make(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    if kwargs["silent"]:
        logger.setLevel(logging.WARNING)

    root_path = os.path.dirname(config_file)

    all_source_data = get_all_source_data(cfg["sources"], root_path)

    env = Environment(
        loader=FileSystemLoader(
            searchpath=os.path.join(root_path, cfg["templatepath"]), encoding="cp1252"
        ),
        keep_trailing_newline=True,
        undefined=StrictUndefined,
        extensions=[
            "jinja2_git.GitExtension",  # Allows {% gitcommit %}
            "jinja2_time.TimeExtension",  # Allows {% now %}
        ],
    )

    original_cnfgfile = os.path.join(root_path, cfg["outputfile"])
    new_cnfgfile = original_cnfgfile + ".new"

    with open(new_cnfgfile, "w") as f:
        for template in cfg["layout"]:
            temp = env.get_template(template["name"])
            if not "source" in template:
                rendered = temp.render({})
                if len(rendered) == 0:
                    continue
                if rendered[-1] != "\n":
                    rendered += "\n"
                f.write(rendered)
                continue
            if "include" in template:
                items = template["include"]
            else:
                items = list(all_source_data[template["source"]].keys())
                if "exclude" in template:
                    items = [x for x in items if x not in template["exclude"]]

            for row, values in all_source_data[template["source"]].items():
                if row in items:
                    rendered = temp.render(values)
                    if rendered[-1] != "\n":
                        rendered += "\n"
                    f.write(rendered)

    diff_backup_and_replace(original_cnfgfile, new_cnfgfile, cfg["verifycontent"])


@main.command()
@click.argument("config_file")
@click.option(
    "--template", default="all", help="name of template file to revert. Default: all."
)
@click.option(
    "--no-verify",
    is_flag=True,
    default=False,
    help="do not prompt for verification of output file before overwriting original (overrides config option).",
)
@click.option(
    "--silent", is_flag=True, default=False, help="only output warnings or errors."
)
def revert(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    if kwargs["silent"]:
        logger.setLevel(logging.WARNING)

    root_path = os.path.dirname(config_file)

    all_source_data = get_all_source_data(cfg["sources"], root_path)

    master_path = os.path.join(root_path, cfg["masterpath"])
    masters = [
        f
        for f in os.listdir(master_path)
        if os.path.isfile(os.path.join(master_path, f))
    ]
    template_path = os.path.join(root_path, cfg["templatepath"])

    if kwargs["template"] != "all" and kwargs["template"] not in masters:
        logger.error(
            f"Unable to locate '{kwargs['template']}' in {master_path}. Exiting."
        )
        sys.exit(1)

    for filename in masters:
        if kwargs["template"] != "all":
            if filename != kwargs["template"]:
                continue
            if filename not in [x["name"] for x in cfg["layout"]]:
                logger.warning(
                    f"Template '{kwargs['template']}' is not defined in config file. Ignoring."
                )
                continue

        original_template = os.path.join(template_path, filename)
        new_template = original_template + ".new"

        for layout_item in cfg["layout"]:
            if layout_item["name"] == filename:
                break

        if not "source" in layout_item:
            logger.warning(
                f"No source defined for {layout_item['name']}. Move this file to templates-dir instead."
            )
            continue

        source_data = all_source_data[layout_item["source"]]
        # Extract the data row to be used for reverse substitution.
        # Masterkey can be overridden per layout item.
        masterkey = layout_item.get("masterkey", cfg.get("masterkey", None))
        if not masterkey:
            logger.error(
                f"No masterkey defined in config file. No idea which row to use for reverse substitution. Exiting"
            )
            sys.exit(1)

        source_data = source_data.get(masterkey, None)
        if not source_data:
            logger.error(
                f"Specified masterkey '{masterkey}' not found in source '{layout_item['source']}'. Exiting."
            )
            sys.exit(1)

        f = open(os.path.join(master_path, filename), "r")
        txt = f.read()
        f.close()

        used_keys = []
        for key, value in source_data.items():
            key = "{{ " + key + " }}"
            txt, num = re.subn(re.escape(value), key, txt)
            if num > 0:
                used_keys.append((value, key, num))
        if len(used_keys) == 0:
            logger.info(f"{filename} substitutions: None")
        else:
            logger.info(f"{filename} substitutions:")
            maxlen = max(
                [(len(x[0]), len(x[1])) for x in [key[0:2] for key in used_keys]]
            )
            for key in used_keys:
                logger.info(
                    f"{key[0]:{maxlen[0]}s} -> {key[1]:{maxlen[1]}s} {'['+str(key[2]):>3s}x]"
                )

        f = open(new_template, "w")
        f.write(txt)
        f.close()

        diff_backup_and_replace(original_template, new_template, cfg["verifycontent"])


@main.command()
@click.argument("originalfile")
@click.argument("newfile")
def diff(originalfile, newfile):
    orig = open(originalfile).readlines()
    new = open(newfile).readlines()
    diff = difflib.unified_diff(orig, new, fromfile=originalfile, tofile=newfile)
    txt = [line for line in diff]
    if len(txt) > 0:
        print("".join(txt))


class logFormatter(logging.Formatter):

    FORMATS = {logging.INFO: "%(msg)s", "DEFAULT": "%(levelname)s - %(msg)s"}

    def format(self, record):
        log_fmt = self.FORMATS.get(record.levelno, self.FORMATS["DEFAULT"])
        formatter = logging.Formatter(log_fmt)
        return formatter.format(record)


if __name__ == "__main__":
    logger = logging.getLogger("scg")
    ch = logging.StreamHandler()
    cf = logging.Formatter("%(levelname)s - %(msg)s ")
    cf = logFormatter()
    ch.setFormatter(cf)
    logger.addHandler(ch)
    logger.setLevel(logging.INFO)

    main()
