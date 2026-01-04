use barforge_web_ui::categories::browse_categories;
use barforge_web_ui::components::SearchToolbar;
use barforge_web_ui::filters::default_sort_options;
use dioxus::prelude::*;

#[test]
fn search_toolbar_renders_category_buttons_and_sort_select() {
    let categories = browse_categories();
    let sort_options = default_sort_options();

    let html = render_toolbar(
        categories,
        sort_options,
        Some("media".to_string()),
        Some("downloads".to_string()),
    );

    assert!(html.contains("Category"));
    assert!(html.contains("Sort By"));
    assert!(html.contains("class=\"filter-section\""));
    assert!(html.contains("class=\"filter-option active\""));
    assert!(html.contains("class=\"sort-select\""));
    assert!(html.contains("value=\"downloads\""));
    assert!(html.contains("value=\"popular\""));
    assert!(!html.contains("category-all.svg"));
    assert!(!html.contains("href=\"/modules/search"));
}

#[test]
fn search_toolbar_renders_all_categories_and_sort_options() {
    let categories = browse_categories();
    let sort_options = default_sort_options();

    let html = render_toolbar(
        categories.clone(),
        sort_options.clone(),
        None,
        Some("popular".to_string()),
    );

    for category in categories {
        assert!(html.contains(category.name));
    }
    for option in sort_options {
        assert!(html.contains(option.name));
    }
}

fn render_toolbar(
    categories: Vec<barforge_web_ui::categories::Category>,
    sort_options: Vec<barforge_web_ui::filters::SortOption>,
    category: Option<String>,
    sort: Option<String>,
) -> String {
    dioxus_ssr::render_element(rsx!(SearchToolbar {
        categories,
        sort_options,
        current_category: category,
        current_sort: sort,
        on_category: None,
        on_sort: None
    }))
}
