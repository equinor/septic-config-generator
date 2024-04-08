use anyhow::{bail, Context, Result};
use calamine::{open_workbook, CellErrorType, Data, DataType, Reader, Xlsx};
use csv::{self, Trim};
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type DataSourceRows = IndexMap<String, HashMap<String, CtxDataType>>;

pub trait DataSourceReader {
    fn read(&self) -> Result<DataSourceRows>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtxErrorType {
    /// Division by 0 error
    Div0,
    /// Unavailable value error
    NA,
    /// Invalid name error
    Name,
    /// Null value error
    Null,
    /// Number error
    Num,
    /// Invalid cell reference error
    Ref,
    /// Value error
    Value,
    /// Getting data
    GettingData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CtxDataType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(f64),
    Error(CtxErrorType),
    Empty,
}

impl Serialize for CtxDataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Int(value) => serializer.serialize_i64(*value),
            Self::Float(value) | Self::DateTime(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Error(value) => {
                let s = match value {
                    CtxErrorType::Div0 => "#DIV/0!",
                    CtxErrorType::NA => "#N/A",
                    CtxErrorType::Name => "#NAME?",
                    CtxErrorType::Null => "#NULL!",
                    CtxErrorType::Num => "#NUM!",
                    CtxErrorType::Ref => "#REF!",
                    CtxErrorType::Value => "#VALUE!",
                    CtxErrorType::GettingData => "#UNKNOWN!",
                };
                serializer.serialize_str(s)
            }
            Self::Empty => serializer.serialize_unit(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CsvSourceReader {
    file_path: PathBuf,
    delimiter: char,
}

impl CsvSourceReader {
    pub fn new(file_name: &str, relative_root: &Path, delimiter: Option<char>) -> Self {
        Self {
            file_path: relative_root.join(file_name),
            delimiter: delimiter.unwrap_or(';'),
        }
    }
}

impl DataSourceReader for CsvSourceReader {
    fn read(&self) -> Result<DataSourceRows> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .flexible(false)
            .comment(Some(b'#'))
            .trim(Trim::All)
            .from_path(&self.file_path)?;

        let headers = reader.headers()?.clone();

        let mut rows = IndexMap::new();

        for record_result in reader.records() {
            let record = record_result.with_context(|| "Error reading CSV record")?;

            let mut data = HashMap::new();
            for (i, value) in record.iter().enumerate() {
                if let Some(header_field) = headers.get(i) {
                    let converted_value = match value {
                        "" => CtxDataType::Empty,
                        v if v.parse::<i64>().is_ok() => {
                            if v.starts_with('0') && v != "0" {
                                CtxDataType::String(value.to_string())
                            } else {
                                CtxDataType::Int(v.parse().unwrap())
                            }
                        }
                        v if v.parse::<f64>().is_ok() => CtxDataType::Float(v.parse().unwrap()),
                        v if v.replace(',', ".").parse::<f64>().is_ok() => {
                            CtxDataType::Float(v.replace(',', ".").parse().unwrap())
                        }
                        v if v.parse::<bool>().is_ok() => CtxDataType::Bool(v.parse().unwrap()),
                        _ => CtxDataType::String(value.to_string()),
                    };
                    data.insert(header_field.to_string(), converted_value);
                }
            }
            let key = record[0].to_string();
            rows.insert(key, data);
        }
        let first_column_name = headers.get(0).map(ToString::to_string).unwrap_or_default();

        if rows.iter().any(|(column_name, data)| {
            column_name != &first_column_name
                && match data.get(&first_column_name) {
                    Some(CtxDataType::String(val)) => val.trim().is_empty(),
                    _ => true,
                }
        }) {
            bail!("First column must contain strings only");
        }
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
        Self {
            file_path: relative_root.join(file_name),
            sheet: sheet.map(std::borrow::ToOwned::to_owned),
        }
    }
}

impl DataSourceReader for ExcelSourceReader {
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::cast_possible_truncation)]
    fn read(&self) -> Result<DataSourceRows> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)?;
        let sheet = self.sheet.as_ref().unwrap();

        let range = workbook.worksheet_range(sheet)?;

        if range
            .rows()
            .skip(1)
            .any(|row| row[0].get_string().is_none())
        {
            bail!("First column must contain strings only");
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
                            Data::Int(i) => CtxDataType::Int(i),
                            Data::Float(f) => {
                                if (f - f.floor()).abs() < f64::EPSILON {
                                    CtxDataType::Int(f as i64)
                                } else {
                                    CtxDataType::Float(f)
                                }
                            }
                            Data::String(s) => CtxDataType::String(s),
                            Data::Bool(b) => CtxDataType::Bool(b),
                            Data::DateTime(d) => CtxDataType::DateTime(d.as_f64()),
                            Data::Empty => CtxDataType::Empty,
                            Data::Error(e) => match e {
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
                            _ => {
                                panic!("Unhandled datatype for {cell})"); // Should never happen
                            }
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
mod csvtests {
    use super::*;
    use std::io::Write;

    #[test]
    fn csv_parses_text_float_int_zeros() {
        let csv_content = r#"keys;text;float;int;mix;zeros
key1;value1;1.1;1;1.0;0
# Ignore this line
key2;value2;2.2;2;2;00"#;
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

        let values = &data["key1"];
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value1".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(1.1)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(1)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Float(1.0)));
        assert_eq!(values.get("zeros"), Some(&CtxDataType::Int(0)));

        let values = &data["key2"];
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value2".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(2.2)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(2)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Int(2)));
        assert_eq!(
            values.get("zeros"),
            Some(&CtxDataType::String("00".to_string()))
        );
    }

    #[test]
    fn csv_parses_padded_text_float_int() {
        let csv_content = r#"keys ;  text   ;  float ; int  
key1  ;   value1  ;    1.1 ; 1  "#;
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

        let values = &data["key1"];
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value1".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(1.1)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(1)));
    }

    // Currently doesn't distinguish between 1.234 and "1.234" and turn both into float,
    // thus failing test.
    #[test]
    #[ignore]
    fn csv_parses_quoted_number_as_text() {
        let csv_content = r#"keys;text;textfloat;textint
    key1;value1;"1.234";1.234"#;

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

        let values = &data["key1"];
        assert_eq!(
            values.get("textfloat"),
            Some(&CtxDataType::String("1.234".to_string()))
        );
        assert_eq!(
            values.get("textint"),
            Some(&CtxDataType::String("1".to_string()))
        );
    }

    #[test]
    fn csv_parses_empty_cell_as_empty() {
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

        let values = &data["key1"];
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
    fn csv_read_errors_on_invalid_row() -> Result<()> {
        let csv_content = r#"keys;header1;header2
key1;value1a;value1b
key2;value2a"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{}", csv_content).unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .root_cause()
            .to_string()
            .contains("CSV error: record 2"));
        Ok(())
    }

    #[test]
    fn csv_read_errors_on_first_column_not_string() {
        let csv_content = r#"keys;text;float;int
key1;value1;1.1;1
2;value2;2.2;2"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{}", csv_content).unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("First column must contain strings only"));
    }

    #[test]
    fn csv_read_errors_on_first_column_empty_string() {
        let csv_content = r#"keys;text;float;int
key1;value1;1.1;1
  ;value2;2.2;2"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{}", csv_content).unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("First column must contain strings only"));
    }
}

#[cfg(test)]
mod xlsxtests {
    use super::*;

    #[test]
    fn xlsx_read_errors_on_missing_source_file() {
        let reader =
            ExcelSourceReader::new("nonexistent_file.xlsx", Path::new("./"), Some("mysheet"));

        let result = reader.read();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2"),);
    }

    #[test]
    fn xlsx_read_errors_on_missing_sheet() {
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
            .contains("Worksheet 'nonexistent_sheet' not found"));
    }
}
