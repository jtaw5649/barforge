use crate::filters::{DEFAULT_SORT, build_search_url};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchQueryFormProps {
    pub current_query: Option<String>,
    pub current_category: Option<String>,
    pub current_sort: Option<String>,
    pub base_url: Option<String>,
}

#[allow(non_snake_case)]
pub fn SearchQueryForm(
    SearchQueryFormProps {
        current_query,
        current_category,
        current_sort,
        base_url,
    }: SearchQueryFormProps,
) -> Element {
    let base_url = base_url.unwrap_or_else(|| "/modules/search".to_string());
    let query_value = current_query.unwrap_or_default();
    let current_sort = current_sort.unwrap_or_else(|| DEFAULT_SORT.to_string());
    let sort_value = if current_sort == DEFAULT_SORT {
        None
    } else {
        Some(current_sort.as_str())
    };

    let category_value = current_category.unwrap_or_default();
    let category_value = category_value.as_str();

    let action = build_search_url(&base_url, None, None, None, None, None);

    rsx! {
        form { class: "search-form", method: "get", action: "{action}",
            div { class: "search-input-wrapper",
                svg {
                    class: "search-icon",
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "m21 21-4.34-4.34" }
                    circle { cx: "11", cy: "11", r: "8" }
                }
                input {
                    class: "search-input",
                    r#type: "search",
                    name: "q",
                    value: "{query_value}",
                    placeholder: "Filter modules..."
                }
            }
            if !category_value.is_empty() {
                input { r#type: "hidden", name: "category", value: "{category_value}" }
            }
            if let Some(sort_value) = sort_value {
                input { r#type: "hidden", name: "sort", value: "{sort_value}" }
            }
            button { class: "search-submit", r#type: "submit", "Search" }
        }
    }
}
