use barforge_types::{ModuleCategory, RegistryModule};
use barforge_web_ui::components::{ModuleList, ModuleSort, ModuleViewMode};
use dioxus::prelude::*;
#[path = "support/sample_module.rs"]
mod sample_module;
use sample_module::sample_module;
use std::collections::HashSet;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[test]
fn module_list_renders_names_in_order() {
    let modules = vec![
        sample_module("weather-wttr@barforge", "Weather"),
        sample_module("clock-time@barforge", "Clock"),
    ];

    let html = render_list(modules.clone(), None, None, None);

    let first = html
        .find("Weather")
        .expect("expected Weather module in output");
    let second = html.find("Clock").expect("expected Clock module in output");

    assert!(first < second, "expected modules to render in input order");
    assert_eq!(
        html.matches("class=\"module-card\"").count(),
        modules.len(),
        "expected one list item per module"
    );
}

#[test]
fn module_list_renders_empty_state() {
    let html = render_list(Vec::<RegistryModule>::new(), None, None, None);

    assert!(html.contains("No modules found"));
    assert_eq!(
        html.matches("class=\"module-card\"").count(),
        0,
        "expected no list items"
    );
}

#[test]
fn module_list_renders_author_and_category() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("by barforge"));
    assert!(html.contains("Weather"));
    assert!(html.contains("module-tag"));
}

#[test]
fn module_list_renders_author_hover_card() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("class=\"profile-hover\""));
    assert!(html.contains("class=\"profile-hover-card\""));
    assert!(html.contains("href=\"/users/barforge\""));
}

#[test]
fn module_list_cards_do_not_render_nested_links() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("class=\"module-card\""));
    assert!(!html.contains("<a class=\"module-card\""));
    assert!(html.contains("class=\"module-card-link\""));
    assert!(html.contains("href=\"/modules/weather-wttr@barforge\""));
    assert!(html.contains("href=\"/users/barforge\""));
}

#[test]
fn module_list_links_to_detail() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("href=\"/modules/weather-wttr@barforge\""));
}

#[test]
fn module_list_renders_downloads_and_version() {
    let mut module = sample_module("weather-wttr@barforge", "Weather");
    module.downloads = 1200;
    module.version = Some("1.2.3".to_string());

    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("1.2k"));
    assert!(html.contains("v1.2.3"));
}

#[test]
fn module_list_renders_download_icon_in_cards() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("class=\"module-downloads\""));
    assert!(html.contains("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"));
}

#[test]
fn module_list_renders_verified_tag() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_list(vec![module], None, None, None);

    assert!(html.contains("Verified"));
}

#[test]
fn module_list_renders_starred_tag() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let mut starred = HashSet::new();
    starred.insert(module.uuid.clone());

    let html = render_list_with_starred(vec![module], starred);

    assert!(html.contains("Starred"));
}

#[test]
fn module_list_renders_new_tag() {
    let mut module = sample_module("weather-wttr@barforge", "Weather");
    module.last_updated = Some("2024-01-08T00:00:00Z".to_string());

    let html = render_list_with_now(vec![module], None, None, None, reference_time(), None, None);

    assert!(html.contains("New"));
}

#[test]
fn module_list_renders_list_view_rows() {
    let modules = vec![
        sample_module("weather-wttr@barforge", "Weather"),
        sample_module("clock-time@barforge", "Clock"),
    ];

    let html = render_list_with_view_mode(modules.clone(), ModuleViewMode::List);

    assert_eq!(
        html.matches("class=\"row\"").count(),
        modules.len(),
        "expected one row per module"
    );
    assert!(html.contains("class=\"row-wrapper\""));
}

#[test]
fn module_list_rows_do_not_render_nested_links() {
    let modules = vec![sample_module("weather-wttr@barforge", "Weather")];
    let html = render_list_with_view_mode(modules, ModuleViewMode::List);

    assert!(html.contains("class=\"row\""));
    assert!(!html.contains("<a class=\"row\""));
    assert!(html.contains("class=\"row-link\""));
    assert!(html.contains("href=\"/modules/weather-wttr@barforge\""));
    assert!(html.contains("href=\"/users/barforge\""));
}

