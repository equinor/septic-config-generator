import os
import sys
import logging
import click
import difflib
import shutil
from jinja2 import Environment, FileSystemLoader
from helpers.config_parser import parse_config, patch_config
from helpers.helpers import get_all_sources
from helpers.version import __version__

@click.group()
@click.version_option(version=__version__)
def main():
    pass

@main.command()
@click.option('--output', help='name of output file (overrides config option)')
@click.option('--check/--no-check', default=True, help='whether to verify output file before overwriting original.')
@click.argument('config_file')
def make(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    sources = get_all_sources(cfg['sources'], cfg['path']['root'])

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
        if kwargs['check']:
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
        shutil.move(new_cnfgfile, original_cnfgfile)

@main.command()
@click.argument('config_file')
def revert(config_file):
    cfg = parse_config(config_file).data

    sources = get_all_sources(cfg['sources'], cfg['path']['root'])

    master_path = os.path.join(cfg['path']['root'], cfg['templategenerator']['masterpath'])
    masters = [f for f in os.listdir(master_path) if os.path.isfile(os.path.join(master_path, f))]
    template_path = os.path.join(cfg['path']['root'], cfg['path']['templatepath'])

    for filename in masters:
        if 'includeonly' in cfg['templategenerator'] and filename not in cfg['templategenerator']['includeonly']:
            continue
        not_found = True
        for item in cfg['layout']:
            if filename == item['name']:
                not_found = False
                break
        source = sources[item['source']]
        source = source[cfg['templategenerator']['master']]

        f = open(os.path.join(master_path, filename), 'r')
        txt = f.read()

        for key, value in source.items():
            key = '{{ '+key+' }}'
            txt = re.sub(value, key, txt)
        #print(txt)
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

if __name__ == '__main__':
    logger = logging.getLogger('scg')
    ch = logging.StreamHandler()
    cf = logging.Formatter("%(levelname)s [%(name)s] - %(message)s ")
    ch.setFormatter(cf)
    logger.addHandler(ch)

    main()