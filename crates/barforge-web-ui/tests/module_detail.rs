use barforge_types::{RegistryModule, Review, ReviewUser, VersionHistoryEntry};
use barforge_web_ui::components::ModuleDetail;
use dioxus::prelude::*;
#[path = "support/sample_module.rs"]
mod sample_module;
use sample_module::sample_module;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[test]
fn module_detail_renders_metadata() {
    let mut module = sample_module("weather-wttr@barforge", "Weather");
    module.description = "Forecasts and alerts.".to_string();
    module.version = Some("1.2.3".to_string());
    module.downloads = 42;

    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("[ MODULE ]"));
    assert!(html.contains("Weather"));
    assert!(html.contains("by barforge"));
    assert!(html.contains("Forecasts and alerts."));
    assert!(html.contains("v1.2.3"));
    assert!(html.contains("42 downloads"));
}

#[test]
fn module_detail_renders_icon_initial_and_verified_badge() {
    let module = sample_module("weather-wttr@barforge", "Weather");

    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("module-detail-icon"));
    assert!(html.contains("module-detail-initial"));
    assert!(html.contains("aria-label=\"Verified author\""));
}

#[test]
fn module_detail_renders_category_tag_and_downloads_icon() {
    let mut module = sample_module("weather-wttr@barforge", "Weather");
    module.downloads = 1200;

    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("module-tag tag-category"));
    assert!(html.contains("module-stats"));
    assert!(html.contains("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"));
}

#[test]
fn module_detail_renders_install_command() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("barforge install weather-wttr@barforge"));
    assert!(html.contains("copy-btn"));
    assert!(html.contains("copy-icon"));
    assert!(html.contains("Copy"));
}

#[test]
fn module_detail_links_to_repo() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("href=\"https://github.com/barforge/example\""));
    assert!(html.contains("module-repo-icon"));
}

#[test]
fn module_detail_renders_back_link() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("Back to modules"));
    assert!(html.contains("href=\"/modules\""));
    assert!(html.contains("module-back-icon"));
}

#[test]
fn module_detail_renders_inline_action_icons() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("module-repo-icon"));
    assert!(html.contains("copy-icon"));
    assert!(html.contains("carousel-close-icon"));
    assert!(html.contains("carousel-nav-icon"));
}

#[test]
fn module_detail_renders_screenshots() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let screenshots = vec![
        "https://example.com/shot-1.png".to_string(),
        "https://example.com/shot-2.png".to_string(),
    ];

    let html = render_detail(
        module,
        screenshots,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("[ SCREENSHOTS ]"));
    assert_eq!(html.matches("data-screenshot-index").count(), 2);
    assert!(html.contains("https://example.com/shot-1.png"));
    assert!(html.contains("https://example.com/shot-2.png"));
    assert!(html.contains("screenshot-carousel"));
}

#[test]
fn module_detail_assigns_keys_to_screenshots() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let screenshots = vec![
        "https://example.com/shot-1.png".to_string(),
        "https://example.com/shot-2.png".to_string(),
        "https://example.com/shot-3.png".to_string(),
    ];
    let expected_len = screenshots.len();

    let html = render_detail(
        module,
        screenshots,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert_eq!(html.matches("data-screenshot-index").count(), expected_len);
}

#[test]
fn module_detail_renders_empty_screenshots_state() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("Placeholder"));
    assert!(html.contains("data-placeholder=\"true\""));
}

#[test]
fn module_detail_renders_related_modules() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let related = vec![
        sample_module("clock-time@barforge", "Clock"),
        sample_module("power-monitor@barforge", "Power"),
    ];

    let html = render_detail(module, Vec::new(), related, Vec::new(), Vec::new(), false);

    assert!(html.contains("[ RELATED MODULES ]"));
    assert!(html.contains("Related modules"));
    assert!(html.contains("Clock"));
    assert!(html.contains("Power"));
    assert_eq!(html.matches("class=\"module-card\"").count(), 2);
}

