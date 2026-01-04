use barforge_web::AppEntry;
use dioxus::prelude::*;
use dioxus_history::{History, MemoryHistory, provide_history_context};
use std::rc::Rc;

#[test]
fn app_entry_renders_root() {
    let html = dioxus_ssr::render_element(rsx!(TestRoot {}));

    assert!(html.contains("Barforge"));
}

#[component]
fn TestRoot() -> Element {
    let history = Rc::new(MemoryHistory::default());
    history.replace("/".to_string());
    provide_history_context(history);

    rsx!(AppEntry {})
}
