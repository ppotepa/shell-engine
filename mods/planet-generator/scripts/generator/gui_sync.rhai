import "generator/params" as params;

fn step(local, gui) {
    let tab = local.tab ?? 0;
    if local.tab_sync_needed ?? false {
        local = params::sync_widgets_from_state(local, gui, tab);
        local.tab_sync_needed = false;
        return local;
    }

    params::read_widgets_into_state(local, gui, tab)
}
