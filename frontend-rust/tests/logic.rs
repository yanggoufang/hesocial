use hesocial_frontend::logic::{next_toggled, toggle_label};

#[test]
fn off_state_uses_off_label() {
    assert_eq!(toggle_label(false), "Off");
}

#[test]
fn on_state_uses_on_label() {
    assert_eq!(toggle_label(true), "On");
}

#[test]
fn toggling_flips_boolean_state() {
    assert!(next_toggled(false));
    assert!(!next_toggled(true));
}