#[test]
fn module_detail_renders_empty_related_state() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("No related modules"));
}

#[test]
fn module_detail_renders_related_skeletons_when_loading() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(module, Vec::new(), Vec::new(), Vec::new(), Vec::new(), true);

    assert!(html.contains("module-card-skeleton"));
}

#[test]
fn module_detail_renders_versions() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let versions = vec![
        VersionHistoryEntry {
            version: "1.2.0".to_string(),
            changelog: Some("Added alerts".to_string()),
            downloads: 12,
            published_at: "2024-01-01".to_string(),
        },
        VersionHistoryEntry {
            version: "1.1.0".to_string(),
            changelog: None,
            downloads: 8,
            published_at: "2023-12-01".to_string(),
        },
    ];

    let html = render_detail(module, Vec::new(), Vec::new(), versions, Vec::new(), false);

    assert!(html.contains("[ VERSION HISTORY ]"));
    assert!(html.contains("Version history"));
    assert!(html.contains("v1.2.0"));
    assert!(html.contains("Added alerts"));
    assert!(html.contains("v1.1.0"));
    assert!(html.contains("No changelog"));
}

#[test]
fn module_detail_renders_empty_versions_state() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("No versions yet"));
}

#[test]
fn module_detail_renders_reviews() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let reviews = vec![sample_review(
        "alice",
        5,
        Some("Solid module"),
        Some("Loved the new forecast layout."),
    )];

    let html = render_detail(module, Vec::new(), Vec::new(), Vec::new(), reviews, false);

    assert!(html.contains("[ REVIEWS ]"));
    assert!(html.contains("Reviews"));
    assert!(html.contains("alice"));
    assert!(html.contains("5/5"));
    assert!(html.contains("Solid module"));
    assert!(html.contains("Loved the new forecast layout."));
    assert!(html.contains("href=\"/users/alice\""));
}

#[test]
fn module_detail_renders_empty_reviews_state() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );

    assert!(html.contains("No reviews yet"));
}

#[test]
fn module_detail_orders_sections_after_versions() {
    let module = sample_module("weather-wttr@barforge", "Weather");
    let html = render_detail(
        module,
        Vec::new(),
        Vec::new(),
        vec![VersionHistoryEntry {
            version: "1.2.0".to_string(),
            changelog: Some("Added alerts".to_string()),
            downloads: 12,
            published_at: "2024-01-01".to_string(),
        }],
        vec![sample_review("alice", 5, None, None)],
        false,
    );

    let versions_index = html.find("[ VERSION HISTORY ]").expect("missing versions");
    let reviews_index = html.find("[ REVIEWS ]").expect("missing reviews");
    let related_index = html
        .find("[ RELATED MODULES ]")
        .expect("missing related modules");

    assert!(versions_index < reviews_index);
    assert!(reviews_index < related_index);
}

fn render_detail(
    module: RegistryModule,
    screenshots: Vec<String>,
    related_modules: Vec<RegistryModule>,
    versions: Vec<VersionHistoryEntry>,
    reviews: Vec<Review>,
    related_loading: bool,
) -> String {
    dioxus_ssr::render_element(rsx!(ModuleDetail {
        module,
        screenshots,
        related_modules,
        versions,
        reviews,
        related_loading,
        now: reference_time(),
    }))
}

fn reference_time() -> OffsetDateTime {
    OffsetDateTime::parse("2024-01-10T00:00:00Z", &Rfc3339).expect("valid reference time")
}

fn sample_review(username: &str, rating: i64, title: Option<&str>, body: Option<&str>) -> Review {
    Review {
        id: 1,
        rating,
        title: title.map(str::to_string),
        body: body.map(str::to_string),
        helpful_count: 2,
        user: ReviewUser {
            username: username.to_string(),
            avatar_url: None,
        },
        created_at: "2025-01-02T00:00:00Z".to_string(),
        updated_at: None,
    }
}
