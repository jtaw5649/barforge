#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortOption {
    pub name: &'static str,
    pub value: &'static str,
}

pub const DEFAULT_SORT: &str = "popular";

pub fn default_sort_options() -> Vec<SortOption> {
    vec![
        SortOption {
            name: "Most Popular",
            value: "popular",
        },
        SortOption {
            name: "Trending",
            value: "trending",
        },
        SortOption {
            name: "Recently Added",
            value: "recent",
        },
        SortOption {
            name: "Most Downloads",
            value: "downloads",
        },
        SortOption {
            name: "Alphabetical",
            value: "alpha",
        },
    ]
}

pub fn build_search_url(
    base_url: &str,
    query: Option<&str>,
    category: Option<&str>,
    sort: Option<&str>,
    view: Option<&str>,
    page: Option<usize>,
) -> String {
    let mut params: Vec<(&str, String)> = Vec::new();

    if let Some(query) = query.filter(|value| !value.is_empty()) {
        params.push(("q", query.to_string()));
    }
    if let Some(category) = category.filter(|value| !value.is_empty()) {
        params.push(("category", category.to_string()));
    }
    if let Some(sort) = sort.filter(|value| !value.is_empty()) {
        params.push(("sort", sort.to_string()));
    }
    if let Some(view) = view.filter(|value| !value.is_empty()) {
        params.push(("view", view.to_string()));
    }
    if let Some(page) = page.filter(|value| *value > 1) {
        params.push(("page", page.to_string()));
    }

    if params.is_empty() {
        base_url.to_string()
    } else {
        let mut query_string = String::new();
        for (index, (key, value)) in params.iter().enumerate() {
            if index > 0 {
                query_string.push('&');
            }
            query_string.push_str(key);
            query_string.push('=');
            query_string.push_str(value);
        }
        format!("{base_url}?{query_string}")
    }
}
