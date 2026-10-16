use anyhow::{Context, Result, bail};
use calamine::{CellErrorType, Data, DataType, Reader, Xlsx, open_workbook};
use csv::{self, Trim};
use indexmap::IndexMap;
use minijinja::Value;
use std::path::{Path, PathBuf};

/// Source rows keyed by row label, each row a minijinja map of column header to cell value.
pub type DataSourceRows = IndexMap<String, Value>;

/// Rows with their columns still separate, so that columns from several files can be merged.
type ColumnRows = IndexMap<String, IndexMap<String, Value>>;

pub trait DataSourceReader {
    fn read(&self) -> Result<DataSourceRows>;
}

fn into_value_rows(rows: ColumnRows) -> DataSourceRows {
    rows.into_iter()
        .map(|(key, row)| (key, Value::from_pairs(row)))
        .collect()
}

fn json_number_to_value(number: &serde_json::Number) -> Value {
    match number.as_i64() {
        Some(value) => Value::from(value),
        None => Value::from(number.as_f64().unwrap()),
    }
}

fn csv_cell_to_value(value: &str) -> Value {
    match value {
        "" => Value::from(()),
        v if v.starts_with('0') && v.len() > 1 && v.bytes().all(|b| b.is_ascii_digit()) => {
            Value::from(v)
        }
        v if let Ok(i) = v.parse::<i64>() => Value::from(i),
        v if let Ok(f) = v.parse::<f64>() => Value::from(f),
        v if let Ok(f) = v.replace(',', ".").parse::<f64>() => Value::from(f),
        v if let Ok(b) = v.parse::<bool>() => Value::from(b),
        v if let Ok(values) = serde_json::from_str::<Vec<serde_json::Number>>(v) => {
            Value::from(values.iter().map(json_number_to_value).collect::<Vec<_>>())
        }
        v => Value::from(v),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn excel_cell_to_value(cell: &Data) -> Value {
    match cell {
        Data::Int(i) => Value::from(*i),
        Data::Float(f) => {
            if (f - f.floor()).abs() < f64::EPSILON {
                Value::from(*f as i64)
            } else {
                Value::from(*f)
            }
        }
        Data::String(s) => Value::from(s.as_str()),
        Data::Bool(b) => Value::from(*b),
        Data::DateTime(d) => Value::from(d.as_f64()),
        Data::Empty => Value::from(()),
        Data::Error(e) => Value::from(match e {
            CellErrorType::Div0 => "#DIV/0!",
            CellErrorType::NA => "#N/A",
            CellErrorType::Name => "#NAME?",
            CellErrorType::Null => "#NULL!",
            CellErrorType::Num => "#NUM!",
            CellErrorType::Ref => "#REF!",
            CellErrorType::Value => "#VALUE!",
            CellErrorType::GettingData => "#UNKNOWN!",
        }),
        _ => panic!("Unhandled datatype for {cell})"), // Should never happen
    }
}

#[derive(Debug, Default)]
pub struct MultiSourceReader {
    file_paths: Vec<PathBuf>,
    delimiter: char,
}

impl MultiSourceReader {
    pub fn new(file_names: Vec<&str>, relative_root: &Path, delimiter: Option<char>) -> Self {
        Self {
            file_paths: file_names.iter().map(|f| relative_root.join(f)).collect(),
            delimiter: delimiter.unwrap_or(';'),
        }
    }
}

impl DataSourceReader for MultiSourceReader {
    fn read(&self) -> Result<DataSourceRows> {
        let mut rows = IndexMap::new();
        if let Some((first, rest)) = self.file_paths.split_first() {
            let first_reader = CsvSourceReader {
                file_path: first.to_owned(),
                delimiter: self.delimiter,
            };
            rows = first_reader
                .read_columns()
                .with_context(|| format!("Problem reading source file '{}'", first.display()))?;

            for file_path in rest {
                let reader = CsvSourceReader {
                    file_path: file_path.to_owned(),
                    delimiter: self.delimiter,
                };
                let new_rows = reader.read_columns().with_context(|| {
                    format!("Problem reading source file '{}'", file_path.display())
                })?;

                for key in rows.keys() {
                    if !new_rows.contains_key(key) {
                        anyhow::bail!(
                            "Key '{}' from '{}' is missing in '{}'",
                            key,
                            first.file_name().unwrap().to_string_lossy(),
                            file_path.file_name().unwrap().to_string_lossy()
                        );
                    }
                }

                for (key, values) in new_rows {
                    if let Some(row) = rows.get_mut(&key) {
                        for (col, value) in values {
                            row.insert(col, value);
                        }
                    } // TODO: warn if first csv does not contain key?
                }
            }
        }
        Ok(into_value_rows(rows))
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

    fn read_columns(&self) -> Result<ColumnRows> {
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

            let data: IndexMap<String, Value> = headers
                .iter()
                .zip(record.iter())
                .map(|(header, value)| (header.to_string(), csv_cell_to_value(value)))
                .collect();
            let key = record[0].to_string();
            rows.insert(key, data);
        }
        let first_column_name = headers.get(0).map(ToString::to_string).unwrap_or_default();

        if rows.iter().any(|(column_name, data)| {
            column_name != &first_column_name
                && data
                    .get(&first_column_name)
                    .and_then(Value::as_str)
                    .is_none_or(|val| val.trim().is_empty())
        }) {
            bail!("First column must contain strings only");
        }
        Ok(rows)
    }
}

impl DataSourceReader for CsvSourceReader {
    fn read(&self) -> Result<DataSourceRows> {
        self.read_columns().map(into_value_rows)
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
                let values = row_headers.iter().zip(row.iter()).map(|(header, cell)| {
                    let header_str = header.get_string().unwrap().to_string();
                    (header_str, excel_cell_to_value(cell))
                });
                (key, Value::from_pairs(values))
            })
            .collect::<DataSourceRows>();
        Ok(data)
    }
}

