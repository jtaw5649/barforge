use barforge_web::{App, SessionStateOverride};
use dioxus::prelude::*;
use dioxus_history::{History, MemoryHistory, provide_history_context};
use std::rc::Rc;

#[test]
fn renders_home_route() {
    let html = render_route("/");

    assert!(html.contains("Barforge"));
    assert!(html.contains("Desktop manager for Waybar modules"));
    assert!(html.contains("[ FEATURES ]"));
    assert!(html.contains("[ DESKTOP APP ]"));
    assert!(html.contains("[ NAVIGATION ]"));
    assert!(html.contains("Browse Modules"));
    assert!(html.contains("Publish a Module"));
}

#[test]
fn renders_home_route_with_inline_icons() {
    let html = render_route("/");

    assert!(html.contains("class=\"feature-icon\""));
    assert!(html.contains("class=\"app-feature-icon\""));
    assert!(html.contains("class=\"nav-card-icon-svg\""));
    assert!(html.contains("class=\"nav-card-arrow-icon\""));
    assert!(html.contains("class=\"panel-link-icon\""));
}

#[test]
fn renders_not_found_route() {
    let html = render_route("/does-not-exist");

    assert!(html.contains("Page not found"));
    assert!(html.contains("/does-not-exist"));
}

#[test]
fn renders_tui_headers_on_core_routes() {
    let html = render_route("/login");
    assert!(html.contains("[ LOGIN ]"));

    let html = render_route("/dashboard");
    assert!(html.contains("[ PROFILE ]"));
    assert!(html.contains("[ MODULES ]"));
    assert!(html.contains("[ COLLECTIONS ]"));

    let html = render_route("/stars");
    assert!(html.contains("[ STARS ]"));

    let html = render_route("/collections/ops-essentials");
    assert!(html.contains("[ COLLECTION ]"));
    assert!(html.contains("[ MODULES ]"));

    let html = render_route("/users/barforge");
    assert!(html.contains("[ PROFILE ]"));
    assert!(html.contains("[ MODULES ]"));
    assert!(html.contains("[ COLLECTIONS ]"));

    let html = render_route("/upload");
    assert!(html.contains("[ UPLOAD ]"));

    let html = render_route("/admin");
    assert!(html.contains("[ MANAGE SUBMISSIONS ]"));
    assert!(html.contains("[ PENDING SUBMISSIONS ]"));

    let html = render_route("/settings/profile");
    assert!(html.contains("[ SETTINGS ]"));
    assert!(html.contains("[ PROFILE SETTINGS ]"));

    let html = render_route("/settings/notifications");
    assert!(html.contains("[ SETTINGS ]"));
    assert!(html.contains("[ NOTIFICATIONS ]"));

    let html = render_route("/settings/security");
    assert!(html.contains("[ SETTINGS ]"));
    assert!(html.contains("[ SECURITY ]"));

    let html = render_route("/terms");
    assert!(html.contains("[ TERMS ]"));

    let html = render_route("/privacy");
    assert!(html.contains("[ PRIVACY ]"));

    let html = render_route("/does-not-exist");
    assert!(html.contains("[ NOT FOUND ]"));
}

#[test]
fn renders_app_shell() {
    let html = render_route("/");

    assert!(html.contains("Skip to main content"));
    assert!(html.contains("Modules"));
    assert!(!html.contains("Upload"));
    assert!(!html.contains("Dashboard"));
    assert!(html.contains("Search modules..."));
    assert!(html.contains("svg class=\"header-search-icon\""));
    assert!(html.contains("svg class=\"theme-toggle-icon\""));
    assert!(html.contains("svg class=\"mobile-search-icon\""));
    assert!(html.contains("svg class=\"mobile-menu-icon\""));
    assert!(html.contains("Toggle theme"));
    assert!(html.contains("Log In"));
    assert!(html.contains("mobile-controls"));
    assert!(html.contains("aria-label=\"Open command palette\""));
    assert!(html.contains("aria-label=\"Open menu\""));
    assert!(html.contains("id=\"mobile-menu\""));
    assert!(html.contains("palette-backdrop"));
    assert!(html.contains("aria-label=\"Command palette\""));
    assert!(html.contains("favicon.ico"));
    assert!(html.contains("apple-touch-icon.png"));
    assert!(html.contains("rel=\"prefetch\" href=\"/modules\""));
    assert!(html.contains("rel=\"prefetch\" href=\"/upload\""));
    assert!(html.contains("Terms"));
    assert!(html.contains("Privacy"));
    assert!(html.contains("GPL-3.0"));
}

