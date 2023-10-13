use crate::config::Source;
use crate::{CtxDataType, CtxErrorType};
use calamine::{open_workbook, CellErrorType, DataType, Reader, Xlsx};
use csv::{self, Trim};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process;

pub type DataSourceRows = Vec<(String, HashMap<String, CtxDataType>)>;

pub fn read_all_sources(
    sources: Vec<Source>,
    relative_root: &Path,
) -> HashMap<String, DataSourceRows> {
    sources
        .iter()
        .map(|source| {
            let reader = match Path::new(&source.filename).extension() {
                Some(ext) if ext == "xlsx" => {
                    let reader = ExcelSourceReader::new(
                        &source.filename,
                        relative_root,
                        source.sheet.as_deref(),
                    );
                    Box::new(reader) as Box<dyn DataSourceReader>
                }
                Some(ext) if ext == "csv" => {
                    let delimiter = source.delimiter.unwrap_or(';');

                    let reader =
                        CsvSourceReader::new(&source.filename, relative_root, Some(delimiter));
                    Box::new(reader) as Box<dyn DataSourceReader>
                }
                _ => {
                    eprintln!(
                        "Unsupported file extension for source file '{}'",
                        source.filename
                    );
                    process::exit(2);
                }
            };
            let source_data = reader.read().unwrap_or_else(|e| {
                eprintln!("Problem reading source file '{}': {e}", source.filename);
                process::exit(2);
            });
            (source.id.clone(), source_data)
        })
        .collect()
}

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
        Self {
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
            .trim(Trim::All)
            .from_path(&self.file_path)?;

        let headers = reader.headers()?.clone();

        let rows = reader
            .records()
            .enumerate()
            .map(|(_, record_result)| {
                let record = record_result.map_err(|e| format!("Error reading CSV record: {e}"))?;

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
                Ok::<_, Box<dyn Error>>((record[0].to_string(), data))
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>();

        let first_column_name = headers.get(0).map(ToString::to_string).unwrap_or_default();

        if let Ok(rows) = &rows {
            if rows.iter().any(|(column_name, data)| {
                column_name != &first_column_name
                    && match data.get(&first_column_name) {
                        Some(CtxDataType::String(val)) => val.trim().is_empty(),
                        _ => true,
                    }
            }) {
                return Err("First column must contain strings only".into());
            }
        }
        rows
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
                            DataType::Duration(_)
                            | DataType::DateTimeIso(_)
                            | DataType::DurationIso(_) => {
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
    fn csv_parses_text_float_int() {
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

        let (key, values) = &data[0];
        assert_eq!(key, "key1");
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value1".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(1.1)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(1)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Float(1.0)));

        let (key, values) = &data[1];
        assert_eq!(key, "key2");
        assert_eq!(
            values.get("text"),
            Some(&CtxDataType::String("value2".to_string()))
        );
        assert_eq!(values.get("float"), Some(&CtxDataType::Float(2.2)));
        assert_eq!(values.get("int"), Some(&CtxDataType::Int(2)));
        assert_eq!(values.get("mix"), Some(&CtxDataType::Int(2)));
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

        let (key, values) = &data[0];
        assert_eq!(key, "key1");
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

        let (_, values) = &data[0];
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
    fn csv_read_errors_on_invalid_row() {
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
            .to_string()
            .contains("CSV error: record 2"));
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
            .contains("Cannot find sheet"));
    }
}
