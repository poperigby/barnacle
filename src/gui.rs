use barnacle::{data::profiles::ResolvedModEntry, state_file::State};
use slint::{ModelRc, StandardListViewItem};

slint::include_modules!();

type TableRow = ModelRc<StandardListViewItem>;
type TableModel = ModelRc<TableRow>;

pub fn start_gui(state: &State) {
    let app = App::new().unwrap();

    // let game = &state.games[0];
    // let profile = game.profiles()[0].clone();
    // let resolved_mods = profile.resolve_mod_entries(game);
    // dbg!(&resolved_mods);
    // let mod_table_model = build_table_model(&resolved_mods);
    //
    // app.global::<ModTableData>().set_model(mod_table_model);
    // app.run().unwrap();
}

fn build_table_model(resolved_entries: &[ResolvedModEntry]) -> TableModel {
    let mut rows = Vec::new();

    for entry in resolved_entries {
        let row = [
            StandardListViewItem::from(entry.mod_ref().name()),
            StandardListViewItem::from(if *entry.entry().enabled() {
                "✅"
            } else {
                "❌"
            }),
        ];

        rows.push(TableRow::from(row));
    }

    TableModel::from(rows.as_slice())
}
