from strictyaml import Map, Decimal, Str, Seq, MapPattern, CommaSeparated, Optional, load, Int, Bool

schema_source = Map({
    "id": Str(),
    "filename": Str(),
    "sheet": Str(),
    Optional("type"): Str()
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
    Optional("exclude"): Seq(Str()),
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
    Optional("templategenerator"): schema_templategenerator
})


def parse_config(filename):
    try:
        with open(filename, 'r') as f:
            cfg = load(f.read(), schema, label=filename)
    except:
        raise FileNotFoundError
    return cfg