#[test]
fn renders_login_link_with_redirect() {
    let html = render_route("/modules/search?q=clock");

    assert!(html.contains("href=\"/login?redirect_to=/modules/search%3Fq%3Dclock\""));
}

#[test]
fn renders_app_shell_authenticated_menu() {
    let html = render_route_with_session(
        "/",
        SessionStateOverride {
            authenticated: true,
            is_admin: false,
            login: Some("barforge".to_string()),
            email: None,
        },
    );

    assert!(html.contains("Dashboard"));
    assert!(html.contains("Your profile"));
    assert!(html.contains("Your modules"));
    assert!(html.contains("Your stars"));
    assert!(html.contains("Upload module"));
    assert!(html.contains("Settings"));
    assert!(html.contains("Log out"));
    assert!(!html.contains("Log In"));
    assert!(html.contains("svg class=\"notification-bell-icon\""));
}

#[test]
fn renders_modules_index_route() {
    let html = render_route("/modules");

    assert!(html.contains("Modules"));
    assert!(html.contains("Browse by Category"));
    assert!(html.contains("[ CATEGORIES ]"));
    assert!(html.contains("Weather"));
    assert!(html.contains("Featured Modules"));
    assert!(html.contains("[ FEATURED MODULES ]"));
    assert!(html.contains("Hand-picked by our team"));
    assert!(html.contains("Popular Modules"));
    assert!(html.contains("[ POPULAR MODULES ]"));
    assert!(html.contains("Top downloads and community favorites"));
    assert!(html.contains("href=\"/modules/search?sort=popular\""));
    assert!(html.contains("Recently Added"));
    assert!(html.contains("[ RECENTLY ADDED ]"));
    assert!(html.contains("Fresh modules, straight from the registry"));
    assert!(html.contains("href=\"/modules/search?sort=recent\""));
    assert!(html.contains("class=\"browse-discover\""));
    assert!(html.contains("See all"));
    assert!(html.contains("href=\"/modules/search\""));
    assert!(html.contains("Discover"));
    assert!(html.contains("Search"));
    assert!(html.contains("class=\"browse-tab active\" href=\"/modules\""));
    assert!(html.contains("class=\"browse-tab\" href=\"/modules/search\""));
    assert!(html.contains("browse-tab-bar"));
    assert!(html.contains("browse-tabs"));
    assert!(html.contains("role=\"tablist\""));
    assert!(html.contains("role=\"tab\""));
    assert!(html.contains("id=\"modules-tab-discover\""));
    assert!(html.contains("id=\"modules-tab-search\""));
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("aria-selected=\"false\""));
    assert!(html.contains("aria-controls=\"modules-discover-panel\""));
    assert!(html.contains("aria-controls=\"modules-search-panel\""));
    assert!(html.contains("role=\"tabpanel\""));
    assert!(html.contains("id=\"modules-discover-panel\""));
    assert!(html.contains("aria-labelledby=\"modules-tab-discover\""));
}

#[test]
fn renders_modules_route_with_session_context() {
    let html = render_route_with_session(
        "/modules",
        SessionStateOverride {
            authenticated: true,
            is_admin: false,
            login: Some("barforge".to_string()),
            email: None,
        },
    );

    assert!(html.contains("Dashboard"));
    assert!(html.contains("Log out"));
    assert!(!html.contains("Log In"));
}

