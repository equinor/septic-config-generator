import sys
import logging
from strictyaml import Map, Str, Seq, Optional, load, Bool

logger = logging.getLogger('scg.'+__name__)

schema_source = Map({
    "id": Str(),
    "filename": Str(),
    "sheet": Str(),
    Optional("type"): Str(),
})

schema_sources = Seq(schema_source)

schema_paths = Map({
    "templatepath": Str(),
    "masterpath": Str()
})

schema_template = Map({
    "name": Str(),
    Optional("source"): Str(),
    Optional("masterkey"): Str(),
    Optional("include"): Seq(Str()),
    Optional("exclude"): Seq(Str())
})

schema_templategenerator = Map({
    "masterkey": Str(),
})

schema = Map({
    "outputfile": Str(),
    "paths": schema_paths,
    "sources": Seq(schema_source),
    "layout": Seq(schema_template),
    Optional("templategenerator"): schema_templategenerator,
    Optional("verifycontent", default=True): Bool()
})


def parse_config(filename):
    if not filename.lower().endswith('.yaml'):
        filename = filename + '.yaml'
    try:
        f = open(filename, 'r')
    except:
        logger.error(f"Config file not found: '{filename}'.")
        sys.exit(1)
    try:
        cfg = load(f.read(), schema, label=filename)
        return cfg
    except Exception as e:
        logger.error(f"{e}",)
        sys.exit()

def patch_config(cfg, overrides):
    if 'output' in overrides and overrides['output'] is not None:
        cfg['output'] = overrides['output']
    if 'no_verify' in overrides and overrides['no_verify']:
        cfg['verifycontent'] = False
    return cfg