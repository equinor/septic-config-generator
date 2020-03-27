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