#[test]
fn module_list_filters_by_query() {
    let modules = vec![
        sample_module("weather-wttr@barforge", "Weather"),
        sample_module("clock-time@barforge", "Clock"),
    ];

    let html = render_list(modules, Some("weath"), None, None);

    assert!(html.contains("Weather"));
    assert!(!html.contains("Clock"));
    assert_eq!(html.matches("<li ").count(), 1);
}

#[test]
fn module_list_sorts_alphabetically() {
    let modules = vec![
        sample_module("weather-wttr@barforge", "Weather"),
        sample_module("clock-time@barforge", "Clock"),
    ];

    let html = render_list(modules, None, Some(ModuleSort::Alphabetical), None);

    let first = html.find("Clock").expect("expected Clock module in output");
    let second = html
        .find("Weather")
        .expect("expected Weather module in output");

    assert!(first < second, "expected alphabetical ordering");
}

#[test]
fn module_list_sorts_by_downloads() {
    let mut weather = sample_module("weather-wttr@barforge", "Weather");
    weather.downloads = 10;
    let mut clock = sample_module("clock-time@barforge", "Clock");
    clock.downloads = 500;

    let html = render_list(
        vec![weather, clock],
        None,
        Some(ModuleSort::Downloads),
        None,
    );

    let first = html.find("Clock").expect("expected Clock module in output");
    let second = html
        .find("Weather")
        .expect("expected Weather module in output");

    assert!(first < second, "expected downloads ordering");
}

#[test]
fn module_list_sorts_by_popularity() {
    let mut heavy = sample_module("heavy-downloads@barforge", "Heavy");
    heavy.downloads = 2000;
    heavy.rating = Some(1.0);
    heavy.last_updated = Some("2000-01-01T00:00:00Z".to_string());

    let mut rising = sample_module("rising-module@barforge", "Rising");
    rising.downloads = 100;
    rising.rating = Some(5.0);
    rising.last_updated = Some("2050-01-01T00:00:00Z".to_string());

    let html = render_list(vec![heavy, rising], None, Some(ModuleSort::Popular), None);

    let first = html
        .find("Rising")
        .expect("expected Rising module in output");
    let second = html.find("Heavy").expect("expected Heavy module in output");

    assert!(first < second, "expected popularity ordering");
}

#[test]
fn module_list_sorts_by_trending() {
    let mut stale = sample_module("stale-module@barforge", "Stale");
    stale.downloads = 200;
    stale.rating = Some(3.0);
    stale.last_updated = Some("2000-01-01T00:00:00Z".to_string());

    let mut recent = sample_module("recent-module@barforge", "Recent");
    recent.downloads = 200;
    recent.rating = Some(3.0);
    recent.last_updated = Some("2050-01-01T00:00:00Z".to_string());

    let html = render_list(vec![stale, recent], None, Some(ModuleSort::Trending), None);

    let first = html
        .find("Recent")
        .expect("expected Recent module in output");
    let second = html.find("Stale").expect("expected Stale module in output");

    assert!(first < second, "expected trending ordering");
}

#[test]
fn module_list_sorts_by_recent() {
    let mut old = sample_module("old-module@barforge", "Old");
    old.last_updated = Some("2000-01-01T00:00:00Z".to_string());

    let mut fresh = sample_module("fresh-module@barforge", "Fresh");
    fresh.last_updated = Some("2050-01-01T00:00:00Z".to_string());

    let html = render_list(vec![old, fresh], None, Some(ModuleSort::Recent), None);

    let first = html.find("Fresh").expect("expected Fresh module in output");
    let second = html.find("Old").expect("expected Old module in output");

    assert!(first < second, "expected recent ordering");
}

