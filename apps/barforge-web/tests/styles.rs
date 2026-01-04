use std::path::Path;

const MAIN_CSS: &str = include_str!("../assets/main.css");

#[test]
fn main_css_includes_light_theme_and_fonts() {
    assert!(MAIN_CSS.contains("[data-theme='light']"));
    assert!(MAIN_CSS.contains("Space Grotesk"));
    assert!(MAIN_CSS.contains("Plus Jakarta Sans"));
}

#[test]
fn main_css_matches_catppuccin_palette_tokens() {
    assert!(MAIN_CSS.contains("--color-text-faint: #8f95aa;"));
    assert!(MAIN_CSS.contains("--color-text-faint: #5b6472;"));
    assert!(MAIN_CSS.contains("--color-grid: rgba(205, 214, 244, 0.06);"));
    assert!(MAIN_CSS.contains("--color-grid: rgba(76, 79, 105, 0.08);"));
    assert!(MAIN_CSS.contains(
        "--color-border-hover: color-mix(in srgb, var(--color-border) 70%, var(--color-text-faint));"
    ));
    assert!(MAIN_CSS.contains("--shadow-glow: 0 0 28px rgba(138, 123, 255, 0.4);"));
    assert!(MAIN_CSS.contains("--shadow-glow: 0 0 20px rgba(136, 57, 239, 0.15);"));
    assert!(MAIN_CSS.contains("--radius-full: 999px;"));
}

#[test]
fn main_css_defines_container_lg() {
    assert!(MAIN_CSS.contains("--container-lg:"));
}

#[test]
fn main_css_includes_command_palette_and_theme_toggle_styles() {
    assert!(MAIN_CSS.contains(".palette-backdrop"));
    assert!(MAIN_CSS.contains(".palette-results"));
    assert!(MAIN_CSS.contains(".theme-toggle"));
    assert!(MAIN_CSS.contains(".icon-wrapper"));
}

#[test]
fn main_css_includes_footer_styles() {
    assert!(MAIN_CSS.contains(".footer-nav"));
    assert!(MAIN_CSS.contains(".github-link"));
}

#[test]
fn main_css_includes_header_and_notification_styles() {
    assert!(MAIN_CSS.contains(".header-container"));
    assert!(MAIN_CSS.contains(".nav-links"));
    assert!(MAIN_CSS.contains(".search-trigger"));
    assert!(MAIN_CSS.contains(".mobile-menu"));
    assert!(MAIN_CSS.contains(".dropdown-menu"));
    assert!(MAIN_CSS.contains(".notification-center"));
    assert!(MAIN_CSS.contains(".notification-dropdown"));
}

#[test]
fn main_css_includes_login_search_and_notification_variants() {
    assert!(MAIN_CSS.contains(".github-btn"));
    assert!(MAIN_CSS.contains("background-color: #24292e;"));
    assert!(MAIN_CSS.contains("border-radius: var(--radius-md);"));
    assert!(MAIN_CSS.contains(".notification-icon"));
    assert!(MAIN_CSS.contains("width: 40px;"));
    assert!(MAIN_CSS.contains("background-color: var(--color-bg-elevated);"));
    assert!(MAIN_CSS.contains(".notification-description"));
    assert!(MAIN_CSS.contains("font-size: var(--font-size-xs);"));
    assert!(MAIN_CSS.contains(".search-sm"));
    assert!(MAIN_CSS.contains("font-size: 0.85rem;"));
    assert!(MAIN_CSS.contains(".terms"));
    assert!(MAIN_CSS.contains("font-size: 0.75rem;"));
}

#[test]
fn main_css_includes_browse_tabs_and_install_snippet_states() {
    assert!(MAIN_CSS.contains(".browse-tab-bar"));
    assert!(MAIN_CSS.contains(".browse-tabs"));
    assert!(MAIN_CSS.contains(".browse-tab"));
    assert!(MAIN_CSS.contains(".home-section"));
    assert!(MAIN_CSS.contains(".copy-btn.copied"));
    assert!(MAIN_CSS.contains(".terminal-cmd.show-full"));
}

#[test]
fn main_css_includes_modules_search_grid_layout() {
    assert!(MAIN_CSS.contains(".modules-search {"));
    assert!(MAIN_CSS.contains("\tdisplay: grid;"));
    assert!(MAIN_CSS.contains("grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);"));
    assert!(MAIN_CSS.contains("\talign-items: start;"));
    assert!(MAIN_CSS.contains("@media (max-width: 960px)"));
    assert!(MAIN_CSS.contains("grid-template-columns: 1fr;"));
}

