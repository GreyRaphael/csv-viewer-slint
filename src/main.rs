slint::include_modules!();

use slint::{ModelRc, SharedString, TableColumn, StandardListViewItem, VecModel};
use std::path::PathBuf;

fn load_csv(path: &PathBuf) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let file = std::fs::File::open(path).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers: Vec<String> = rdr.headers().ok()?.iter().map(|s| s.to_string()).collect();

    let rows: Vec<Vec<String>> = rdr
        .records()
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|s| s.to_string()).collect())
        .collect();

    Some((headers, rows))
}

fn make_table_columns(headers: &[String]) -> ModelRc<TableColumn> {
    let cols: Vec<TableColumn> = headers
        .iter()
        .map(|h| {
            let mut col = TableColumn::default();
            col.title = SharedString::from(h.as_str());
            col.min_width = 80.0;
            col.horizontal_stretch = 1.0;
            col
        })
        .collect();
    ModelRc::new(VecModel::from(cols))
}

fn make_table_rows(rows: &[Vec<String>]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let result: Vec<ModelRc<StandardListViewItem>> = rows
        .iter()
        .map(|row| {
            let items: Vec<StandardListViewItem> = row
                .iter()
                .map(|cell| {
                    let mut item = StandardListViewItem::default();
                    item.text = SharedString::from(cell.as_str());
                    item
                })
                .collect();
            ModelRc::new(VecModel::from(items))
        })
        .collect();
    ModelRc::new(VecModel::from(result))
}

fn set_table_data(app: &App, headers: &[String], rows: &[Vec<String>]) {
    let global = app.global::<CsvAdapter>();
    global.set_columns(make_table_columns(headers));
    global.set_rows(make_table_rows(rows));
    global.set_row_count(rows.len() as i32);
    global.set_col_count(headers.len() as i32);
}

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;

    let app_weak = app.as_weak();
    app.global::<CsvAdapter>().on_open_file(move || {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv", "tsv"])
            .pick_file()
        {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some((headers, rows)) = load_csv(&path) {
                set_table_data(&app, &headers, &rows);
                app.global::<CsvAdapter>().set_file_name(file_name.into());
            }
        }
    });

    app.run()
}