#[test]
fn renders_modules_search_route() {
    let html = render_route("/modules/search");

    assert!(html.contains("Modules"));
    assert!(html.contains("Browse Modules"));
    assert!(html.contains("modules-search-header"));
    assert!(html.contains("class=\"modules-search\""));
    assert!(html.contains("modules-search-filters"));
    assert!(html.contains("modules-search-results"));
    assert!(html.contains("[ SEARCH ]"));
    assert!(html.contains("[ FILTERS ]"));
    assert!(html.contains("[ RESULTS ]"));
    assert!(html.contains("class=\"filter-input\""));
    assert!(html.contains("class=\"filter-input-icon\""));
    assert!(html.contains("svg class=\"filter-input-icon\""));
    assert!(html.contains("class=\"mobile-filter-toggle\""));
    assert!(html.contains("class=\"view-toggle\""));
    assert!(html.contains("aria-label=\"View mode\""));
    assert!(html.contains("Discover"));
    assert!(html.contains("Search"));
    assert!(html.contains("class=\"browse-tab\" href=\"/modules\""));
    assert!(html.contains("class=\"browse-tab active\" href=\"/modules/search\""));
    assert!(html.contains("browse-tab-bar"));
    assert!(html.contains("browse-tabs"));
    assert!(html.contains("Category"));
    assert!(html.contains("Sort By"));
    assert!(html.contains("name=\"q\""));
    assert!(html.contains("Weather"));
    assert!(html.contains("role=\"tablist\""));
    assert!(html.contains("role=\"tab\""));
    assert!(html.contains("id=\"modules-tab-discover\""));
    assert!(html.contains("id=\"modules-tab-search\""));
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("aria-selected=\"false\""));
    assert!(html.contains("aria-controls=\"modules-discover-panel\""));
    assert!(html.contains("aria-controls=\"modules-search-panel\""));
    assert!(html.contains("role=\"tabpanel\""));
    assert!(html.contains("id=\"modules-search-panel\""));
    assert!(html.contains("aria-labelledby=\"modules-tab-search\""));
}

#[test]
fn renders_modules_search_sidebar_toggle() {
    let html = render_route("/modules/search");

    assert!(html.contains("class=\"sidebar-toggle\""));
    assert!(html.contains("aria-controls=\"filter-sidebar\""));
}

#[test]
fn renders_modules_search_query() {
    let html = render_route("/modules/search?q=clock");

    assert!(html.contains("href=\"/modules/clock-time@barforge\""));
    assert!(!html.contains("href=\"/modules/weather-wttr@barforge\""));
    assert!(html.contains("value=\"clock\""));
}

#[test]
fn renders_modules_search_page_param() {
    let html = render_route("/modules/search?q=clock&page=2");

    assert!(html.contains("No modules found"));
}

#[test]
fn hides_modules_search_pagination_when_single_page() {
    let html = render_route("/modules/search");

    assert!(!html.contains("aria-label=\"Pagination\""));
    assert!(!html.contains("page=2"));
}

#[test]
fn renders_modules_search_category_filter() {
    let html = render_route("/modules/search?category=weather");

    assert!(html.contains("href=\"/modules/weather-wttr@barforge\""));
    assert!(!html.contains("href=\"/modules/clock-time@barforge\""));
}

#[test]
fn renders_modules_search_sort_downloads() {
    let html = render_route("/modules/search?sort=downloads");
    let clock_index = html
        .find("href=\"/modules/clock-time@barforge\"")
        .expect("Clock not found");
    let weather_index = html
        .find("href=\"/modules/weather-wttr@barforge\"")
        .expect("Weather not found");

    assert!(weather_index < clock_index);
}

