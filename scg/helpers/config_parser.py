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

schema_template = Map({
    "name": Str(),
    Optional("source"): Str(),
    Optional("masterkey"): Str(),
    Optional("include"): Seq(Str()),
    Optional("exclude"): Seq(Str())
})

schema = Map({
    "outputfile": Str(),
    "templatepath": Str(),
    "masterpath": Str(),
    Optional("masterkey"): Str(),
    "sources": Seq(schema_source),
    "layout": Seq(schema_template),
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
        sys.exit(1)

def patch_config(cfg, overrides):
    if 'output' in overrides and overrides['output'] is not None:
        cfg['outputfile'] = overrides['outputfile']
    if 'no_verify' in overrides and overrides['no_verify']:
        cfg['verifycontent'] = False
    return cfg