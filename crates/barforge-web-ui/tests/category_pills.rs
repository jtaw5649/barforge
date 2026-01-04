use barforge_web_ui::categories::{Category, browse_categories, homepage_categories};
use barforge_web_ui::components::{CategoryPills, CategoryPillsProps};
use dioxus::prelude::*;
#[path = "support/fragment_keys.rs"]
mod fragment_keys;

#[test]
fn category_pills_render_links_with_colors() {
    let categories = homepage_categories();

    let html = render_pills(categories, Some("/modules/search".to_string()));

    assert!(html.contains("category-pills"));
    assert!(html.contains("System"));
    assert!(html.contains("Media"));
    assert!(html.contains("href=\"/modules/search?category=system\""));
    assert!(html.contains("href=\"/modules/search?category=media\""));
    assert!(html.contains("category-system.svg"));
    assert!(html.contains("category-media.svg"));
    assert!(html.contains("#617dfa"));
    assert!(html.contains("#ec4899"));
}

#[test]
fn category_pills_default_base_url() {
    let categories = homepage_categories();

    let html = render_pills(categories, None);

    assert!(html.contains("href=\"/modules/search?category=system\""));
}

#[test]
fn category_pills_assigns_keys_to_categories() {
    let categories = browse_categories();
    let expected_len = categories.len();

    let vnode = CategoryPills(CategoryPillsProps {
        categories,
        base_url: None,
    })
    .expect("expected CategoryPills vnode");
    let keys = fragment_keys::find_fragment_keys(&vnode, expected_len)
        .expect("expected category fragment for keys");

    assert!(
        keys.iter()
            .all(|key| key.as_ref().map(|value| !value.is_empty()).unwrap_or(false))
    );
}

fn render_pills(categories: Vec<Category>, base_url: Option<String>) -> String {
    dioxus_ssr::render_element(rsx!(CategoryPills {
        categories,
        base_url
    }))
}
