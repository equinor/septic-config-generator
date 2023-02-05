use calamine::{open_workbook, CellErrorType, DataType, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(remote = "CellErrorType")]
enum CellErrorTypeDef {
    Div0,
    NA,
    Name,
    Null,
    Num,
    Ref,
    Value,
    GettingData,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "DataType")]
pub enum DataTypeSer {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(f64),
    #[serde(with = "CellErrorTypeDef")]
    Error(CellErrorType),
    Empty,
}

impl Serialize for DataTypeSer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DataTypeSer::Int(value) => serializer.serialize_i64(*value),
            DataTypeSer::Float(value) => serializer.serialize_f64(*value),
            DataTypeSer::String(value) => serializer.serialize_str(value),
            DataTypeSer::Bool(value) => serializer.serialize_bool(*value),
            DataTypeSer::DateTime(value) => serializer.serialize_f64(*value),
            DataTypeSer::Error(value) => serializer.serialize_str(&value.to_string()[..]),
            // DataTypeSer::Error(_) => serializer.serialize_str("Error in cell"), // Do I need to handle this as Err or just return a special value?
            DataTypeSer::Empty => serializer.serialize_unit(),
        }
    }
}

pub type RowItem = Vec<(String, HashMap<String, DataTypeSer>)>;

pub fn read(file: &PathBuf, sheet: &String) -> Result<RowItem, Box<dyn Error>> {
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
                        DataType::Int(i) => DataTypeSer::Int(i),
                        DataType::Float(f) => DataTypeSer::Float(f),
                        DataType::String(s) => DataTypeSer::String(s),
                        DataType::Bool(b) => DataTypeSer::Bool(b),
                        DataType::DateTime(d) => DataTypeSer::DateTime(d),
                        // DataType::Error(e) => MyDataType::Error(e),
                        DataType::Empty => DataTypeSer::Empty,
                        _ => DataTypeSer::String("Her var det en feil".to_string()),
                    };
                    (header_str, value)
                })
                .collect::<HashMap<String, DataTypeSer>>();
            (key, values)
        })
        .collect::<RowItem>();

    Ok(data)
}
