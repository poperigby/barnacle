use barnacle::profiles::ResolvedModEntry;
use slint::{StandardListViewItem, VecModel};

slint::include_modules!();

type TableRow = VecModel<StandardListViewItem>;
type TableModel = VecModel<TableRow>;

pub fn start_gui() {
    let main_window = MainWindow::new().unwrap();
    main_window.run().unwrap();
}

fn build_table_model(resolved_entries: &[ResolvedModEntry]) -> TableModel {
    let mut rows = Vec::new();

    for entry in resolved_entries {
        let row = vec![
            StandardListViewItem::from(entry.mod_ref().name()),
            StandardListViewItem::from(if *entry.entry().enabled() { "✔" } else { "" }),
        ];

        rows.push(TableRow::from(row));
    }

    TableModel::from(rows)
}
