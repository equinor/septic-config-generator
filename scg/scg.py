import os
import re
import sys
import logging
import click
import difflib
from jinja2 import Environment, FileSystemLoader
from helpers.config_parser import parse_config, patch_config
from helpers.helpers import get_all_source_data, diff_backup_and_replace
from helpers.version import __version__

@click.group()
@click.version_option(version=__version__)
def main():
    pass

@main.command()
@click.option('--output', help='name of output file (overrides config option)')
@click.option('--no-check', is_flag=True, default=False, help='do not prompt for verification of output file before overwriting original.')
@click.option('--silent', is_flag=True, default=False, help='only output warnings or errors.')
@click.argument('config_file')
def make(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)
    if kwargs['silent']:
        logger.setLevel(logging.WARNING)

    sources = get_all_source_data(cfg['sources'], cfg['path']['root'])

    env = Environment(
        loader=FileSystemLoader(searchpath=os.path.join(cfg['path']['root'],
                                                        cfg['path']['templatepath']),
                                encoding='cp1252'),
        keep_trailing_newline=True
    )

    original_cnfgfile = cfg['output']
    new_cnfgfile = original_cnfgfile+'.new'
    with open(new_cnfgfile, 'w') as f:
        for template in cfg['layout']:
            temp = env.get_template(template['name'])
            if not 'source' in template:
                f.write(temp.render({}))
                if str(temp.module)[-1] != '\n':
                    f.write('\n')
                continue
            if 'include' in template:
                items = template['include']
            else:
                items = list(sources[template['source']].keys())
                if 'exclude' in template:
                    items = [x for x in items if x not in template['exclude']]

            for row, values in sources[template['source']].items():
                if row in items:
                    f.write(temp.render(values))
                    if str(temp.module)[-1] != '\n':
                        f.write('\n')

    diff_backup_and_replace(original_cnfgfile, new_cnfgfile, cfg['check'])

@main.command()
@click.argument('config_file')
@click.option('--template', default='all', help='name of template file to revert. Default: all.')
@click.option('--no-check', is_flag=True, default=False, help='do not prompt for verification of output file before overwriting original.')
@click.option('--silent', is_flag=True, default=False, help='only output warnings or errors.')
def revert(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    if kwargs['silent']:
        logger.setLevel(logging.WARNING)

    all_source_data = get_all_source_data(cfg['sources'], cfg['path']['root'])

    master_path = os.path.join(cfg['path']['root'], cfg['templategenerator']['masterpath'])
    masters = [f for f in os.listdir(master_path) if os.path.isfile(os.path.join(master_path, f))]
    template_path = os.path.join(cfg['path']['root'], cfg['path']['templatepath'])

    if kwargs['template'] != 'all' and kwargs['template'] not in masters:
        logger.error(f"Unable to locate '{kwargs['template']}' in {master_path}")

    for filename in masters:
        if kwargs['template'] != 'all':
            if filename != kwargs['template']:
                continue
            if filename not in [x['name'] for x in cfg['layout']]:
                logger.error(f"Template file '{kwargs['template']}' is not defined in config file. Don't know what to do with it.")
                continue

        original_template = os.path.join(template_path, filename)
        new_template = os.path.join(template_path, filename+'.new')

        for layout_item in cfg['layout']:
            if layout_item['name'] == filename:
                break

        source_data = all_source_data[layout_item['source']]
        # Extract the data row to be used for reverse substitution.
        # Masterkey can be overridden per layout item.
        if 'masterkey' in layout_item:
            masterkey = layout_item['masterkey']
        elif 'masterkey' in cfg['templategenerator']:
            masterkey = cfg['templategenerator']['masterkey']
        else:
            logger.error(f"No master defined in config file. No idea which row to use for reverse substitution. Exiting")
            sys.exit(1)

        if masterkey in source_data:
            source_data = source_data[masterkey]
        else:
            logger.error(f"Unknown master '{masterkey}'. Exiting.")
            sys.exit(1)

        f = open(os.path.join(master_path, filename), 'r')
        txt = f.read()
        f.close()

        used_keys = []
        for key, value in source_data.items():
            key = '{{ '+key+' }}'
            txt, num = re.subn(value, key, txt)
            if num > 0:
                used_keys.append((value, key))
        if len(used_keys) == 0:
            logger.info(f"No substitutions performed in {filename}")
        else:
            logger.info(f"Substitutions in {filename}:")
            for key in used_keys:
                logger.info(f"'{key[0]}' -> '{key[1]}'")

        f = open(new_template, 'w')
        f.write(txt)
        f.close()

        diff_backup_and_replace(original_template, new_template, cfg['check'])

@main.command()
@click.argument('originalfile')
@click.argument('newfile')
def diff(originalfile, newfile):
    orig = open(originalfile).readlines()
    new = open(newfile).readlines()
    diff = difflib.unified_diff(orig, new, fromfile=originalfile, tofile=newfile)
    txt = [line for line in diff]
    if len(txt) > 0:
        print(''.join(txt))

class logFormatter(logging.Formatter):

    FORMATS = {
        logging.INFO: "%(msg)s",
        "DEFAULT": "%(levelname)s [%(name)s] - %(msg)s"
    }

    def format(self, record):
        log_fmt = self.FORMATS.get(record.levelno, self.FORMATS['DEFAULT'])
        formatter = logging.Formatter(log_fmt)
        return formatter.format(record)

if __name__ == '__main__':
    logger = logging.getLogger('scg')
    ch = logging.StreamHandler()
    cf = logging.Formatter("%(levelname)s [%(name)s] - %(msg)s ")
    cf = logFormatter()
    ch.setFormatter(cf)
    logger.addHandler(ch)
    logger.setLevel(logging.INFO)

    main()