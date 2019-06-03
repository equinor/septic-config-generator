import os
import re
import sys
import logging
import click
import difflib
import shutil
from jinja2 import Environment, FileSystemLoader
from helpers.config_parser import parse_config, patch_config
from helpers.helpers import get_all_source_data
from helpers.version import __version__

@click.group()
@click.version_option(version=__version__)
def main():
    pass

@main.command()
@click.option('--output', help='name of output file (overrides config option)')
@click.option('--no-check', is_flag=True, default=False, help='do not prompt for verification of output file before overwriting original.')
@click.argument('config_file')
def make(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    sources = get_all_source_data(cfg['sources'], cfg['path']['root'])

    env = Environment(
        loader=FileSystemLoader(searchpath=os.path.join(cfg['path']['root'], cfg['path']['templatepath']))
    )

    original_cnfgfile = cfg['output']
    new_cnfgfile = original_cnfgfile+'.new'
    with open(new_cnfgfile, 'w') as f:
        for template in cfg['layout']:
            temp = env.get_template(template['name'])
            if not 'source' in template:
                f.write(print(temp.render({})))
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

    if os.path.exists(original_cnfgfile):
        backup_cnfgfile = original_cnfgfile + '.bak'
        if cfg['check']:
            diff = diff_cnfgs(original_cnfgfile, new_cnfgfile)
            txt = [line for line in diff]
            if len(txt) > 0:
                print(''.join(txt))
                q = input("Replace original? [Y]es or [N]o: ")
                if len(q) > 0 and q[0].lower() == 'y':
                    backup_cnfgfile = original_cnfgfile+'.bak'
                    if os.path.isfile(backup_cnfgfile):
                        os.remove(backup_cnfgfile)
                    shutil.move(original_cnfgfile, backup_cnfgfile)
                    shutil.move(new_cnfgfile, original_cnfgfile)
                else:
                    os.remove(new_cnfgfile)
            else:
                print("No change. Keeping original config.")
                os.remove(new_cnfgfile)
        else:
            shutil.move(original_cnfgfile, backup_cnfgfile)
            shutil.move(new_cnfgfile, original_cnfgfile)
    else:
        shutil.move(new_cnfgfile, original_cnfgfile)

@main.command()
@click.argument('config_file')
@click.option('--template', default='all', help='name of template file to revert. Default: all.')
def revert(config_file, **kwargs):
    cfg = parse_config(config_file).data

    all_source_data = get_all_source_data(cfg['sources'], cfg['path']['root'])

    master_path = os.path.join(cfg['path']['root'], cfg['templategenerator']['masterpath'])
    masters = [f for f in os.listdir(master_path) if os.path.isfile(os.path.join(master_path, f))]
    template_path = os.path.join(cfg['path']['root'], cfg['path']['templatepath'])

    if kwargs['template'] != 'all' and kwargs['template'] not in masters:
        logger.error(f"Unable to locate '{kwargs['template']}' in {master_path}")

    for filename in masters:
        if kwargs['template'] != 'all' and filename != kwargs['template']:  # TODO: Check extension!
            continue

        if filename not in [x['name'] for x in cfg['layout']]:  # TODO: Check extension!
            logger.error(f"Template file '{kwargs['template']}' is not defined in config file. Don't know what to do with it.")
            continue

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

        f = open(os.path.join(master_path, filename), 'r')

        txt = f.read()
        f.close()
        #print(txt)


        #f = open(os.path.join(template_path, filename), 'w')
        #f.write(txt)
        #f.close()

#@main.command()
#@click.argument('original_config')
#@click.argument('new_config')
def diff_cnfgs(original_config, new_config):
    orig = open(original_config).readlines()
    new = open(new_config).readlines()
    return difflib.unified_diff(orig, new, fromfile=original_config, tofile=new_config)

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