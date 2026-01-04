use crate::filters::{DEFAULT_SORT, build_search_url};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PaginationControlsProps {
    pub current_page: usize,
    pub total_pages: usize,
    pub current_query: Option<String>,
    pub current_category: Option<String>,
    pub current_sort: Option<String>,
    pub current_view: Option<String>,
    pub base_url: Option<String>,
}

enum PaginationItem {
    Page(usize),
    Ellipsis,
}

#[allow(non_snake_case)]
pub fn PaginationControls(
    PaginationControlsProps {
        current_page,
        total_pages,
        current_query,
        current_category,
        current_sort,
        current_view,
        base_url,
    }: PaginationControlsProps,
) -> Element {
    if total_pages <= 1 {
        return VNode::empty();
    }

    let base_url = base_url.unwrap_or_else(|| "/modules/search".to_string());
    let query = current_query.unwrap_or_default();
    let category = current_category.unwrap_or_default();
    let current_sort = current_sort.unwrap_or_else(|| DEFAULT_SORT.to_string());
    let sort_value = if current_sort == DEFAULT_SORT {
        None
    } else {
        Some(current_sort.as_str())
    };
    let view = current_view.unwrap_or_default();
    let view_value = if view.is_empty() {
        None
    } else {
        Some(view.as_str())
    };

    let prev_page = current_page.saturating_sub(1);
    let next_page = current_page.saturating_add(1);

    let page_items = build_page_items(total_pages, current_page);

    let prev_link = if current_page > 1 {
        let href = build_search_url(
            &base_url,
            Some(query.as_str()),
            if category.is_empty() {
                None
            } else {
                Some(category.as_str())
            },
            sort_value,
            view_value,
            Some(prev_page),
        );
        rsx!(
            a { class: "pagination-btn", href: "{href}",
                svg {
                    class: "pagination-icon",
                    width: "14",
                    height: "14",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "m15 18-6-6 6-6" }
                }
                span { class: "pagination-label", "Previous" }
            }
        )
    } else {
        rsx!(
            span { class: "pagination-btn is-disabled",
                svg {
                    class: "pagination-icon",
                    width: "14",
                    height: "14",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "m15 18-6-6 6-6" }
                }
                span { class: "pagination-label", "Previous" }
            }
        )
    };

    let next_link = if current_page < total_pages {
        let href = build_search_url(
            &base_url,
            Some(query.as_str()),
            if category.is_empty() {
                None
            } else {
                Some(category.as_str())
            },
            sort_value,
            view_value,
            Some(next_page),
        );
        rsx!(
            a { class: "pagination-btn", href: "{href}",
                span { class: "pagination-label", "Next" }
                svg {
                    class: "pagination-icon",
                    width: "14",
                    height: "14",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "m9 18 6-6-6-6" }
                }
            }
        )
    } else {
        rsx!(
            span { class: "pagination-btn is-disabled",
                span { class: "pagination-label", "Next" }
                svg {
                    class: "pagination-icon",
                    width: "14",
                    height: "14",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    path { d: "m9 18 6-6-6-6" }
                }
            }
        )
    };

    rsx! {
        nav { class: "pagination", "aria-label": "Pagination",
            {prev_link}
            div { class: "pagination-pages",
                {page_items.into_iter().enumerate().map(|(index, item)| {
                    match item {
                        PaginationItem::Ellipsis => {
                            let key = format!("ellipsis-{index}");
                            rsx!(
                                span { class: "pagination-ellipsis", key: "{key}", "..." }
                            )
                        }
                        PaginationItem::Page(page) => {
                            let href = build_search_url(
                                &base_url,
                                Some(query.as_str()),
                                if category.is_empty() { None } else { Some(category.as_str()) },
                                sort_value,
                                view_value,
                                Some(page),
                            );
                            let is_active = page == current_page;
                            let key = format!("page-{page}");
                            rsx!(
                                a {
                                    class: if is_active { "pagination-page is-active" } else { "pagination-page" },
                                    href: "{href}",
                                    key: "{key}",
                                    aria_current: if is_active { "page" } else { "false" },
                                    "{page}"
                                }
                            )
                        }
                    }
                })}
            }
            {next_link}
        }
    }
}

fn build_page_items(total_pages: usize, current_page: usize) -> Vec<PaginationItem> {
    if total_pages <= 7 {
        return (1..=total_pages).map(PaginationItem::Page).collect();
    }

    let current_page = current_page.min(total_pages).max(1);
    let mut pages = Vec::new();
    pages.push(PaginationItem::Page(1));

    if current_page > 3 {
        pages.push(PaginationItem::Ellipsis);
    }

    let start = (current_page.saturating_sub(1)).max(2);
    let end = (current_page + 1).min(total_pages - 1);

    for page in start..=end {
        pages.push(PaginationItem::Page(page));
    }

    if current_page < total_pages.saturating_sub(2) {
        pages.push(PaginationItem::Ellipsis);
    }

    pages.push(PaginationItem::Page(total_pages));
    pages
}