#[test]
fn renders_modules_search_sort_popular() {
    let html = render_route("/modules/search?sort=popular");
    let clock_index = html.find("Clock").expect("Clock not found");
    let weather_index = html.find("Weather").expect("Weather not found");

    assert!(clock_index < weather_index);
}

#[test]
fn renders_modules_search_sort_trending() {
    let html = render_route("/modules/search?sort=trending");
    let clock_index = html.find("Clock").expect("Clock not found");
    let weather_index = html.find("Weather").expect("Weather not found");

    assert!(clock_index < weather_index);
}

#[test]
fn renders_modules_search_sort_recent() {
    let html = render_route("/modules/search?sort=recent");
    let clock_index = html.find("Clock").expect("Clock not found");
    let weather_index = html.find("Weather").expect("Weather not found");

    assert!(clock_index < weather_index);
}

#[test]
fn renders_module_detail_route() {
    let html = render_route("/modules/weather-wttr@barforge");

    assert!(html.contains("Weather"));
    assert!(html.contains("Version history"));
}

#[test]
fn invalid_module_route_renders_not_found() {
    let html = render_route("/modules/invalid");

    assert!(html.contains("Page not found"));
    assert!(html.contains("/modules/invalid"));
}

#[test]
fn renders_module_detail_screenshots_use_local_assets() {
    let html = render_route("/modules/weather-wttr@barforge");

    assert!(html.contains("module-screenshot-1.svg"));
    assert!(html.contains("module-screenshot-2.svg"));
    assert!(!html.contains("assets.barforge.dev"));
}

#[test]
fn renders_user_profile_route() {
    let html = render_route("/users/barforge");

    assert!(html.contains("User Profile"));
    assert!(html.contains("@barforge"));
    assert!(html.contains("Modules by Barforge"));
    assert!(html.contains("class=\"profile-socials\""));
    assert!(html.matches("class=\"profile-social-icon\"").count() >= 2);
}

#[test]
fn invalid_user_route_renders_not_found() {
    let html = render_route("/users/bad--name");

    assert!(html.contains("Page not found"));
    assert!(html.contains("/users/bad--name"));
}

#[test]
fn renders_user_profile_collections() {
    let html = render_route("/users/barforge");

    assert!(html.contains("Collections"));
    assert!(html.contains("Ops Essentials"));
}

#[test]
fn renders_login_route() {
    let html = render_route("/login");

    assert!(html.contains("Log in to Barforge"));
    assert!(html.contains("Continue with GitHub"));
    assert!(html.contains("href=\"/auth/github\""));
    assert!(html.contains("Terms of Service"));
    assert!(html.contains("Privacy Policy"));
    assert!(html.contains("svg class=\"github-icon\""));
}

#[test]
fn renders_login_route_with_redirect() {
    let html = render_route("/login?redirect_to=/dashboard");

    assert!(html.contains("href=\"/auth/github?redirect_to=/dashboard\""));
}

#[test]
fn renders_login_route_rejects_unsafe_redirect() {
    let html = render_route("/login?redirect_to=https://evil.com");

    assert!(!html.contains("redirect_to="));
}

#[test]
fn renders_dashboard_route() {
    let html = render_route("/dashboard");

    assert!(html.contains("Dashboard"));
    assert!(html.contains("Profile overview"));
    assert!(html.contains("Total downloads"));
    assert!(html.contains("Your modules"));
    assert!(html.contains("Collections"));
    assert!(html.contains("Ops Essentials"));
}

#[test]
fn renders_dashboard_profile_action_form() {
    let html = render_route("/dashboard");

    assert!(html.contains("data-dashboard-action=\"profile-update\""));
    assert!(html.contains("action=\"/api/users/me\""));
    assert!(html.contains("data-csrf-endpoint=\"/api/csrf-token\""));
    assert!(html.contains("name=\"display_name\""));
    assert!(html.contains("name=\"bio\""));
    assert!(html.contains("name=\"website_url\""));
}

