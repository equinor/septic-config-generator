import os
import sys
from openpyxl import load_workbook
import logging

logger = logging.getLogger('scg.'+__name__)

def read_source(source, root):
    if not source['filename'].endswith('.xlsx'):
        logging.error(f"Source files need to be xlsx.")
        sys.exit()
    filename = source['filename']
    try:
        wb = load_workbook(os.path.join(root, filename), read_only=True, data_only=True)
    except:
        msg = [f"Unable to open file: {'filename'}\n"]
        msg.append(f" Source files need to exist in the root directory ({root}) and must have an extension '.xlsx'")
        logging.error(''.join(msg))
        sys.exit(1)

    wb = load_workbook(os.path.join(root, source['filename']), read_only=True, data_only=True)
    sheet = wb[source['sheet']]
    rows = sheet.max_row
    cols = sheet.max_column
    headers = dict((i, sheet.cell(row=1, column=i).value) for i in range(1, cols+1))
    ret = dict()
    for i in range(2, rows + 1):
        temp = dict()
        for j in range(2, cols + 1):
            temp[sheet.cell(row=1, column=j).value] = str(sheet.cell(row=i, column=j).value)
            ret[sheet.cell(row=i, column=1).value] = temp
    return ret
    #def item(i, j):
    #    return (sheet.cell(row=1, column=j).value, sheet.cell(row=i, column=j).value)
    #return (dict(item(i, j) for j in range(1, cols + 1 )) for i in range(2, rows + 1))


def get_all_sources(sources, path):
    res = dict()
    for source in sources:
        s = read_source(source, path)
        res[source['id']] = s
    return res
