import os
import sys
import click
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
@click.argument('config_file')
def make(config_file, **kwargs):
    file_cfg = parse_config(config_file).data
    cfg = patch_config(file_cfg, kwargs)

    sources = get_all_sources(cfg['sources'], cfg['path']['root'])

    env = Environment(
        loader=FileSystemLoader(searchpath=os.path.join(cfg['path']['root'], cfg['path']['templatepath']))
    )

    with open(cfg['output'], 'w') as f:
        for template in cfg['layout']:
            temp = env.get_template(template['name'])
            if not 'source' in template:
                print(temp.render({}))
                continue
            if 'include' in template:
                items = template['include']
            else:
                items = list(sources[template['source']].keys())
                if 'exclude' in template:
                    items = [x for x in items if x not in template['exclude']]

            for row, values in sources[template['source']].items():
                if row in items:
                    print(temp.render(values))
                    f.write(temp.render(values))

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
        print(txt)
        f = open(os.path.join(master_path, filename), 'r')

        txt = f.read()
        f.close()
        #print(txt)


        #f = open(os.path.join(template_path, filename), 'w')
        #f.write(txt)
        #f.close()

if __name__ == '__main__':
    main()