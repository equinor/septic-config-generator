import sys
import logging
from strictyaml import Map, Decimal, Str, Seq, MapPattern, CommaSeparated, Optional, load, Int, Bool

logger = logging.getLogger('scg.'+__name__)

schema_source = Map({
    "id": Str(),
    "filename": Str(),
    "sheet": Str(),
    Optional("type"): Str(),
    Optional("safe_reverse"): Seq(Str())
})

schema_sources = Seq(schema_source)

schema_path = Map({
    "root": Str(),
    "templatepath": Str()
})

schema_template = Map({
    "name": Str(),
    Optional("source"): Str(),
    Optional("include"): Seq(Str()),
    Optional("exclude"): Seq(Str())
})

schema_templategenerator = Map({
    "masterpath": Str(),
    "outputdir": Str(),
    "master": Str(),
    Optional("includeonly"): Seq(Str())
})

schema = Map({
    "output": Str(),
    "path": schema_path,
    "sources": Seq(schema_source),
    "layout": Seq(schema_template),
    Optional("templategenerator"): schema_templategenerator,
    Optional("check", default=True): Bool()
})


def parse_config(filename):
    if not filename.lower().endswith('.yaml'):
        filename = filename + '.yaml'
    try:
        f = open(filename, 'r')
    except:
        logger.error(f"File not found: '{filename}'.\n Config files need to be on the format 'filename.yaml'.")
        sys.exit(1)
    try:
        cfg = load(f.read(), schema, label=filename)
        return cfg
    except Exception as e:
        logger.error(f"{e}",)
        sys.exit()

def patch_config(cfg, overrides):
    if overrides['output'] is not None:
        cfg['output'] = overrides['output']
    if overrides['no_check']:
        cfg['check'] = False
    return cfg