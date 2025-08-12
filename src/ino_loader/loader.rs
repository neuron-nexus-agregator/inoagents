use crate::ino_loader::model::Item;
use anyhow::{Context, Result};
use calamine::{Data, Reader, open_workbook_auto};

fn data_to_string(dt: &Data) -> String {
    match dt {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if (f.fract()).abs() < f64::EPSILON {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("error:{e}"),
        Data::DateTime(f) => f.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

pub fn load(path: &str, status: &str) -> Result<Vec<Item>> {
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("cannot open workbook '{path}'"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .context("workbook has no sheets")?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .with_context(|| format!("cannot find sheet '{sheet_name}'"))?
        .to_owned();

    let mut items = Vec::new();

    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < 3 {
            continue; // начинаем с B4
        }

        let name_dt = row.get(1).unwrap_or(&Data::Empty); // B
        let e_dt = row.get(4).unwrap_or(&Data::Empty); // E

        let name = data_to_string(name_dt);
        let is_removed = match e_dt {
            Data::Empty => false,
            Data::String(s) if s.trim().is_empty() => false,
            _ => true,
        };

        items.push(Item {
            name,
            embedding: Vec::new(),
            status: status.to_string(),
            is_removed,
        });
    }

    Ok(items)
}