#[cfg(test)]
fn cell(rows: &DataSourceRows, key: &str, column: &str) -> String {
    format!("{:?}", rows[key].get_attr(column).unwrap())
}

#[cfg(test)]
mod multisourcetests {
    use super::*;
    use std::io::Write;

    #[test]
    fn multi_merges_same_keys() {
        let csv_content1 = r#"keys;text
        key1;value1
        key2;value2"#;
        let csv_content2 = r#"keys;float;int
        key2;2.2;2
        key1;1.1;1"#;
        let mut tmp_file1 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file1, "{csv_content1}").unwrap();
        let mut tmp_file2 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file2, "{csv_content2}").unwrap();

        let reader = MultiSourceReader::new(
            vec![
                tmp_file1.path().to_str().unwrap(),
                tmp_file2.path().to_str().unwrap(),
            ],
            std::path::Path::new(""),
            Some(';'),
        );
        let result = reader.read();
        assert!(result.is_ok());
        let data = result.unwrap();
        println!("{data:?}");
        assert_eq!(data.len(), 2);

        assert_eq!(cell(&data, "key1", "text"), "'value1'");
        assert_eq!(cell(&data, "key1", "float"), "1.1");
        assert_eq!(cell(&data, "key1", "int"), "1");
        assert_eq!(cell(&data, "key2", "text"), "'value2'");
        assert_eq!(cell(&data, "key2", "float"), "2.2");
        assert_eq!(cell(&data, "key2", "int"), "2");
    }

    #[test]
    fn multi_ignores_new_keys() {
        let csv_content1 = r#"keys;text
        key1;value1
        key2;value2"#;
        let csv_content2 = r#"keys;float;int
        key2;2.2;2
        key1;1.1;1
        key3;3.3;3"#;
        let mut tmp_file1 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file1, "{csv_content1}").unwrap();
        let mut tmp_file2 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file2, "{csv_content2}").unwrap();

        let reader = MultiSourceReader::new(
            vec![
                tmp_file1.path().to_str().unwrap(),
                tmp_file2.path().to_str().unwrap(),
            ],
            std::path::Path::new(""),
            Some(';'),
        );
        let result = reader.read();
        println!("{result:?}");
        assert!(result.is_ok());
        let data = result.unwrap();
        println!("{data:?}");
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn multi_bails_on_missing_keys() {
        let csv_content1 = r#"keys;text
        key1;value1
        key2;value2
        key3;value3"#;
        let csv_content2 = r#"keys;float;int
        key1;1.1;1
        key3;3.3;3"#;
        let mut tmp_file1 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file1, "{csv_content1}").unwrap();
        let mut tmp_file2 = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file2, "{csv_content2}").unwrap();

        let reader = MultiSourceReader::new(
            vec![
                tmp_file1.path().to_str().unwrap(),
                tmp_file2.path().to_str().unwrap(),
            ],
            std::path::Path::new(""),
            Some(';'),
        );
        let result = reader.read();
        println!("{result:?}");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .root_cause()
                .to_string()
                .contains("Key 'key2' from")
        )
    }
}

#[cfg(test)]
mod csvtests {
    use super::*;
    use std::io::Write;