#[test]
fn renders_dashboard_collection_action_forms() {
    let html = render_route("/dashboard");

    assert!(html.contains("data-dashboard-action=\"collection-create\""));
    assert!(html.contains("action=\"/api/collections\""));
    assert!(html.contains("name=\"collection_name\""));
    assert!(html.contains("name=\"collection_visibility\""));
    assert!(html.contains("data-dashboard-action=\"collection-update\""));
    assert!(html.contains("action=\"/api/collections/1\""));
    assert!(html.contains("data-dashboard-action=\"collection-delete\""));
}

#[test]
fn renders_stars_route() {
    let html = render_route("/stars");

    assert!(html.contains("Starred modules"));
    assert!(html.contains("No starred modules yet"));
    assert!(html.contains("Log in to sync your stars"));
    assert!(html.contains("href=\"/login?redirect_to=/stars\""));
}

#[test]
fn renders_collection_detail_route() {
    let html = render_route("/collections/ops-essentials");

    assert!(html.contains("Ops Essentials"));
    assert!(html.contains("Modules for steady-state ops dashboards."));
    assert!(html.contains("collection-modules"));
    assert!(html.contains("Weather"));
}

#[test]
fn invalid_collection_route_renders_not_found() {
    let html = render_route("/collections/bad$$");

    assert!(html.contains("Page not found"));
    assert!(html.contains("/collections/bad$$"));
}

#[test]
fn renders_upload_route() {
    let html = render_route("/upload");

    assert!(html.contains("Upload module"));
    assert!(html.contains("Share your module with the community"));
    assert!(html.contains("name=\"name\""));
    assert!(html.contains("name=\"description\""));
    assert!(html.contains("name=\"category\""));
    assert!(html.contains("name=\"repo_url\""));
    assert!(html.contains("name=\"version\""));
    assert!(html.contains("name=\"license\""));
    assert!(html.contains("name=\"package_file\""));
    assert!(html.contains("name=\"changelog\""));
}

#[test]
fn renders_upload_form_label_associations() {
    let html = render_route("/upload");

    assert!(html.contains("for=\"upload-name\""));
    assert!(html.contains("id=\"upload-name\""));
    assert!(html.contains("for=\"upload-description\""));
    assert!(html.contains("id=\"upload-description\""));
    assert!(html.contains("for=\"upload-category\""));
    assert!(html.contains("id=\"upload-category\""));
    assert!(html.contains("for=\"upload-repo-url\""));
    assert!(html.contains("id=\"upload-repo-url\""));
    assert!(html.contains("for=\"upload-version\""));
    assert!(html.contains("id=\"upload-version\""));
    assert!(html.contains("for=\"upload-license\""));
    assert!(html.contains("id=\"upload-license\""));
    assert!(html.contains("for=\"upload-package\""));
    assert!(html.contains("id=\"upload-package\""));
    assert!(html.contains("for=\"upload-changelog\""));
    assert!(html.contains("id=\"upload-changelog\""));
}

#[test]
fn renders_upload_form_submission_attributes() {
    let html = render_route("/upload");

    assert!(html.contains("method=\"post\""));
    assert!(html.contains("enctype=\"multipart/form-data\""));
    assert!(html.contains("data-upload-endpoint=\"/api/upload\""));
    assert!(html.contains("data-csrf-endpoint=\"/api/csrf-token\""));
}
#[test]
fn renders_admin_route() {
    let html = render_route("/admin");

    assert!(html.contains("Admin dashboard"));
    assert!(html.contains("Manage submissions"));
    assert!(html.contains("Total modules"));
    assert!(html.contains("Total users"));
    assert!(html.contains("Pending submissions"));
    assert!(html.contains("No pending submissions"));
}

