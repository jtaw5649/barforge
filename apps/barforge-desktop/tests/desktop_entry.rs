use barforge_desktop::ui::DesktopEntry;
use dioxus::prelude::*;

#[test]
fn desktop_entry_renders_shell() {
    let html = dioxus_ssr::render_element(rsx!(DesktopEntry {}));

    assert!(html.contains("<ul"));
}
