use crate::categories::Category;
use crate::filters::{DEFAULT_SORT, SortOption};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchToolbarProps {
    pub categories: Vec<Category>,
    pub sort_options: Vec<SortOption>,
    pub current_category: Option<String>,
    pub current_sort: Option<String>,
    pub on_category: Option<EventHandler<String>>,
    pub on_sort: Option<EventHandler<String>>,
}

#[allow(non_snake_case)]
pub fn SearchToolbar(
    SearchToolbarProps {
        categories,
        sort_options,
        current_category,
        current_sort,
        on_category,
        on_sort,
    }: SearchToolbarProps,
) -> Element {
    let current_category = current_category.unwrap_or_default();
    let current_sort = current_sort.unwrap_or_else(|| DEFAULT_SORT.to_string());
    rsx! {
        div {
            div { class: "filter-section",
                h3 { "Category" }
                div { class: "filter-options",
                    {categories.into_iter().map(|category| {
                        let is_active = current_category == category.slug;
                        let key = if category.slug.is_empty() {
                            category.name
                        } else {
                            category.slug
                        };
                        let slug = category.slug.to_string();
                        rsx! {
                            button {
                                class: if is_active { "filter-option active" } else { "filter-option" },
                                r#type: "button",
                                key: "{key}",
                                onclick: move |_| {
                                    if let Some(handler) = on_category {
                                        handler.call(slug.clone());
                                    }
                                },
                                "{category.name}"
                            }
                        }
                    })}
                }
            }
            div { class: "filter-section",
                h3 { "Sort By" }
                select {
                    class: "sort-select",
                    value: "{current_sort}",
                    onchange: move |evt| {
                        if let Some(handler) = on_sort {
                            handler.call(evt.value());
                        }
                    },
                    {sort_options.into_iter().map(|option| {
                        rsx! {
                            option {
                                value: "{option.value}",
                                key: "{option.value}",
                                "{option.name}"
                            }
                        }
                    })}
                }
            }
        }
    }
}
