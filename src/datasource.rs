use calamine::{open_workbook, Reader, Xlsx};
use std::collections::HashMap;
use std::error::Error;

pub fn read(
    filename: &String,
    sheet: &String,
) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn Error>> {
    let path = format!("basic example/{}", filename);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
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
                .map(|(header, cell)| {
                    (
                        header.get_string().unwrap().to_string(),
                        cell.get_string().unwrap().to_string(),
                    )
                })
                .collect::<HashMap<String, String>>();
            (key, values)
        })
        .collect::<HashMap<String, HashMap<String, String>>>();

    Ok(data)
}
