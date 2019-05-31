import os
from jinja2 import Environment, FileSystemLoader
from helpers.config_parser import parse_config
from helpers.helpers import read_source
from helpers.version import __version__

def main(config):
    cfg = parse_config(config).data

    env = Environment(
        loader=FileSystemLoader(searchpath=os.path.join(cfg['path']['root'], cfg['path']['templatepath']))
    )
    sources = dict()
    for source in cfg['sources']:
        s = read_source(source, cfg['path']['root'])
        sources[source['id']] = s

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

if __name__ == '__main__':
    main('johan_sverdrup.yaml')