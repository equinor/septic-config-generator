use calamine::{open_workbook, DataType, Reader, Xlsx};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

pub fn read(
    file: &PathBuf,
    sheet: &String,
) -> Result<HashMap<String, HashMap<String, DataType>>, Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    let range = workbook
        .worksheet_range(&sheet)
        .ok_or_else(|| format!("Cannot find sheet '{}'", sheet))??;

    let row_headers = range.rows().next().unwrap();
    let data = range
        .rows()
        .skip(1)
        .map(|row| {
            let key = row[0].get_string().unwrap().to_string();
            let values = row_headers
                .iter()
                .zip(row.iter())
                .map(|(header, cell)| (header.get_string().unwrap().to_string(), cell.to_owned()))
                .collect::<HashMap<String, DataType>>();
            (key, values)
        })
        .collect::<HashMap<String, HashMap<String, DataType>>>();

    Ok(data)
}