#[test]
fn renders_barforge_route() {
    let html = render_route("/barforge");

    assert!(html.contains("Barforge Ecosystem"));
    assert!(html.contains("[ REPOSITORIES ]"));
    assert!(html.contains("[ TECH STACK ]"));
    assert!(html.contains("Stars"));
    assert!(html.contains("Forks"));
    assert!(html.contains("Contributors"));
    assert!(html.contains("barforge-web"));
    assert!(html.contains("barforge-app"));
    assert!(html.contains("class=\"stat-icon-svg\""));
    assert!(html.contains("class=\"repo-icon-svg\""));
    assert!(html.contains("class=\"repo-arrow-icon\""));
    assert!(html.contains("class=\"tech-icon-svg\""));
}

#[test]
fn renders_terms_route() {
    let html = render_route("/terms");

    assert!(html.contains("Terms of Service"));
    assert!(html.contains("Effective: December 27, 2025"));
    assert!(html.contains("1. About Barforge"));
}

#[test]
fn renders_privacy_route() {
    let html = render_route("/privacy");

    assert!(html.contains("Privacy Policy"));
    assert!(html.contains("Effective: December 27, 2025"));
    assert!(html.contains("1. Information We Collect"));
}

#[test]
fn renders_settings_route() {
    let html = render_route("/settings");

    assert!(html.contains("Settings"));
    assert!(html.contains("Redirecting to settings profile"));
}

#[test]
fn renders_settings_profile_route() {
    let html = render_route("/settings/profile");

    assert!(html.contains("Profile Settings"));
    assert!(html.contains("Manage your public profile"));
}

#[test]
fn renders_settings_profile_form_fields() {
    let html = render_route("/settings/profile");

    assert!(html.contains("Display Name"));
    assert!(html.contains("Bio"));
    assert!(html.contains("Website"));
    assert!(html.contains("GitHub"));
    assert!(html.contains("X"));
    assert!(html.contains("Bluesky"));
    assert!(html.contains("Discord"));
    assert!(html.contains("Sponsor"));
    assert!(html.contains("name=\"display_name\""));
    assert!(html.contains("name=\"bio\""));
    assert!(html.contains("name=\"website_url\""));
    assert!(html.contains("name=\"github_url\""));
    assert!(html.contains("name=\"twitter_url\""));
    assert!(html.contains("name=\"bluesky_url\""));
    assert!(html.contains("name=\"discord_url\""));
    assert!(html.contains("name=\"sponsor_url\""));
}

#[test]
fn renders_settings_profile_prefills_sample_data() {
    let html = render_route("/settings/profile");

    assert!(html.contains("value=\"Barforge\""));
    assert!(html.contains("Building the Barforge ecosystem."));
    assert!(html.contains("value=\"https://barforge.dev\""));
    assert!(html.contains("value=\"https://github.com/barforge\""));
    assert!(html.contains("value=\"https://x.com/barforge\""));
    assert!(html.contains("value=\"https://bsky.app/profile/barforge.dev\""));
    assert!(html.contains("value=\"https://discord.gg/barforge\""));
}

#[test]
fn renders_settings_profile_form_submission_attributes() {
    let html = render_route("/settings/profile");

    assert!(html.contains("action=\"/api/users/me\""));
    assert!(html.contains("method=\"post\""));
    assert!(html.contains("data-profile-endpoint=\"/api/users/me\""));
    assert!(html.contains("data-csrf-endpoint=\"/api/csrf-token\""));
}

#[test]
fn renders_settings_profile_label_associations() {
    let html = render_route("/settings/profile");

    assert!(html.contains("for=\"settings-display-name\""));
    assert!(html.contains("id=\"settings-display-name\""));
    assert!(html.contains("for=\"settings-bio\""));
    assert!(html.contains("id=\"settings-bio\""));
    assert!(html.contains("for=\"settings-website\""));
    assert!(html.contains("id=\"settings-website\""));
    assert!(html.contains("for=\"settings-github\""));
    assert!(html.contains("id=\"settings-github\""));
    assert!(html.contains("for=\"settings-x\""));
    assert!(html.contains("id=\"settings-x\""));
    assert!(html.contains("for=\"settings-bluesky\""));
    assert!(html.contains("id=\"settings-bluesky\""));
    assert!(html.contains("for=\"settings-discord\""));
    assert!(html.contains("id=\"settings-discord\""));
    assert!(html.contains("for=\"settings-sponsor\""));
    assert!(html.contains("id=\"settings-sponsor\""));
}

