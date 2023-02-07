use crate::{CtxDataType, CtxErrorType, DataSourceRow};
use calamine::{open_workbook, CellErrorType, DataType, Reader, Xlsx};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

pub fn read(file: &PathBuf, sheet: &String) -> Result<DataSourceRow, Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    let range = workbook
        .worksheet_range(sheet)
        .ok_or_else(|| format!("Cannot find sheet '{sheet}'"))??;

    let row_headers = range.rows().next().unwrap();
    let data = range
        .rows()
        .skip(1)
        .map(|row| {
            let key = row[0].get_string().unwrap().to_string();
            let values = row_headers
                .iter()
                .zip(row.iter())
                .map(|(header, cell)| {
                    let header_str = header.get_string().unwrap().to_string();
                    let value = match cell.to_owned() {
                        DataType::Int(i) => CtxDataType::Int(i),
                        DataType::Float(f) => match f == f.floor() {
                            true => CtxDataType::Int(f as i64),
                            false => CtxDataType::Float(f),
                        }, // TODO: This is hacky. Why doesn't Calamine parse ints as Int?
                        DataType::String(s) => CtxDataType::String(s),
                        DataType::Bool(b) => CtxDataType::Bool(b),
                        DataType::DateTime(d) => CtxDataType::DateTime(d),
                        DataType::Empty => CtxDataType::Empty,
                        DataType::Error(e) => match e {
                            CellErrorType::Div0 => CtxDataType::Error(CtxErrorType::Div0),
                            CellErrorType::NA => CtxDataType::Error(CtxErrorType::NA),
                            CellErrorType::Name => CtxDataType::Error(CtxErrorType::Name),
                            CellErrorType::Null => CtxDataType::Error(CtxErrorType::Null),
                            CellErrorType::Num => CtxDataType::Error(CtxErrorType::Num),
                            CellErrorType::Ref => CtxDataType::Error(CtxErrorType::Ref),
                            CellErrorType::Value => CtxDataType::Error(CtxErrorType::Value),
                            CellErrorType::GettingData => {
                                CtxDataType::Error(CtxErrorType::GettingData)
                            }
                        },
                    };
                    (header_str, value)
                })
                .collect::<HashMap<String, CtxDataType>>();
            (key, values)
        })
        .collect::<DataSourceRow>();

    Ok(data)
}
