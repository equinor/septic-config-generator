use crate::{CtxDataType, CtxErrorType};
use calamine::{open_workbook, CellErrorType, DataType, Reader, Xlsx};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
pub type DataSourceRows = Vec<(String, HashMap<String, CtxDataType>)>;

pub trait DataSourceReader {
    fn read(&self) -> Result<DataSourceRows, Box<dyn Error>>;
}

pub struct ExcelSourceReader {
    file_path: PathBuf,
    sheet: Option<String>,
}

impl ExcelSourceReader {
    pub fn new(file_name: &str, relative_root: &Path, sheet: Option<&str>) -> Self {
        ExcelSourceReader {
            file_path: relative_root.join(file_name),
            sheet: sheet.map(|s| s.to_owned()),
        }
    }
}

impl DataSourceReader for ExcelSourceReader {
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::cast_possible_truncation)]
    fn read(&self) -> Result<DataSourceRows, Box<dyn Error>> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)?;
        let sheet = self.sheet.as_ref().unwrap();

        let range = workbook
            .worksheet_range(sheet)
            .ok_or_else(|| format!("Cannot find sheet '{sheet}'"))??;

        if range
            .rows()
            .skip(1)
            .any(|row| row[0].get_string().is_none())
        {
            return Err("First column must contain strings only".into());
        }

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
                        let value = match cell.clone() {
                            DataType::Int(i) => CtxDataType::Int(i),
                            DataType::Float(f) => {
                                if (f - f.floor()).abs() < f64::EPSILON {
                                    CtxDataType::Int(f as i64)
                                } else {
                                    CtxDataType::Float(f)
                                }
                            }
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
            .collect::<DataSourceRows>();
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_source_file_does_not_exist() {
        let reader =
            ExcelSourceReader::new("nonexistent_file.xlsx", Path::new("./"), Some("mysheet"));

        let result = reader.read();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2"),);
    }

    #[test]
    fn test_read_source_file_sheet_does_not_exist() {
        let reader = ExcelSourceReader::new(
            "test.xlsx",
            Path::new("tests/testdata"),
            Some("nonexistent_sheet"),
        );

        let result = reader.read();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot find sheet"));
    }
}
