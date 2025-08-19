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
    // Открываем рабочую книгу
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("cannot open workbook '{path}'"))?;

    // Получаем имя первого листа
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .context("workbook has no sheets")?;

    // Получаем диапазон данных
    let range = workbook
        .worksheet_range(&sheet_name)
        .with_context(|| format!("cannot find sheet '{sheet_name}'"))?;

    // Проверяем, что диапазон не пустой
    if range.is_empty() {
        println!("Range is empty: returning empty items");
        return Ok(Vec::new());
    }

    // Выводим размеры диапазона для отладки
    println!(
        "Range dimensions: rows={}, cols={}",
        range.rows().len(),
        range.width()
    );

    // Проверяем, что диапазон содержит достаточно строк и столбцов
    if range.rows().len() < 4 || range.width() < 10 {
        println!(
            "Invalid range: rows={} or cols={} too small",
            range.rows().len(),
            range.width()
        );
        return Ok(Vec::new());
    }

    let mut items = Vec::new();

    // Итерируем по строкам, начиная с 4-й (индекс 3)
    for (row_idx, row) in range.rows().skip(3).enumerate() {
        // Безопасно берем первые 10 столбцов
        let row_safe: Vec<&Data> = row.iter().take(10).collect();
        println!("Row {}: {:?}", row_idx + 4, row_safe);

        // Проверяем, что строка содержит достаточно столбцов
        if row_safe.len() < 5 {
            println!(
                "Skipping row {}: insufficient columns ({})",
                row_idx + 4,
                row_safe.len()
            );
            continue;
        }

        // Безопасный доступ к ячейкам
        let name_dt = row_safe.get(1).copied().unwrap_or(&Data::Empty); // Столбец B (индекс 1)
        let e_dt = row_safe.get(4).copied().unwrap_or(&Data::Empty); // Столбец E (индекс 4)

        // Преобразуем данные в строку
        let name = data_to_string(name_dt);

        // Определяем флаг is_removed
        let is_removed = match e_dt {
            Data::Empty => false,
            Data::String(s) if s.trim().is_empty() => false,
            _ => true,
        };

        // Пропускаем полностью пустые строки
        if name.is_empty() && !is_removed {
            continue;
        }

        items.push(Item {
            name,
            embedding: Vec::new(),
            status: status.to_string(),
            is_removed,
        });
    }

    Ok(items)
}
