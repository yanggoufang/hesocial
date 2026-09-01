pub fn toggle_label(toggled: bool) -> &'static str {
    if toggled { "On" } else { "Off" }
}

pub fn next_toggled(current: bool) -> bool {
    !current
}
