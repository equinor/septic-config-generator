use crate::{CtxDataType, CtxErrorType};
use calamine::{open_workbook, CellErrorType, DataType, Reader, Xlsx};
use csv;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
pub type DataSourceRows = Vec<(String, HashMap<String, CtxDataType>)>;

pub trait DataSourceReader {
    fn read(&self) -> Result<DataSourceRows, Box<dyn Error>>;
}

#[derive(Debug, Default)]
pub struct CsvSourceReader {
    file_path: PathBuf,
    delimiter: char,
}

impl CsvSourceReader {
    pub fn new(file_name: &str, relative_root: &Path, delimiter: Option<char>) -> Self {
        CsvSourceReader {
            file_path: relative_root.join(file_name),
            delimiter: delimiter.unwrap_or(';'),
        }
    }
}

impl DataSourceReader for CsvSourceReader {
    fn read(&self) -> Result<DataSourceRows, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .flexible(false)
            .comment(Some(b'#'))
            .from_path(&self.file_path)?;

        let headers = reader.headers()?.clone();

        let rows = reader
            .records()
            .enumerate()
            .map(|(_, record_result)| {
                let record =
                    record_result.map_err(|e| format!("Error reading CSV record: {}", e))?;

                let mut data = HashMap::new();
                for (i, value) in record.iter().enumerate() {
                    if let Some(header_field) = headers.get(i) {
                        let converted_value = match value {
                            "" => CtxDataType::Empty,
                            _ => {
                                if let Ok(int_value) = value.parse::<i64>() {
                                    CtxDataType::Int(int_value)
                                } else if let Ok(float_value) = value.parse::<f64>() {
                                    CtxDataType::Float(float_value)
                                } else if let Ok(bool_value) = value.parse::<bool>() {
                                    CtxDataType::Bool(bool_value)
                                } else {
                                    CtxDataType::String(value.to_string())
                                }
                            }
                        };
                        data.insert(header_field.to_string(), converted_value);
                    }
                }
                Ok::<_, Box<dyn Error>>((record[0].to_string(), data))
            })
            .collect::<Result<_, Box<dyn Error>>>()?;

        Ok(rows)
    }
}

#[derive(Debug)]
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
    use std::io::Write;

    #[test]
    fn test_csv_source_reader() {
        let csv_content = r#"keys;text;float;int;mix
key1;value1;1.1;1;1.0
# Ignore this line
key2;value2;2.2;2;2"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{}", csv_content).unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 2);

        let (header, values) = &data[0];
        assert_eq!(header, "key1");
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value1".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(1.1)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(1)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Float(1.0)));

        let (header, values) = &data[1];
        assert_eq!(header, "key2");
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value2".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(2.2)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(2)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Int(2)));
    }


    #[test]
    fn test_csvsource_empty_cell_is_empty() {
        let csv_content = r#"keys;header1;header2;header3
    key1;value1;;value3"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{}", csv_content).unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 1);

        let (_, values) = &data[0];
        assert_eq!(
            values.get("header1"),
            Some(&CtxDataType::String("value1".to_string()))
        );
        assert_eq!(values.get("header2"), Some(&CtxDataType::Empty));
        assert_eq!(
            values.get("header3"),
            Some(&CtxDataType::String("value3".to_string()))
        );
    }

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