#[test]
fn main_css_includes_modules_search_panels() {
    assert!(MAIN_CSS.contains(".modules-search-header {"));
    assert!(MAIN_CSS.contains(".modules-search-filters {"));
    assert!(MAIN_CSS.contains(".modules-search-results {"));
    assert!(MAIN_CSS.contains(".modules-search-controls {"));
}

#[test]
fn main_css_expands_modules_layout_width() {
    assert!(MAIN_CSS.contains(".modules-layout {"));
    let layout_block = MAIN_CSS
        .split(".modules-layout {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(layout_block.contains("\twidth: 100%;"));
}

#[test]
fn main_css_includes_browse_filter_radii() {
    let filter_input_block = MAIN_CSS
        .split(".filter-input {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(filter_input_block.contains("border-radius: var(--radius-md);"));

    let clear_filters_block = MAIN_CSS
        .rsplit(".clear-filters-btn {")
        .next()
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(clear_filters_block.contains("border-radius: var(--radius-md);"));

    let filter_option_block = MAIN_CSS
        .split(".filter-option {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(filter_option_block.contains("border-radius: var(--radius-md);"));

    let sort_select_block = MAIN_CSS
        .split(".sort-select {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(sort_select_block.contains("border-radius: var(--radius-md);"));

    let sort_focus_block = MAIN_CSS
        .split(".sort-select:focus {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(sort_focus_block.contains("box-shadow: var(--focus-ring);"));

    let filter_count_block = MAIN_CSS
        .split(".filter-count {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(filter_count_block.contains("border-radius: var(--radius-md);"));

    let collapsed_badge_block = MAIN_CSS
        .split(".collapsed-filter-badge {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(collapsed_badge_block.contains("border-radius: var(--radius-md);"));
}

#[test]
fn main_css_styles_pagination_state() {
    let pagination_block = MAIN_CSS
        .split(".pagination {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(pagination_block.contains("padding-top: var(--space-xl);"));
    assert!(pagination_block.contains("border-top: 1px solid"));

    let pagination_active_block = MAIN_CSS
        .split(".pagination-page.is-active {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(pagination_active_block.contains("background-color: var(--color-primary);"));

    let pagination_disabled_block = MAIN_CSS
        .split(".pagination-btn.is-disabled {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(pagination_disabled_block.contains("opacity: 0.5;"));
    assert!(pagination_disabled_block.contains("cursor: not-allowed;"));
}

#[test]
fn main_css_polishes_module_detail_reviews_and_related() {
    let review_meta_block = MAIN_CSS
        .split(".review-meta {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(review_meta_block.contains("flex-wrap: wrap;"));

    let review_rating_block = MAIN_CSS
        .split(".review-rating {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(review_rating_block.contains("border: var(--border-tui);"));

    let review_helpful_block = MAIN_CSS
        .split(".review-helpful {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(review_helpful_block.contains("margin-left: auto;"));

    let related_block = MAIN_CSS
        .split(".related-grid {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(related_block.contains("gap: var(--space-lg);"));
}

#[test]
fn main_css_styles_module_star_primary() {
    let star_block = MAIN_CSS
        .split(".module-star {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(star_block.contains("color: var(--color-primary);"));
}

#[test]
fn main_css_polishes_screenshot_placeholders_and_carousel() {
    let watermark_block = MAIN_CSS
        .split(".screenshot-watermark {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(watermark_block.contains("border: var(--border-tui);"));
    assert!(watermark_block.contains("padding: var(--space-xs) var(--space-sm);"));

    let carousel_dialog_block = MAIN_CSS
        .split(".screenshot-carousel-dialog {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(carousel_dialog_block.contains("box-shadow: var(--shadow-lg);"));

    let carousel_nav_block = MAIN_CSS
        .split(".carousel-nav {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(carousel_nav_block.contains("border-radius: var(--radius-sm);"));
}

#[test]
fn main_css_aligns_module_list_rows_and_empty_state() {
    let version_badge_block = MAIN_CSS
        .split(".version-badge {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(version_badge_block.contains("font-family: var(--font-mono);"));
    assert!(version_badge_block.contains("letter-spacing: 0.05em;"));
    assert!(version_badge_block.contains("border: var(--border-tui);"));

    let row_title_block = MAIN_CSS
        .split(".row-title {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(row_title_block.contains("flex-wrap: wrap;"));

    let empty_block = MAIN_CSS
        .split(".module-list-empty {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(empty_block.contains("border: var(--border-tui);"));
    assert!(empty_block.contains("padding: var(--space-xl);"));
}

#[test]
fn main_css_aligns_module_card_visuals() {
    let card_footer_block = MAIN_CSS
        .split(".module-card-footer {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(card_footer_block.contains("border-top: 1px solid"));
    assert!(card_footer_block.contains("padding-top: var(--space-sm);"));

    let icon_image_block = MAIN_CSS
        .split(".module-card-icon-image {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(icon_image_block.contains("object-fit: contain;"));
    assert!(icon_image_block.contains("padding: 6px;"));

    let stats_block = MAIN_CSS
        .split(".module-stats {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(stats_block.contains("border-radius: var(--radius-md);"));
}

#[test]
fn main_css_avoids_pill_radii() {
    assert!(!MAIN_CSS.contains("border-radius: 999"));
}

#[test]
fn main_css_aligns_module_version_badge_with_tags() {
    let version_block = MAIN_CSS
        .split(".module-version {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(version_block.contains("display: inline-flex;"));
    assert!(version_block.contains("font-size: 0.625rem;"));
    assert!(version_block.contains("letter-spacing: 0.05em;"));
    assert!(version_block.contains("padding: var(--space-2xs) var(--space-sm);"));
}

#[test]
fn main_css_resets_module_list_and_rows() {
    let container_block = MAIN_CSS
        .split(".module-container {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(container_block.contains("list-style: none;"));
    assert!(container_block.contains("margin: 0;"));
    assert!(container_block.contains("padding: 0;"));

    let list_block = MAIN_CSS
        .split(".module-container.list {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(list_block.contains("align-items: stretch;"));
    assert!(list_block.contains("width: 100%;"));

    let row_block = MAIN_CSS
        .split(".row {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(row_block.contains("width: 100%;"));

    let row_wrapper_block = MAIN_CSS
        .split(".row-wrapper {")
        .nth(1)
        .unwrap_or_default()
        .split('}')
        .next()
        .unwrap_or_default();
    assert!(row_wrapper_block.contains("display: block;"));
    assert!(row_wrapper_block.contains("width: 100%;"));
}

#[test]
fn main_css_includes_module_screenshots_grid() {
    assert!(MAIN_CSS.contains(".module-screenshots {"));
    assert!(MAIN_CSS.contains("\tdisplay: block;"));
    assert!(MAIN_CSS.contains(".screenshots-grid {"));
    assert!(MAIN_CSS.contains("\tdisplay: grid;"));
    assert!(MAIN_CSS.contains("grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));"));
}

#[test]
fn main_css_includes_related_modules_grid_styles() {
    assert!(MAIN_CSS.contains(".related-grid {"));
    assert!(MAIN_CSS.contains("\talign-content: start;"));
    assert!(MAIN_CSS.contains("\talign-items: stretch;"));
    assert!(MAIN_CSS.contains("\tlist-style: none;"));
    assert!(MAIN_CSS.contains("\tmargin: 0;"));
    assert!(MAIN_CSS.contains("\tpadding: 0;"));
}

#[test]
fn main_css_includes_module_card_overlay_links() {
    assert!(MAIN_CSS.contains(".module-card-link {"));
    assert!(MAIN_CSS.contains(".row-link {"));
    assert!(MAIN_CSS.contains("\tposition: absolute;"));
}

#[test]
fn font_assets_exist() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("public/assets/fonts");
    let fonts = [
        "space-grotesk-400.ttf",
        "space-grotesk-500.ttf",
        "space-grotesk-600.ttf",
        "space-grotesk-700.ttf",
        "plus-jakarta-sans-400.ttf",
        "plus-jakarta-sans-500.ttf",
        "plus-jakarta-sans-600.ttf",
        "plus-jakarta-sans-700.ttf",
        "JetBrainsMono-Regular.ttf",
        "JetBrainsMono-Medium.ttf",
    ];

    for font in fonts {
        assert!(base.join(font).exists(), "missing font asset: {font}");
    }
}
