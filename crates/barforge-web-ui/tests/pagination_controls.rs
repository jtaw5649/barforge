use barforge_web_ui::components::{PaginationControls, PaginationControlsProps};
use dioxus::prelude::*;
#[path = "support/fragment_keys.rs"]
mod fragment_keys;

#[test]
fn pagination_controls_render_page_links() {
    let html = render_pagination(
        2,
        3,
        Some("clock".to_string()),
        Some("media".to_string()),
        Some("downloads".to_string()),
    );

    assert!(html.contains("aria-label=\"Pagination\""));
    assert!(html.contains("pagination-btn"));
    assert!(html.contains("pagination-page"));
    assert!(
        html.contains("href=\"/modules/search?q=clock&#38;category=media&#38;sort=downloads\"")
    );
    assert!(html.contains(
        "href=\"/modules/search?q=clock&#38;category=media&#38;sort=downloads&#38;page=2\""
    ));
    assert!(html.contains(
        "href=\"/modules/search?q=clock&#38;category=media&#38;sort=downloads&#38;page=3\""
    ));
    assert!(html.contains("class=\"pagination-icon\""));
    assert!(html.contains("stroke=\"currentColor\""));
}

#[test]
fn pagination_controls_render_ellipsis() {
    let html = render_pagination(5, 10, None, None, None);

    assert!(html.contains("pagination-ellipsis"));
}

#[test]
fn pagination_controls_assigns_keys_to_page_items() {
    let current_page = 5;
    let total_pages = 10;

    let vnode = PaginationControls(PaginationControlsProps {
        current_page,
        total_pages,
        current_query: None,
        current_category: None,
        current_sort: None,
        current_view: None,
        base_url: None,
    })
    .expect("expected PaginationControls vnode");

    let keys = fragment_keys::find_fragment_keys(&vnode, 7).expect("expected page items fragment");
    assert!(
        keys.iter()
            .all(|key| key.as_ref().map(|value| !value.is_empty()).unwrap_or(false))
    );
}

fn render_pagination(
    current_page: usize,
    total_pages: usize,
    query: Option<String>,
    category: Option<String>,
    sort: Option<String>,
) -> String {
    dioxus_ssr::render_element(rsx!(PaginationControls {
        current_page,
        total_pages,
        current_query: query,
        current_category: category,
        current_sort: sort,
        current_view: None,
        base_url: None
    }))
}