#[test]
fn renders_settings_navigation() {
    let html = render_route("/settings/profile");

    assert!(html.contains("Profile"));
    assert!(html.contains("Notifications"));
    assert!(html.contains("Security"));
    assert!(html.contains("Back to Dashboard"));
    assert!(html.contains("settings-back"));
    assert!(html.contains("svg class=\"settings-icon\""));
    assert!(html.contains("svg class=\"settings-back-icon\""));
}

#[test]
fn renders_settings_notifications_route() {
    let html = render_route("/settings/notifications");

    assert!(html.contains("Notifications"));
    assert!(html.contains("Manage how and when you receive notifications."));
    assert!(html.contains("Email"));
    assert!(html.contains("In-app"));
    assert!(html.contains("Download milestones"));
    assert!(
        html.contains(
            "Get notified when your modules reach download milestones (100, 1k, 10k, etc.)"
        )
    );
    assert!(html.contains("New comments"));
    assert!(html.contains("When someone comments on your modules"));
    assert!(html.contains("New stars"));
    assert!(html.contains("When someone stars your modules"));
    assert!(html.contains("Module updates"));
    assert!(html.contains("When modules you starred release new versions"));
    assert!(html.contains("Platform announcements"));
    assert!(html.contains("Important updates about Barforge"));
    assert!(html.contains("name=\"notify_downloads_email\""));
    assert!(html.contains("name=\"notify_downloads_inapp\""));
    assert!(html.contains("name=\"notify_comments_email\""));
    assert!(html.contains("name=\"notify_comments_inapp\""));
    assert!(html.contains("name=\"notify_stars_email\""));
    assert!(html.contains("name=\"notify_stars_inapp\""));
    assert!(html.contains("name=\"notify_updates_email\""));
    assert!(html.contains("name=\"notify_updates_inapp\""));
    assert!(html.contains("name=\"notify_announcements_email\""));
    assert!(html.contains("name=\"notify_announcements_inapp\""));
    assert!(html.contains("Save preferences"));
    assert!(html.contains("toggle-track"));
    assert!(html.contains("toggle-thumb"));
    assert!(html.contains("notification-toggles"));
    assert!(html.contains("form-actions"));
    assert!(html.contains("svg class=\"notification-icon\""));
    assert!(html.contains("svg class=\"notification-channel-icon\""));
}

#[test]
fn renders_settings_security_route() {
    let html = render_route("/settings/security");

    assert!(html.contains("Security Settings"));
    assert!(html.contains("Connected Accounts"));
    assert!(html.contains("Sessions"));
    assert!(html.contains("Data Export"));
    assert!(html.contains("Danger Zone"));
    assert!(html.contains("Export Data"));
    assert!(html.contains("Delete Account"));
}

fn render_route(path: &str) -> String {
    dioxus_ssr::render_element(rsx!(TestApp {
        route: path.to_string()
    }))
}

fn render_route_with_session(path: &str, session: SessionStateOverride) -> String {
    dioxus_ssr::render_element(rsx!(TestAppWithSession {
        route: path.to_string(),
        session
    }))
}

#[component]
fn TestApp(route: String) -> Element {
    let history = Rc::new(MemoryHistory::default());
    history.replace(route);
    provide_history_context(history);

    rsx!(App {})
}

#[component]
fn TestAppWithSession(route: String, session: SessionStateOverride) -> Element {
    let history = Rc::new(MemoryHistory::default());
    history.replace(route);
    provide_history_context(history);
    use_context_provider(|| session);

    rsx!(App {})
}
