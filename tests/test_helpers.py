import pytest
from scg.helpers.helpers import *


source = {"filename": "example.xlsx", "id": "main", "sheet": "Sheet1"}
root_path = "basic example"


@pytest.mark.skip()
def test_diff_backup_and_replace():
    pass


def test_read_source():
    res = read_source(source, root_path)
    assert res["D01"]["TagId"] == "06"


def test_get_all_source_data():
    sources = [source]
    res = get_all_source_data(sources, root_path)
    assert res["main"]["D01"]["TagId"] == "06"


def test_get_global_variables():
    data = (("int", "2"), ("float", "1.2"), ("str", "1.2.3"), ("bool", "true"))
    res = get_global_variables(data)
    assert len(res) == 4
    for key, value in res.items():
        assert isinstance(value, eval(key))