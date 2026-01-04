use barforge_desktop::state::{BrowseState, SortField, SortOrder, ViewMode};
use barforge_types::{ModuleCategory, RegistryModule};
use std::time::{Duration, Instant};

fn make_module(
    name: &str,
    category: ModuleCategory,
    downloads: i64,
    rating: Option<f32>,
    last_updated: Option<&str>,
    verified: bool,
    tags: &[&str],
) -> RegistryModule {
    RegistryModule {
        uuid: format!("{name}-uuid"),
        name: name.to_string(),
        description: format!("Module {name} description"),
        author: "test-author".to_string(),
        category,
        icon: None,
        screenshot: None,
        repo_url: "https://example.test".to_string(),
        downloads,
        version: None,
        last_updated: last_updated.map(str::to_string),
        rating,
        verified_author: verified,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
        checksum: None,
        license: None,
    }
}

#[test]
fn filters_by_search_query_case_insensitive() {
    let modules = vec![
        make_module(
            "Weather",
            ModuleCategory::Weather,
            1200,
            Some(4.7),
            Some("2024-02-01T00:00:00Z"),
            true,
            &["forecast"],
        ),
        make_module(
            "Clock",
            ModuleCategory::Time,
            800,
            Some(4.2),
            Some("2024-01-01T00:00:00Z"),
            false,
            &["time"],
        ),
    ];

    let state = BrowseState {
        search_query: "weAtHer".to_string(),
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["Weather"]);
}

#[test]
fn filters_by_category() {
    let modules = vec![
        make_module(
            "Weather",
            ModuleCategory::Weather,
            1200,
            Some(4.7),
            Some("2024-02-01T00:00:00Z"),
            true,
            &["forecast"],
        ),
        make_module(
            "Clock",
            ModuleCategory::Time,
            800,
            Some(4.2),
            Some("2024-01-01T00:00:00Z"),
            false,
            &["time"],
        ),
    ];

    let state = BrowseState {
        selected_category: Some(ModuleCategory::Time),
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["Clock"]);
}

#[test]
fn filters_verified_only() {
    let modules = vec![
        make_module(
            "Weather",
            ModuleCategory::Weather,
            1200,
            Some(4.7),
            Some("2024-02-01T00:00:00Z"),
            true,
            &["forecast"],
        ),
        make_module(
            "Clock",
            ModuleCategory::Time,
            800,
            Some(4.2),
            Some("2024-01-01T00:00:00Z"),
            false,
            &["time"],
        ),
    ];

    let state = BrowseState {
        verified_only: true,
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["Weather"]);
}

#[test]
fn sorts_by_downloads_descending() {
    let modules = vec![
        make_module(
            "Low",
            ModuleCategory::System,
            10,
            Some(3.9),
            Some("2024-01-01T00:00:00Z"),
            false,
            &[],
        ),
        make_module(
            "High",
            ModuleCategory::System,
            500,
            Some(4.9),
            Some("2024-02-01T00:00:00Z"),
            true,
            &[],
        ),
    ];

    let state = BrowseState {
        sort_field: SortField::Downloads,
        sort_order: SortOrder::Descending,
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["High", "Low"]);
}

#[test]
fn sorts_by_name_ascending_case_insensitive() {
    let modules = vec![
        make_module(
            "clock",
            ModuleCategory::Time,
            10,
            Some(4.1),
            Some("2024-01-01T00:00:00Z"),
            false,
            &[],
        ),
        make_module(
            "Weather",
            ModuleCategory::Weather,
            20,
            Some(4.5),
            Some("2024-02-01T00:00:00Z"),
            true,
            &[],
        ),
    ];

    let state = BrowseState {
        sort_field: SortField::Name,
        sort_order: SortOrder::Ascending,
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["clock", "Weather"]);
}

#[test]
fn sorts_by_rating_descending() {
    let modules = vec![
        make_module(
            "Low",
            ModuleCategory::System,
            10,
            None,
            Some("2024-01-01T00:00:00Z"),
            false,
            &[],
        ),
        make_module(
            "High",
            ModuleCategory::System,
            10,
            Some(4.8),
            Some("2024-01-02T00:00:00Z"),
            true,
            &[],
        ),
    ];

    let state = BrowseState {
        sort_field: SortField::Rating,
        sort_order: SortOrder::Descending,
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["High", "Low"]);
}

#[test]
fn sorts_by_recently_updated_descending() {
    let modules = vec![
        make_module(
            "Older",
            ModuleCategory::System,
            10,
            Some(4.2),
            Some("2024-01-01T00:00:00Z"),
            false,
            &[],
        ),
        make_module(
            "Newer",
            ModuleCategory::System,
            10,
            Some(4.2),
            Some("2024-02-01T00:00:00Z"),
            false,
            &[],
        ),
    ];

    let state = BrowseState {
        sort_field: SortField::RecentlyUpdated,
        sort_order: SortOrder::Descending,
        ..BrowseState::default()
    };

    let names: Vec<_> = state
        .filtered_modules(&modules)
        .iter()
        .map(|module| module.name.as_str())
        .collect();

    assert_eq!(names, vec!["Newer", "Older"]);
}

#[test]
fn toggles_sort_order() {
    assert_eq!(SortOrder::Ascending.toggle(), SortOrder::Descending);
    assert_eq!(SortOrder::Descending.toggle(), SortOrder::Ascending);
}

#[test]
fn debounced_search_applies_after_delay() {
    let start = Instant::now();
    let mut state = BrowseState::default();

    state.queue_search("clock".to_string(), start);
    state.apply_debounced_searches_at(start + Duration::from_millis(149));

    assert_eq!(state.search_query, "");
    assert_eq!(state.pending_search.as_deref(), Some("clock"));

    state.apply_debounced_searches_at(start + Duration::from_millis(150));

    assert_eq!(state.search_query, "clock");
    assert!(state.pending_search.is_none());
    assert!(state.search_debounce_start.is_none());
}

#[test]
fn search_display_prefers_pending_query() {
    let start = Instant::now();
    let mut state = BrowseState::default();

    state.search_query = "old".to_string();
    state.queue_search("new".to_string(), start);

    assert_eq!(state.search_display(), "new");
}

#[test]
fn refresh_updates_timestamp_on_success() {
    let now = Instant::now();
    let mut state = BrowseState::default();

    state.start_refresh();
    assert!(state.refreshing);

    state.finish_refresh(true, now);

    assert!(!state.refreshing);
    assert_eq!(state.last_refreshed, Some(now));
}

#[test]
fn refresh_failure_clears_refreshing_without_changing_timestamp() {
    let now = Instant::now();
    let mut state = BrowseState::default();

    state.last_refreshed = Some(now);
    state.start_refresh();
    state.finish_refresh(false, now + Duration::from_secs(5));

    assert!(!state.refreshing);
    assert_eq!(state.last_refreshed, Some(now));
}

#[test]
fn view_mode_changes_track_persistence() {
    let mut state = BrowseState::default();

    assert_eq!(state.view_mode, ViewMode::Cards);
    assert!(!state.view_mode_dirty());

    state.set_view_mode(ViewMode::Table);

    assert_eq!(state.view_mode, ViewMode::Table);
    assert!(state.view_mode_dirty());

    state.mark_view_mode_persisted();

    assert!(!state.view_mode_dirty());
    assert_eq!(state.persisted_view_mode, ViewMode::Table);
}
