from scg.config_parser import parse_config
from scg.scg import read_source
import os
import re

def main():
    cfg = parse_config('johan_sverdrup.yaml').data
    sources = dict()
    for source in cfg['sources']:
        s = read_source(source, cfg['path']['root'])
        sources[source['id']] = s

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

if '__main__' == __name__:
    main()