    #[test]
    fn csv_rows_preserve_column_order() {
        let csv_content = "keys;zeta;alpha;empty;list\nkey1;1;text;;[1, 2.5]";
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();
        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let rows = reader.read().unwrap();

        let columns: Vec<String> = rows["key1"]
            .try_iter()
            .unwrap()
            .map(|k| k.to_string())
            .collect();
        assert_eq!(columns, ["keys", "zeta", "alpha", "empty", "list"]);
    }

    #[test]
    fn csv_parses_text_float_int_zeros_and_numeric_sequences() {
        let csv_content = r#"keys;text;float;int;mix;zeros;long_zeros;numbers;single_number;empty_numbers;not_numbers
    key1;value1;1.1;1;1.0;0;0000000000011000000000000000001;[1, 2.5, 3];[20];[];[a, b]
# Ignore this line
    key2;value2;2.2;2;2;00;001;[4, 5];[20];[];[x]"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 2);

        assert_eq!(cell(&data, "key1", "text"), "'value1'");
        assert_eq!(cell(&data, "key1", "float"), "1.1");
        assert_eq!(cell(&data, "key1", "int"), "1");
        assert_eq!(cell(&data, "key1", "mix"), "1.0");
        assert_eq!(cell(&data, "key1", "zeros"), "0");
        assert_eq!(
            cell(&data, "key1", "long_zeros"),
            "'0000000000011000000000000000001'"
        );
        assert_eq!(cell(&data, "key1", "numbers"), "[1, 2.5, 3]");
        assert_eq!(cell(&data, "key1", "single_number"), "[20]");
        assert_eq!(cell(&data, "key1", "empty_numbers"), "[]");
        assert_eq!(cell(&data, "key1", "not_numbers"), "'[a, b]'");

        assert_eq!(cell(&data, "key2", "text"), "'value2'");
        assert_eq!(cell(&data, "key2", "float"), "2.2");
        assert_eq!(cell(&data, "key2", "int"), "2");
        assert_eq!(cell(&data, "key2", "mix"), "2");
        assert_eq!(cell(&data, "key2", "zeros"), "'00'");
        assert_eq!(cell(&data, "key2", "long_zeros"), "'001'");
        assert_eq!(cell(&data, "key2", "numbers"), "[4, 5]");
        assert_eq!(cell(&data, "key2", "single_number"), "[20]");
    }

    #[test]
    fn csv_parses_padded_text_float_int() {
        let csv_content = r#"keys ;  text   ;  float ; int  
key1  ;   value1  ;    1.1 ; 1  "#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 1);

        assert_eq!(cell(&data, "key1", "text"), "'value1'");
        assert_eq!(cell(&data, "key1", "float"), "1.1");
        assert_eq!(cell(&data, "key1", "int"), "1");
    }

    // Currently doesn't distinguish between 1.234 and "1.234" and turn both into float,
    // thus failing test.
    #[test]
    #[ignore]
    fn csv_parses_quoted_number_as_text() {
        let csv_content = r#"keys;text;textfloat;textint
    key1;value1;"1.234";1.234"#;

        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 1);

        assert_eq!(cell(&data, "key1", "textfloat"), "'1.234'");
        assert_eq!(cell(&data, "key1", "textint"), "'1'");
    }

    #[test]
    fn csv_parses_empty_cell_as_empty() {
        let csv_content = r#"keys;header1;header2;header3
    key1;value1;;value3"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.len(), 1);

        assert_eq!(cell(&data, "key1", "header1"), "'value1'");
        assert_eq!(cell(&data, "key1", "header2"), "None");
        assert_eq!(cell(&data, "key1", "header3"), "'value3'");
    }

    #[test]
    fn csv_read_errors_on_invalid_row() -> Result<()> {
        let csv_content = r#"keys;header1;header2
key1;value1a;value1b
key2;value2a"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .root_cause()
                .to_string()
                .contains("CSV error: record 2")
        );
        Ok(())
    }

    #[test]
    fn csv_read_errors_on_first_column_not_string() {
        let csv_content = r#"keys;text;float;int
key1;value1;1.1;1
2;value2;2.2;2"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("First column must contain strings only")
        );
    }

    #[test]
    fn csv_read_errors_on_first_column_empty_string() {
        let csv_content = r#"keys;text;float;int
key1;value1;1.1;1
  ;value2;2.2;2"#;
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        write!(tmp_file, "{csv_content}").unwrap();

        let reader = CsvSourceReader::new(
            tmp_file.path().to_str().unwrap(),
            std::path::Path::new(""),
            Some(';'),
        );

        let result = reader.read();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("First column must contain strings only")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Worksheet 'nonexistent_sheet' not found")
        );
    }
}
