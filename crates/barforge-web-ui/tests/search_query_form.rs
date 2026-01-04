use barforge_web_ui::components::SearchQueryForm;
use dioxus::prelude::*;

#[test]
fn search_query_form_renders_inputs() {
    let html = render_form(
        Some("clock".to_string()),
        Some("media".to_string()),
        Some("downloads".to_string()),
    );

    assert!(html.contains("action=\"/modules/search\""));
    assert!(html.contains("name=\"q\""));
    assert!(html.contains("value=\"clock\""));
    assert!(html.contains("name=\"category\""));
    assert!(html.contains("value=\"media\""));
    assert!(html.contains("name=\"sort\""));
    assert!(html.contains("value=\"downloads\""));
    assert!(html.contains("class=\"search-icon\""));
    assert!(html.contains("stroke=\"currentColor\""));
}

#[test]
fn search_query_form_omits_default_sort() {
    let html = render_form(Some("clock".to_string()), None, Some("popular".to_string()));

    assert!(html.contains("name=\"q\""));
    assert!(!html.contains("name=\"sort\""));
}

fn render_form(query: Option<String>, category: Option<String>, sort: Option<String>) -> String {
    dioxus_ssr::render_element(rsx!(SearchQueryForm {
        current_query: query,
        current_category: category,
        current_sort: sort,
        base_url: None
    }))
}
