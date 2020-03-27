import pytest
from scg.helpers.config_parser import *
import strictyaml


config_file = '..\\basic example\\example.yaml'


def test_parse_config():
    res = parse_config(config_file)
    assert(type(res) == strictyaml.representation.YAML)


def test_patch_config():
    cfg = parse_config(config_file)
    overrides = {"output": "TESTVAL_OUTPUT", "no_verify": False}
    assert(cfg["outputfile"] == 'example.cnfg')
    assert(cfg["verifycontent"] == True)
    res = patch_config(cfg, overrides)
    assert(res["outputfile"] == overrides["output"])
    assert(res["verifycontent"] != overrides["no_verify"])
    