#[test]
fn module_list_uses_reference_time_for_scoring() {
    let mut older = sample_module("older-module@barforge", "Older");
    older.downloads = 100;
    older.rating = Some(3.0);
    older.last_updated = Some("2024-01-01T00:00:00Z".to_string());

    let mut newer = sample_module("newer-module@barforge", "Newer");
    newer.downloads = 100;
    newer.rating = Some(3.0);
    newer.last_updated = Some("2024-01-09T00:00:00Z".to_string());

    let now =
        OffsetDateTime::parse("2024-01-10T00:00:00Z", &Rfc3339).expect("valid reference time");
    let html = render_list_with_now(
        vec![older, newer],
        None,
        Some(ModuleSort::Popular),
        None,
        now,
        None,
        None,
    );

    let first = html.find("Newer").expect("expected Newer module in output");
    let second = html.find("Older").expect("expected Older module in output");

    assert!(first < second, "expected scoring to use reference time");
}

fn render_list(
    modules: Vec<RegistryModule>,
    query: Option<&str>,
    sort: Option<ModuleSort>,
    category: Option<ModuleCategory>,
) -> String {
    render_list_with_page(modules, query, sort, category, None, None)
}

fn render_list_with_page(
    modules: Vec<RegistryModule>,
    query: Option<&str>,
    sort: Option<ModuleSort>,
    category: Option<ModuleCategory>,
    page: Option<usize>,
    per_page: Option<usize>,
) -> String {
    render_list_with_now(
        modules,
        query,
        sort,
        category,
        reference_time(),
        page,
        per_page,
    )
}

fn render_list_with_starred(modules: Vec<RegistryModule>, starred: HashSet<String>) -> String {
    dioxus_ssr::render_element(rsx!(ModuleList {
        modules,
        query: None,
        sort: None,
        category: None,
        page: None,
        per_page: None,
        starred: Some(starred),
        on_toggle_star: None,
        view_mode: None,
        now: reference_time(),
    }))
}

fn render_list_with_now(
    modules: Vec<RegistryModule>,
    query: Option<&str>,
    sort: Option<ModuleSort>,
    category: Option<ModuleCategory>,
    now: OffsetDateTime,
    page: Option<usize>,
    per_page: Option<usize>,
) -> String {
    dioxus_ssr::render_element(rsx!(ModuleList {
        modules,
        query: query.map(|value| value.to_string()),
        sort,
        category,
        page,
        per_page,
        view_mode: None,
        now
    }))
}

fn render_list_with_view_mode(modules: Vec<RegistryModule>, view_mode: ModuleViewMode) -> String {
    dioxus_ssr::render_element(rsx!(ModuleList {
        modules,
        query: None,
        sort: None,
        category: None,
        page: None,
        per_page: None,
        view_mode: Some(view_mode),
        now: reference_time(),
    }))
}

fn reference_time() -> OffsetDateTime {
    OffsetDateTime::parse("2024-01-10T00:00:00Z", &Rfc3339).expect("valid reference time")
}

#[test]
fn module_list_filters_by_category() {
    let mut weather = sample_module("weather-wttr@barforge", "Weather");
    weather.category = ModuleCategory::Weather;
    let mut audio = sample_module("audio-control@barforge", "Audio");
    audio.category = ModuleCategory::Audio;

    let html = render_list(
        vec![weather, audio],
        None,
        None,
        Some(ModuleCategory::Weather),
    );

    assert!(html.contains("Weather"));
    assert!(!html.contains("Audio"));
    assert_eq!(html.matches("<li ").count(), 1);
}

#[test]
fn module_list_paginates_results() {
    let modules = vec![
        sample_module("first@barforge", "First"),
        sample_module("second@barforge", "Second"),
        sample_module("third@barforge", "Third"),
    ];

    let html = render_list_with_page(modules, None, None, None, Some(2), Some(2));

    assert!(!html.contains("First"));
    assert!(!html.contains("Second"));
    assert!(html.contains("Third"));
    assert_eq!(html.matches("<li ").count(), 1);
}
