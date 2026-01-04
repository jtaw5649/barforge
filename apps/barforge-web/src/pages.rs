use barforge_types::{
    AdminStats, Collection, CollectionModule, CollectionOwner, LandingData, ModuleCategory,
    RegistryModule, Review, Screenshot, Submission, SubmissionsResponse, VersionHistoryEntry,
};
use barforge_web_ui::categories::{browse_categories, category_label};
use barforge_web_ui::components::{
    CategoryPills, ModuleDetail, ModuleList, ModuleSort, ModuleViewMode, PaginationControls,
    SearchToolbar,
};
use barforge_web_ui::filters::{DEFAULT_SORT, build_search_url, default_sort_options};
use dioxus::prelude::*;
use dioxus_fullstack::{FullstackContext, StatusCode};
use dioxus_router::navigation::NavigationTarget;
use dioxus_router::{Outlet, use_navigator, use_route};
use manganis::{Asset, asset};
use std::collections::HashSet;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::KeyboardEvent as WebKeyboardEvent;

use crate::api;
use crate::app_time::app_now;
use crate::auth_redirect;
use crate::auth_ui::{
    AdminGate, ProtectedRoute, github_auth_href, login_redirect_target, use_auth,
};
use crate::forms::{
    FieldError, field_error_element, field_error_message, form_value, profile_form_values,
    profile_request_spec, upload_error_summary, upload_form_values, upload_request_spec,
    validate_profile_form, validate_upload_form,
};
#[cfg(target_arch = "wasm32")]
use crate::forms::{
    ProfileRequestSpec, UpdateProfileRequest, UploadRequestSpec, update_profile_request,
};
use crate::notifications::{
    NotificationIcon, PreferenceField, SaveStatus, notification_description, notification_icon,
    notification_key, notification_label, preferences_or_default, use_notifications,
};
use crate::recently_viewed::{RecentModule, use_recently_viewed};
use crate::routes::{CollectionId, GithubUsername, ModuleSlug, Route};
use crate::sample_data::{
    sample_collection, sample_collection_modules, sample_fallback_module, sample_module_by_uuid,
    sample_modules, sample_related_modules, sample_screenshots, sample_user_profile,
    sample_versions,
};
use crate::stars::use_stars;
use crate::state::{
    RemoteState, admin_stats_from_state, admin_submissions_from_state, featured_modules_from_state,
    github_stats_from_state, landing_stats_from_state, modules_from_state, use_admin_stats_state,
    use_admin_submissions_state, use_collection_detail_state, use_collections_state,
    use_featured_state, use_github_stats_state, use_landing_state, use_module_detail_state,
    use_module_reviews_state, use_module_screenshots_state, use_module_versions_state,
    use_modules_mine_state, use_registry_index_state, use_related_modules_state,
    use_user_collections_state, use_user_modules_state, use_user_profile_me_state,
    use_user_profile_state, user_profile_view_from_state,
};

#[component]
fn StarredModuleList(
    modules: Vec<RegistryModule>,
    query: Option<String>,
    sort: Option<ModuleSort>,
    category: Option<ModuleCategory>,
    page: Option<usize>,
    per_page: Option<usize>,
    view_mode: Option<ModuleViewMode>,
    now: time::OffsetDateTime,
) -> Element {
    let stars = use_stars();
    let starred = stars.starred_set();
    let on_toggle = EventHandler::new(move |uuid: String| {
        stars.toggle(&uuid);
    });

    rsx! {
        ModuleList {
            modules,
            query,
            sort,
            category,
            page,
            per_page,
            starred: Some(starred),
            on_toggle_star: Some(on_toggle),
            view_mode,
            now,
        }
    }
}

#[component]
pub(crate) fn Home() -> Element {
    let landing_state = use_landing_state();
    let stats = landing_stats_from_state(&landing_state);
    let downloads_label = format_number(stats.total_downloads);
    let retry_href = Route::Home {}.to_string();

    rsx! {
        section { class: "home",
            {landing_status_banner(&landing_state, &retry_href)}
            section { class: "tui-panel hero-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ BARFORGE ]" }
                    span { class: "tui-panel-status", "v1.0" }
                }
                div { class: "tui-panel-body hero-body",
                    p { class: "hero-tagline", "Desktop manager for Waybar modules" }
                    p { class: "hero-desc",
                        "Browse, install, and manage community modules for your Waybar status bar."
                        br {}
                        "Native desktop app with one-click installs."
                    }
                    div { class: "stats-grid",
                        div { class: "stat-box",
                            div { class: "stat-icon",
                                svg {
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z" }
                                    path { d: "M12 22V12" }
                                    polyline { points: "3.29 7 12 12 20.71 7" }
                                    path { d: "m7.5 4.27 9 5.15" }
                                }
                            }
                            div { class: "stat-info",
                                span { class: "stat-value", "{stats.total_modules}" }
                                span { class: "stat-label", "modules" }
                            }
                        }
                        div { class: "stat-box",
                            div { class: "stat-icon",
                                svg {
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                                    path { d: "M16 3.128a4 4 0 0 1 0 7.744" }
                                    path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                                    circle { cx: "9", cy: "7", r: "4" }
                                }
                            }
                            div { class: "stat-info",
                                span { class: "stat-value", "{stats.total_authors}" }
                                span { class: "stat-label", "authors" }
                            }
                        }
                        div { class: "stat-box",
                            div { class: "stat-icon",
                                svg {
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M12 15V3" }
                                    path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                    path { d: "m7 10 5 5 5-5" }
                                }
                            }
                            div { class: "stat-info",
                                span { class: "stat-value", "{downloads_label}" }
                                span { class: "stat-label", "downloads" }
                            }
                        }
                    }
                }
            }
            InstallSnippet {}
            section { class: "tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ FEATURES ]" }
                }
                div { class: "tui-panel-body",
                    div { class: "features-grid",
                        div { class: "feature-box",
                            div { class: "feature-header",
                                svg {
                                    class: "feature-icon",
                                    width: "14",
                                    height: "14",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M12.586 12.586 19 19" }
                                    path {
                                        d: "M3.688 3.037a.497.497 0 0 0-.651.651l6.5 15.999a.501.501 0 0 0 .947-.062l1.569-6.083a2 2 0 0 1 1.448-1.479l6.124-1.579a.5.5 0 0 0 .063-.947z"
                                    }
                                }
                                span { "One-Click Install" }
                            }
                            div { class: "feature-body", "No config editing required" }
                        }
                        div { class: "feature-box",
                            div { class: "feature-header",
                                svg {
                                    class: "feature-icon",
                                    width: "14",
                                    height: "14",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path {
                                        d: "M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"
                                    }
                                    path { d: "M12 22V12" }
                                    polyline { points: "3.29 7 12 12 20.71 7" }
                                    path { d: "m7.5 4.27 9 5.15" }
                                }
                                span { "Module Registry" }
                            }
                            div { class: "feature-body", "Community-driven collection" }
                        }
                        div { class: "feature-box",
                            div { class: "feature-header",
                                svg {
                                    class: "feature-icon",
                                    width: "14",
                                    height: "14",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path {
                                        d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"
                                    }
                                    circle { cx: "12", cy: "12", r: "3" }
                                }
                                span { "Auto Configuration" }
                            }
                            div { class: "feature-body", "Generated preferences UI" }
                        }
                    }
                }
            }
            section { class: "tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ DESKTOP APP ]" }
                    a { class: "tui-panel-link", href: "https://github.com/jtaw5649/barforge-app", target: "_blank", rel: "noopener",
                        "GitHub"
                        svg {
                            class: "panel-link-icon",
                            width: "12",
                            height: "12",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M5 12h14" }
                            path { d: "m12 5 7 7-7 7" }
                        }
                    }
                }
                div { class: "tui-panel-body",
                    div { class: "app-features",
                        div { class: "app-feature",
                            svg {
                                class: "app-feature-icon",
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M12 15V3" }
                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                path { d: "m7 10 5 5 5-5" }
                            }
                            div { class: "app-feature-text",
                                span { class: "app-feature-title", "Browse & Install" }
                                span { class: "app-feature-desc", "Discover and install modules from the registry" }
                            }
                        }
                        div { class: "app-feature",
                            svg {
                                class: "app-feature-icon",
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                                path { d: "M21 3v5h-5" }
                                path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                                path { d: "M8 16H3v5" }
                            }
                            div { class: "app-feature-text",
                                span { class: "app-feature-title", "Update Manager" }
                                span { class: "app-feature-desc", "Keep your modules up to date" }
                            }
                        }
                        div { class: "app-feature",
                            svg {
                                class: "app-feature-icon",
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path {
                                    d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"
                                }
                                circle { cx: "12", cy: "12", r: "3" }
                            }
                            div { class: "app-feature-text",
                                span { class: "app-feature-title", "Module Preferences" }
                                span { class: "app-feature-desc", "Auto-generated settings for each module" }
                            }
                        }
                        div { class: "app-feature",
                            svg {
                                class: "app-feature-icon",
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path {
                                    d: "M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"
                                }
                            }
                            div { class: "app-feature-text",
                                span { class: "app-feature-title", "Enable / Disable" }
                                span { class: "app-feature-desc", "Toggle modules without uninstalling" }
                            }
                        }
                    }
                }
            }
            section { class: "tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ NAVIGATION ]" }
                }
                div { class: "tui-panel-body nav-body",
                    a { class: "nav-card", href: "/modules",
                        div { class: "nav-card-icon",
                            svg {
                                class: "nav-card-icon-svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path {
                                    d: "M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"
                                }
                                path { d: "M12 22V12" }
                                polyline { points: "3.29 7 12 12 20.71 7" }
                                path { d: "m7.5 4.27 9 5.15" }
                            }
                        }
                        div { class: "nav-card-content",
                            span { class: "nav-card-title", "Browse Modules" }
                            span { class: "nav-card-desc", "{stats.total_modules} community modules across all categories" }
                        }
                        div { class: "nav-card-arrow",
                            svg {
                                class: "nav-card-arrow-icon",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "m9 18 6-6-6-6" }
                            }
                        }
                    }
                    a { class: "nav-card", href: "/upload",
                        div { class: "nav-card-icon",
                            svg {
                                class: "nav-card-icon-svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "currentColor",
                                "aria-hidden": "true",
                                path {
                                    d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                                }
                            }
                        }
                        div { class: "nav-card-content",
                            span { class: "nav-card-title", "Publish a Module" }
                            span { class: "nav-card-desc", "Share your work with the community" }
                        }
                        div { class: "nav-card-arrow",
                            svg {
                                class: "nav-card-arrow-icon",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "m9 18 6-6-6-6" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InstallSnippet() -> Element {
    static ICON_GNUBASH: Asset = asset!("/assets/icons/icon-gnubash.svg");
    static ICON_ARCH: Asset = asset!("/assets/icons/icon-archlinux.svg");
    static ICON_RUST: Asset = asset!("/assets/icons/icon-rust.svg");
    static ICON_COPY: Asset = asset!("/assets/icons/icon-copy.svg");
    static ICON_CHECK: Asset = asset!("/assets/icons/icon-check.svg");

    #[derive(Clone)]
    struct InstallMethod {
        id: &'static str,
        label: &'static str,
        icon: Asset,
        color: &'static str,
        command: &'static str,
    }

    let methods = std::rc::Rc::new(vec![
        InstallMethod {
            id: "shell",
            label: "Shell",
            icon: ICON_GNUBASH,
            color: "#4EAA25",
            command: "curl -sSL https://barforge.dev/install | sh",
        },
        InstallMethod {
            id: "aur",
            label: "AUR",
            icon: ICON_ARCH,
            color: "#1793D1",
            command: "yay -S barforge",
        },
        InstallMethod {
            id: "source",
            label: "Source",
            icon: ICON_RUST,
            color: "#DEA584",
            command: "git clone https://github.com/jtaw5649/barforge-app && cd barforge-app && cargo build --release",
        },
    ]);
    let tab_count = methods.len();
    let selected_index = use_signal(|| 1usize);
    let copied = use_signal(|| false);
    let expanded = use_signal(|| false);
    let hovered = use_signal(|| false);
    let is_mouse_in_section = use_signal(|| false);
    let copy_timeout = use_signal(|| None::<i32>);
    let tab_refs = use_signal(|| vec![None::<std::rc::Rc<MountedData>>; tab_count]);
    let active_index = *selected_index.read();
    let show_full = *expanded.read() || *hovered.read();
    let copy_icon = if *copied.read() {
        ICON_CHECK
    } else {
        ICON_COPY
    };
    let copy_label = if *copied.read() {
        "Copied"
    } else {
        "Copy to clipboard"
    };
    let copy_text = if *copied.read() { "Copied!" } else { "Copy" };
    let copy_class = if *copied.read() {
        "copy-btn copied"
    } else {
        "copy-btn"
    };
    let copy_disabled = *copied.read();

    let handle_expand = {
        let mut expanded = expanded;
        move |_| {
            let next = !*expanded.read();
            expanded.set(next);
        }
    };
    let handle_expand_keydown = {
        let mut expanded = expanded;
        move |event: KeyboardEvent| {
            if event.key() == Key::Enter {
                event.prevent_default();
                let next = !*expanded.read();
                expanded.set(next);
            }
        }
    };
    let handle_copy = {
        let methods = methods.clone();
        EventHandler::new(move |event: dioxus::prelude::MouseEvent| {
            event.stop_propagation();
            if *copied.read() {
                return;
            }
            let index = *selected_index.read();
            let command = methods
                .get(index)
                .map(|method| method.command)
                .unwrap_or(methods[0].command)
                .to_string();
            copy_to_clipboard(command, copied, copy_timeout);
        })
    };

    #[cfg(target_arch = "wasm32")]
    use_effect({
        let methods = methods.clone();
        let selected_index = selected_index;
        let copied = copied;
        let copy_timeout = copy_timeout;
        let is_mouse_in_section = is_mouse_in_section;
        move || {
            let Some(window) = web_sys::window() else {
                return;
            };
            let methods_inner = methods.clone();
            let window_inner = window.clone();
            let handler =
                Closure::<dyn FnMut(WebKeyboardEvent)>::new(move |event: WebKeyboardEvent| {
                    if !*is_mouse_in_section.read() {
                        return;
                    }
                    if *copied.read() {
                        return;
                    }
                    if !event.ctrl_key() {
                        return;
                    }
                    let key = event.key();
                    if key != "c" && key != "C" {
                        return;
                    }
                    if let Some(target) = event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    {
                        let tag = target.tag_name();
                        if tag == "INPUT" || tag == "TEXTAREA" {
                            return;
                        }
                    }
                    if let Ok(Some(selection)) = window_inner.get_selection() {
                        let selection_text: String = selection.to_string().into();
                        if !selection_text.is_empty() {
                            return;
                        }
                    }
                    event.prevent_default();
                    let index = *selected_index.read();
                    let command = methods_inner
                        .get(index)
                        .map(|method| method.command)
                        .unwrap_or(methods_inner[0].command)
                        .to_string();
                    copy_to_clipboard(command, copied, copy_timeout);
                });
            let _ = window
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
            handler.forget();
        }
    });

    rsx! {
        section {
            class: "tui-panel install-snippet",
            "aria-label": "Install methods",
            onmouseenter: {
                let mut is_mouse_in_section = is_mouse_in_section;
                move |_| {
                    is_mouse_in_section.set(true);
                }
            },
            onmouseleave: {
                let mut is_mouse_in_section = is_mouse_in_section;
                move |_| {
                    is_mouse_in_section.set(false);
                }
            },
            div { class: "tui-panel-header",
                span { class: "tui-panel-title", "[ INSTALL ]" }
            }
            div { class: "install-tabs", role: "tablist", "aria-label": "Install methods",
                {methods.iter().enumerate().map(|(index, method)| {
                    let key = format!("install-{}", method.id);
                    let is_active = index == active_index;
                    let class_name = if is_active { "install-tab active" } else { "install-tab" };
                    let aria_selected = if is_active { "true" } else { "false" };
                    let tab_index = if is_active { "0" } else { "-1" };
                    let tab_id = format!("install-tab-{index}");
                    let panel_id = format!("install-panel-{index}");
                    let tab_style = format!("--tab-color: {};", method.color);
                    rsx! {
                        button {
                            class: "{class_name}",
                            id: "{tab_id}",
                            key: "{key}",
                            role: "tab",
                            tabindex: "{tab_index}",
                            "aria-selected": "{aria_selected}",
                            "aria-controls": "{panel_id}",
                            r#type: "button",
                            style: "{tab_style}",
                            onmounted: {
                                let mut tab_refs = tab_refs;
                                move |event| {
                                    let mut refs = tab_refs.write();
                                    if let Some(slot) = refs.get_mut(index) {
                                        *slot = Some(event.data());
                                    }
                                }
                            },
                            onclick: {
                                let mut selected_index = selected_index;
                                move |_| {
                                    selected_index.set(index);
                                }
                            },
                            onkeydown: {
                                let mut selected_index = selected_index;
                                move |event: KeyboardEvent| {
                                    let current = *selected_index.read();
                                    if let Some(next_index) =
                                        next_tab_index(current, tab_count, &event.key())
                                    {
                                        event.prevent_default();
                                        selected_index.set(next_index);
                                        let element = tab_refs
                                            .read()
                                            .get(next_index)
                                            .and_then(|tab| tab.clone());
                                        if let Some(element) = element {
                                            spawn(async move {
                                                let _ = element.set_focus(true).await;
                                            });
                                        }
                                    }
                                }
                            },
                            img { src: method.icon, alt: "", width: "16", height: "16" }
                            span { "{method.label}" }
                        }
                    }
                })}
            }
            div { class: "tui-panel-body",
                {methods.iter().enumerate().map(|(index, method)| {
                    let panel_id = format!("install-panel-{index}");
                    let tab_id = format!("install-tab-{index}");
                    let is_active = index == active_index;
                    let terminal_style = format!("--terminal-accent: {};", method.color);
                    let cmd_class = if is_active && show_full {
                        "terminal-cmd show-full"
                    } else {
                        "terminal-cmd"
                    };
                    rsx! {
                        div {
                            class: "install-panel",
                            id: "{panel_id}",
                            role: "tabpanel",
                            "aria-labelledby": "{tab_id}",
                            hidden: !is_active,
                            div { class: "terminal-window", style: "{terminal_style}",
                                div { class: "terminal-header",
                                    span { class: "terminal-dot" }
                                    span { class: "terminal-dot" }
                                    span { class: "terminal-dot" }
                                    span { class: "terminal-title", "Install barforge" }
                                }
                                div {
                                    class: "terminal-body",
                                    role: "button",
                                    tabindex: "0",
                                    "aria-label": "Toggle command expansion",
                                    onclick: handle_expand,
                                    onkeydown: handle_expand_keydown,
                                    onmouseenter: {
                                        let mut hovered = hovered;
                                        move |_| hovered.set(true)
                                    },
                                    onmouseleave: {
                                        let mut hovered = hovered;
                                        move |_| hovered.set(false)
                                    },
                                    span { class: "terminal-prompt", "$" }
                                    code { class: "{cmd_class}", "{method.command}" }
                                    div { class: "terminal-actions",
                                        div { class: "kbd-group",
                                            kbd { class: "kbd", "Ctrl" }
                                            kbd { class: "kbd", "C" }
                                        }
                                        button {
                                            class: "{copy_class}",
                                            r#type: "button",
                                            "aria-label": "{copy_label}",
                                            disabled: copy_disabled,
                                            onclick: handle_copy,
                                            img { src: copy_icon, alt: "", width: "14", height: "14" }
                                            span { "{copy_text}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
const COPY_RESET_MS: i32 = 1200;

#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(command: String, mut copied: Signal<bool>, copy_timeout: Signal<Option<i32>>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let clipboard = window.navigator().clipboard();
    let promise = clipboard.write_text(&command);
    wasm_bindgen_futures::spawn_local(async move {
        if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
            copied.set(true);
            schedule_copy_reset(copied, copy_timeout);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_to_clipboard(_command: String, _copied: Signal<bool>, _copy_timeout: Signal<Option<i32>>) {}

#[cfg(target_arch = "wasm32")]
fn schedule_copy_reset(mut copied: Signal<bool>, mut copy_timeout: Signal<Option<i32>>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(handle) = *copy_timeout.read() {
        window.clear_timeout_with_handle(handle);
    }
    let callback = Closure::once_into_js(move || {
        copied.set(false);
    });
    if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.unchecked_ref(),
        COPY_RESET_MS,
    ) {
        copy_timeout.set(Some(handle));
    }
}

#[component]
pub(crate) fn ModulesLayout() -> Element {
    let navigator = use_navigator();
    let route: Route = use_route();
    let current = route.to_string();
    let is_search = current.starts_with("/modules/search");
    let active_index = if is_search { 1 } else { 0 };
    let tab_count = 2usize;
    let tab_refs = use_signal(|| vec![None::<std::rc::Rc<MountedData>>; tab_count]);
    let discover_class = if is_search {
        "browse-tab"
    } else {
        "browse-tab active"
    };
    let search_class = if is_search {
        "browse-tab active"
    } else {
        "browse-tab"
    };
    let discover_selected = if is_search { "false" } else { "true" };
    let search_selected = if is_search { "true" } else { "false" };
    let discover_tabindex = if is_search { "-1" } else { "0" };
    let search_tabindex = if is_search { "0" } else { "-1" };

    rsx! {
        section { class: "modules-layout",
            div { class: "browse-tab-bar",
                nav { class: "browse-tabs", role: "tablist", "aria-label": "Browse navigation",
                    a {
                        id: "modules-tab-discover",
                        class: "{discover_class}",
                        href: "/modules",
                        role: "tab",
                        tabindex: "{discover_tabindex}",
                        "aria-selected": "{discover_selected}",
                        "aria-controls": "modules-discover-panel",
                        onmounted: {
                            let mut tab_refs = tab_refs;
                            move |event| {
                                let mut refs = tab_refs.write();
                                if let Some(slot) = refs.get_mut(0) {
                                    *slot = Some(event.data());
                                }
                            }
                        },
                        onkeydown: {
                            move |event: KeyboardEvent| {
                                if let Some(next_index) =
                                    next_tab_index(active_index, tab_count, &event.key())
                                {
                                    event.prevent_default();
                                    let route = if next_index == 0 {
                                        Route::ModulesIndex {}
                                    } else {
                                        Route::ModulesSearch {
                                            q: None,
                                            category: None,
                                            sort: None,
                                            page: None,
                                            view: None,
                                        }
                                    };
                                    navigator.replace(route);
                                    let element = tab_refs
                                        .read()
                                        .get(next_index)
                                        .and_then(|tab| tab.clone());
                                    if let Some(element) = element {
                                        spawn(async move {
                                            let _ = element.set_focus(true).await;
                                        });
                                    }
                                }
                            }
                        },
                        "Discover"
                    }
                    a {
                        id: "modules-tab-search",
                        class: "{search_class}",
                        href: "/modules/search",
                        role: "tab",
                        tabindex: "{search_tabindex}",
                        "aria-selected": "{search_selected}",
                        "aria-controls": "modules-search-panel",
                        onmounted: {
                            let mut tab_refs = tab_refs;
                            move |event| {
                                let mut refs = tab_refs.write();
                                if let Some(slot) = refs.get_mut(1) {
                                    *slot = Some(event.data());
                                }
                            }
                        },
                        onkeydown: {
                            move |event: KeyboardEvent| {
                                if let Some(next_index) =
                                    next_tab_index(active_index, tab_count, &event.key())
                                {
                                    event.prevent_default();
                                    let route = if next_index == 0 {
                                        Route::ModulesIndex {}
                                    } else {
                                        Route::ModulesSearch {
                                            q: None,
                                            category: None,
                                            sort: None,
                                            page: None,
                                            view: None,
                                        }
                                    };
                                    navigator.replace(route);
                                    let element = tab_refs
                                        .read()
                                        .get(next_index)
                                        .and_then(|tab| tab.clone());
                                    if let Some(element) = element {
                                        spawn(async move {
                                            let _ = element.set_focus(true).await;
                                        });
                                    }
                                }
                            }
                        },
                        "Search"
                    }
                }
            }
            {modules_tab_panels(is_search, rsx!(Outlet::<Route> {}))}
        }
    }
}

fn modules_tab_panels(is_search: bool, content: Element) -> Element {
    if is_search {
        rsx! {
            div {
                class: "modules-panel browse-page",
                role: "tabpanel",
                id: "modules-discover-panel",
                "aria-labelledby": "modules-tab-discover",
                hidden: true,
            }
            div {
                class: "modules-panel browse-page",
                role: "tabpanel",
                id: "modules-search-panel",
                "aria-labelledby": "modules-tab-search",
                hidden: false,
                {content}
            }
        }
    } else {
        rsx! {
            div {
                class: "modules-panel browse-page",
                role: "tabpanel",
                id: "modules-discover-panel",
                "aria-labelledby": "modules-tab-discover",
                hidden: false,
                {content}
            }
            div {
                class: "modules-panel browse-page",
                role: "tabpanel",
                id: "modules-search-panel",
                "aria-labelledby": "modules-tab-search",
                hidden: true,
            }
        }
    }
}

pub(crate) fn status_banner_with_retry<T>(
    state: &RemoteState<T>,
    loading_label: &str,
    error_title: &str,
    error_detail: &str,
    retry_href: Option<&str>,
) -> Element {
    match state {
        RemoteState::Loading => rsx! {
            div {
                class: "index-status loading",
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                "{loading_label}"
            }
        },
        RemoteState::Error(_message) => rsx! {
            div {
                class: "index-status error",
                role: "alert",
                "aria-live": "assertive",
                "aria-atomic": "true",
                span { class: "index-status-title", "{error_title}" }
                span { class: "index-status-detail", "{error_detail}" }
                if let Some(retry_href) = retry_href {
                    div { class: "index-status-actions",
                        a { class: "index-status-action", href: "{retry_href}", "Retry" }
                    }
                }
            }
        },
        RemoteState::Ready(_) | RemoteState::Unavailable => VNode::empty(),
    }
}

fn landing_status_banner(state: &RemoteState<LandingData>, retry_href: &str) -> Element {
    status_banner_with_retry(
        state,
        "Loading landing data...",
        "Landing unavailable",
        "Please try again.",
        Some(retry_href),
    )
}

#[component]
pub(crate) fn ModulesIndex() -> Element {
    let index_state = use_registry_index_state();
    let fallback_modules = modules_from_state(&index_state);
    let featured_state = use_featured_state();
    let featured_payload = featured_modules_from_state(&featured_state, &fallback_modules);
    let recently_viewed = use_recently_viewed();
    let recent_items = recently_viewed.items();
    let recommended = recommended_modules(
        &recent_items,
        &featured_payload.popular,
        &featured_payload.recent,
    );
    let now = app_now();
    let retry_href = Route::ModulesIndex {}.to_string();
    let show_modules = matches!(
        index_state,
        RemoteState::Ready(_) | RemoteState::Unavailable
    ) || matches!(
        featured_state,
        RemoteState::Ready(_) | RemoteState::Unavailable
    );

    rsx! {
        div { class: "browse-discover",
            {status_banner_with_retry(
                &index_state,
                "Loading module index...",
                "Index unavailable",
                "Please try again.",
                Some(&retry_href),
            )}
            {status_banner_with_retry(
                &featured_state,
                "Loading featured modules...",
                "Featured modules unavailable",
                "Please try again.",
                Some(&retry_href),
            )}
            section { class: "categories-section tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ CATEGORIES ]" }
                    a { class: "tui-panel-link", href: "/modules/search", "See all →" }
                }
                div { class: "tui-panel-body",
                    div { class: "section-header",
                        div {
                            h2 { "Browse by Category" }
                            p { "Jump into the essentials without the noise" }
                        }
                    }
                    CategoryPills {
                        categories: browse_categories(),
                        base_url: Some("/modules/search".to_string()),
                    }
                }
            }
            if !recommended.is_empty() {
                section { class: "home-section tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ RECOMMENDED ]" }
                        a { class: "tui-panel-link", href: "/modules/search", "See all →" }
                    }
                    div { class: "tui-panel-body",
                        div { class: "section-header",
                            div {
                                h2 { "Recommended for You" }
                                p { "Based on modules you've viewed" }
                            }
                        }
                        StarredModuleList {
                            modules: recommended.clone(),
                            query: None,
                            sort: None,
                            category: None,
                            page: None,
                            per_page: Some(6),
                            view_mode: None,
                            now,
                        }
                    }
                }
            }
            section { class: "home-section tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ FEATURED MODULES ]" }
                }
                div { class: "tui-panel-body",
                    div { class: "section-header",
                        div {
                            h2 { "Featured Modules" }
                            p { "Hand-picked by our team" }
                        }
                    }
                    if show_modules {
                        StarredModuleList {
                            modules: featured_payload.featured.clone(),
                            query: None,
                            sort: Some(ModuleSort::Popular),
                            category: None,
                            page: None,
                            per_page: Some(6),
                            view_mode: None,
                            now,
                        }
                    }
                }
            }
            section { class: "home-section tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ POPULAR MODULES ]" }
                    a { class: "tui-panel-link", href: "/modules/search?sort=popular", "See all →" }
                }
                div { class: "tui-panel-body",
                    div { class: "section-header",
                        div {
                            h2 { "Popular Modules" }
                            p { "Top downloads and community favorites" }
                        }
                    }
                    if show_modules {
                        StarredModuleList {
                            modules: featured_payload.popular.clone(),
                            query: None,
                            sort: Some(ModuleSort::Popular),
                            category: None,
                            page: None,
                            per_page: Some(6),
                            view_mode: None,
                            now,
                        }
                    }
                }
            }
            section { class: "home-section tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ RECENTLY ADDED ]" }
                    a { class: "tui-panel-link", href: "/modules/search?sort=recent", "See all →" }
                }
                div { class: "tui-panel-body",
                    div { class: "section-header",
                        div {
                            h2 { "Recently Added" }
                            p { "Fresh modules, straight from the registry" }
                        }
                    }
                    if show_modules {
                        StarredModuleList {
                            modules: featured_payload.recent.clone(),
                            query: None,
                            sort: Some(ModuleSort::Recent),
                            category: None,
                            page: None,
                            per_page: Some(6),
                            view_mode: None,
                            now,
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn ModulesSearch(
    q: Option<String>,
    category: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    view: Option<String>,
) -> Element {
    let index_state = use_registry_index_state();
    let navigator = use_navigator();
    let modules = modules_from_state(&index_state);
    let recently_viewed = use_recently_viewed();
    let recent_items = recently_viewed.items();
    let recent_modules = recent_items
        .iter()
        .map(RecentModule::to_registry_module)
        .collect::<Vec<_>>();
    let now = app_now();
    let current_page = page.unwrap_or(1).max(1);
    let sort_choice = parse_sort(sort.clone());
    let category_filter = parse_category(category.clone());
    let view_mode_value = parse_view(view.clone());
    let view_param = if view_mode_value == ModuleViewMode::List {
        Some("list".to_string())
    } else {
        None
    };
    let view_value = view_param.as_deref();
    let page_param = if current_page > 1 {
        Some(current_page)
    } else {
        None
    };
    let retry_href = Route::ModulesSearch {
        q: q.clone(),
        category: category.clone(),
        sort: sort.clone(),
        page,
        view: view_param.clone(),
    }
    .to_string();
    let show_modules = matches!(
        index_state,
        RemoteState::Ready(_) | RemoteState::Unavailable
    );
    let base_url = "/modules/search".to_string();
    let search_action = build_search_url(&base_url, None, None, None, view_value, None);
    let current_query = q.clone().unwrap_or_default();
    let current_category = category.clone().unwrap_or_default();
    let current_sort = sort.clone().unwrap_or_else(|| DEFAULT_SORT.to_string());
    let query_value = current_query.trim().to_lowercase();
    let filtered_count = modules
        .iter()
        .filter(|module| {
            let matches_query = if query_value.is_empty() {
                true
            } else {
                module.name.to_lowercase().contains(&query_value)
                    || module.description.to_lowercase().contains(&query_value)
                    || module.author.to_lowercase().contains(&query_value)
            };
            let matches_category = category_filter
                .as_ref()
                .map(|category| module.category == *category)
                .unwrap_or(true);
            matches_query && matches_category
        })
        .count();
    let per_page = 12usize;
    let total_pages = filtered_count.div_ceil(per_page);
    let active_filter_count = (if current_query.is_empty() { 0 } else { 1 })
        + (if current_category.is_empty() { 0 } else { 1 })
        + (if current_sort == DEFAULT_SORT { 0 } else { 1 });
    let has_active_filters = active_filter_count > 0;
    let show_recently_viewed = !recent_modules.is_empty() && !has_active_filters;
    let sidebar_collapsed = use_signal(|| false);
    let sidebar_is_collapsed = *sidebar_collapsed.read();
    let mobile_filters_open = use_signal(|| false);
    let mobile_filters_is_open = *mobile_filters_open.read();
    let modules_search_class = if sidebar_is_collapsed {
        "modules-search sidebar-collapsed"
    } else {
        "modules-search"
    };
    let mobile_filter_class = match (mobile_filters_is_open, sidebar_is_collapsed) {
        (true, true) => "tui-panel modules-search-filters filter-sidebar open collapsed",
        (true, false) => "tui-panel modules-search-filters filter-sidebar open",
        (false, true) => "tui-panel modules-search-filters filter-sidebar collapsed",
        (false, false) => "tui-panel modules-search-filters filter-sidebar",
    };
    let mobile_filter_expanded = if mobile_filters_is_open {
        "true"
    } else {
        "false"
    };
    let collapsed_badge_title = if active_filter_count == 1 {
        "Expand to see 1 active filter".to_string()
    } else {
        format!("Expand to see {active_filter_count} active filters")
    };
    let grid_class = if view_mode_value == ModuleViewMode::Grid {
        "view-btn active"
    } else {
        "view-btn"
    };
    let list_class = if view_mode_value == ModuleViewMode::List {
        "view-btn active"
    } else {
        "view-btn"
    };
    let grid_pressed = if view_mode_value == ModuleViewMode::Grid {
        "true"
    } else {
        "false"
    };
    let list_pressed = if view_mode_value == ModuleViewMode::List {
        "true"
    } else {
        "false"
    };
    let recent_container_class = if view_mode_value == ModuleViewMode::Grid {
        "recently-viewed-container grid"
    } else {
        "recently-viewed-container list"
    };
    let sidebar_label = if sidebar_is_collapsed {
        "Expand sidebar"
    } else {
        "Collapse sidebar"
    };
    let sidebar_expanded = if sidebar_is_collapsed {
        "false"
    } else {
        "true"
    };
    let sidebar_icon_class = if sidebar_is_collapsed { "rotated" } else { "" };
    let query_param = if current_query.is_empty() {
        None
    } else {
        Some(current_query.clone())
    };
    let category_param = if current_category.is_empty() {
        None
    } else {
        Some(current_category.clone())
    };
    let sort_param = if current_sort == DEFAULT_SORT {
        None
    } else {
        Some(current_sort.clone())
    };
    let grid_href = build_search_url(
        &base_url,
        query_param.as_deref(),
        category_param.as_deref(),
        sort_param.as_deref(),
        None,
        page_param,
    );
    let list_href = build_search_url(
        &base_url,
        query_param.as_deref(),
        category_param.as_deref(),
        sort_param.as_deref(),
        Some("list"),
        page_param,
    );
    let on_category = {
        let query_param = query_param.clone();
        let sort_param = sort_param.clone();
        let view_param = view_param.clone();
        let mut mobile_filters_open = mobile_filters_open;
        EventHandler::new(move |slug: String| {
            let category_param = if slug.is_empty() { None } else { Some(slug) };
            let route = Route::ModulesSearch {
                q: query_param.clone(),
                category: category_param,
                sort: sort_param.clone(),
                page: None,
                view: view_param.clone(),
            };
            navigator.replace(route);
            if *mobile_filters_open.read() {
                mobile_filters_open.set(false);
            }
        })
    };
    let on_sort = {
        let query_param = query_param.clone();
        let category_param = category_param.clone();
        let view_param = view_param.clone();
        EventHandler::new(move |value: String| {
            let sort_param = if value == DEFAULT_SORT {
                None
            } else {
                Some(value)
            };
            let route = Route::ModulesSearch {
                q: query_param.clone(),
                category: category_param.clone(),
                sort: sort_param,
                page: None,
                view: view_param.clone(),
            };
            navigator.replace(route);
        })
    };

    rsx! {
        section { class: "tui-panel modules-search-header",
            div { class: "tui-panel-header",
                span { class: "tui-panel-title", "[ SEARCH ]" }
                span { class: "tui-panel-status", "{modules.len()} modules" }
            }
            div { class: "tui-panel-body modules-search-header-body",
                div { class: "modules-search-text",
                    h1 { "Browse Modules" }
                    p { "Discover {modules.len()} community-created modules for Waybar" }
                }
                div { class: "modules-search-controls",
                    form { class: "filter-input-wrapper", method: "get", action: "{search_action}",
                        svg {
                            class: "filter-input-icon",
                            width: "18",
                            height: "18",
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
                            class: "filter-input",
                            r#type: "text",
                            name: "q",
                            value: "{current_query}",
                            placeholder: "Filter modules...",
                            autocomplete: "off"
                        }
                        if !current_category.is_empty() {
                            input { r#type: "hidden", name: "category", value: "{current_category}" }
                        }
                        if current_sort != DEFAULT_SORT {
                            input { r#type: "hidden", name: "sort", value: "{current_sort}" }
                        }
                    }
                }
            }
        }
        div { class: "{modules_search_class}",
            aside { id: "filter-sidebar", class: "{mobile_filter_class}",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ FILTERS ]" }
                }
                div { class: "tui-panel-body",
                    if has_active_filters && !sidebar_is_collapsed {
                        a { class: "clear-filters-btn", href: "{base_url}",
                            svg {
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                line { x1: "18", y1: "6", x2: "6", y2: "18" }
                                line { x1: "6", y1: "6", x2: "18", y2: "18" }
                            }
                            "Clear all filters"
                            span { class: "filter-count", "{active_filter_count}" }
                        }
                    }
                    if has_active_filters && sidebar_is_collapsed {
                        button {
                            class: "collapsed-filter-badge",
                            r#type: "button",
                            title: "{collapsed_badge_title}",
                            onclick: {
                                let mut sidebar_collapsed = sidebar_collapsed;
                                move |_| sidebar_collapsed.set(false)
                            },
                            svg {
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
                            }
                            span { class: "badge-count", "{active_filter_count}" }
                        }
                    }
                    SearchToolbar {
                        categories: browse_categories(),
                        sort_options: default_sort_options(),
                        current_category: category.clone(),
                        current_sort: sort.clone(),
                        on_category: Some(on_category),
                        on_sort: Some(on_sort),
                    }
                }
            }
            section { class: "tui-panel modules-search-results",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ RESULTS ]" }
                    div { class: "modules-search-actions",
                        button {
                            class: "mobile-filter-toggle",
                            r#type: "button",
                            aria_expanded: "{mobile_filter_expanded}",
                            onclick: {
                                let mut mobile_filters_open = mobile_filters_open;
                                move |_| {
                                    let next = !*mobile_filters_open.read();
                                    mobile_filters_open.set(next);
                                }
                            },
                            svg {
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
                            }
                            "Filters"
                        }
                        div { class: "view-toggle", role: "group", aria_label: "View mode",
                            a {
                                class: "{grid_class}",
                                href: "{grid_href}",
                                role: "button",
                                "aria-pressed": "{grid_pressed}",
                                aria_label: "Grid view",
                                title: "Grid view",
                                svg {
                                    width: "18",
                                    height: "18",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                                    rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                                    rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                                    rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
                                }
                            }
                            a {
                                class: "{list_class}",
                                href: "{list_href}",
                                role: "button",
                                "aria-pressed": "{list_pressed}",
                                aria_label: "List view",
                                title: "List view",
                                svg {
                                    width: "18",
                                    height: "18",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    line { x1: "8", y1: "6", x2: "21", y2: "6" }
                                    line { x1: "8", y1: "12", x2: "21", y2: "12" }
                                    line { x1: "8", y1: "18", x2: "21", y2: "18" }
                                    circle { cx: "4", cy: "6", r: "1", fill: "currentColor" }
                                    circle { cx: "4", cy: "12", r: "1", fill: "currentColor" }
                                    circle { cx: "4", cy: "18", r: "1", fill: "currentColor" }
                                }
                            }
                        }
                        button {
                            class: "sidebar-toggle",
                            r#type: "button",
                            aria_expanded: "{sidebar_expanded}",
                            aria_controls: "filter-sidebar",
                            aria_label: "{sidebar_label}",
                            title: "{sidebar_label}",
                            onclick: {
                                let mut sidebar_collapsed = sidebar_collapsed;
                                move |_| {
                                    let next = !*sidebar_collapsed.read();
                                    sidebar_collapsed.set(next);
                                }
                            },
                            svg {
                                width: "18",
                                height: "18",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                "aria-hidden": "true",
                                class: "{sidebar_icon_class}",
                                rect { x: "3", y: "3", width: "18", height: "18", rx: "2", ry: "2" }
                                line { x1: "9", y1: "3", x2: "9", y2: "21" }
                                polyline { points: "14 9 17 12 14 15" }
                            }
                        }
                    }
                }
                div { class: "tui-panel-body",
                    div { class: "results-container",
                        {status_banner_with_retry(
                            &index_state,
                            "Loading module index...",
                            "Index unavailable",
                            "Please try again.",
                            Some(&retry_href),
                        )}
                        if show_recently_viewed {
                            section { class: "recently-viewed",
                                div { class: "section-header",
                                    h2 {
                                        svg {
                                            width: "18",
                                            height: "18",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            circle { cx: "12", cy: "12", r: "10" }
                                            polyline { points: "12 6 12 12 16 14" }
                                        }
                                        "Recently Viewed"
                                    }
                                    button {
                                        class: "clear-history-btn",
                                        r#type: "button",
                                        onclick: {
                                            let recently_viewed = recently_viewed.clone();
                                            move |_| recently_viewed.clear()
                                        },
                                        "Clear history"
                                    }
                                }
                                div { class: "{recent_container_class}",
                                    StarredModuleList {
                                        modules: recent_modules.clone(),
                                        query: None,
                                        sort: None,
                                        category: None,
                                        page: None,
                                        per_page: Some(6),
                                        view_mode: Some(view_mode_value),
                                        now,
                                    }
                                }
                            }
                        }
                        section { class: "results",
                            if show_modules {
                                StarredModuleList {
                                    modules,
                                    query: q.clone(),
                                    sort: Some(sort_choice),
                                    category: category_filter,
                                    page: Some(current_page),
                                    per_page: Some(per_page),
                                    view_mode: Some(view_mode_value),
                                    now,
                                }
                                PaginationControls {
                                    current_page,
                                    total_pages,
                                    current_query: q,
                                    current_category: category,
                                    current_sort: sort,
                                    current_view: view_param.clone(),
                                    base_url: Some(base_url),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ModuleDetailReady {
    pub(crate) module: RegistryModule,
    pub(crate) versions: Vec<VersionHistoryEntry>,
    pub(crate) related_modules: Vec<RegistryModule>,
    pub(crate) screenshots: Vec<String>,
    pub(crate) reviews: Vec<Review>,
    pub(crate) collections: Vec<Collection>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ModuleDetailView {
    Loading,
    NotFound,
    Error { title: String, message: String },
    Ready(Box<ModuleDetailReady>),
}

pub(crate) fn resolve_module_detail_view(
    uuid: &str,
    module_state: &RemoteState<RegistryModule>,
    versions_state: &RemoteState<Vec<VersionHistoryEntry>>,
    related_state: &RemoteState<Vec<RegistryModule>>,
    screenshots_state: &RemoteState<Vec<Screenshot>>,
    reviews_state: &RemoteState<Vec<Review>>,
    collections_state: &RemoteState<Vec<Collection>>,
) -> ModuleDetailView {
    match module_state {
        RemoteState::Loading => ModuleDetailView::Loading,
        RemoteState::Error(message) => {
            if message.contains("404") {
                ModuleDetailView::NotFound
            } else {
                ModuleDetailView::Error {
                    title: "Module unavailable".to_string(),
                    message: message.clone(),
                }
            }
        }
        _ => {
            let module = match module_state {
                RemoteState::Ready(module) => module.clone(),
                RemoteState::Unavailable => sample_module_by_uuid(uuid),
                RemoteState::Loading | RemoteState::Error(_) => sample_fallback_module(),
            };
            let versions = match versions_state {
                RemoteState::Ready(versions) => versions.clone(),
                RemoteState::Unavailable => sample_versions(),
                RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
            };
            let related_modules = match related_state {
                RemoteState::Ready(modules) => modules.clone(),
                RemoteState::Unavailable => sample_related_modules(&module.uuid),
                RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
            };
            let screenshots = match screenshots_state {
                RemoteState::Ready(screenshots) => {
                    build_screenshot_urls(api::api_base_url(), &module.uuid, screenshots)
                }
                RemoteState::Unavailable => sample_screenshots(),
                RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
            };
            let reviews = match reviews_state {
                RemoteState::Ready(reviews) => reviews.clone(),
                RemoteState::Unavailable => Vec::new(),
                RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
            };
            let collections = match collections_state {
                RemoteState::Ready(collections) => collections.clone(),
                RemoteState::Unavailable => Vec::new(),
                RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
            };
            ModuleDetailView::Ready(Box::new(ModuleDetailReady {
                module,
                versions,
                related_modules,
                screenshots,
                reviews,
                collections,
            }))
        }
    }
}

#[component]
pub(crate) fn ModuleDetailRoute(uuid: ModuleSlug) -> Element {
    let uuid = uuid.as_str();
    let module_state = use_module_detail_state(uuid);
    let versions_state = use_module_versions_state(uuid);
    let related_state = use_related_modules_state(uuid);
    let screenshots_state = use_module_screenshots_state(uuid);
    let reviews_state = use_module_reviews_state(uuid);
    let auth = use_auth();
    let collections_state = use_collections_state(auth.authenticated);
    let recently_viewed = use_recently_viewed();
    let last_viewed = use_signal(|| None::<String>);
    let related_loading = matches!(related_state, RemoteState::Loading);
    let now = app_now();
    let retry_href = format!("/modules/{uuid}");

    let view = resolve_module_detail_view(
        uuid,
        &module_state,
        &versions_state,
        &related_state,
        &screenshots_state,
        &reviews_state,
        &collections_state,
    );
    let view_snapshot = view.clone();

    use_effect({
        let recently_viewed = recently_viewed.clone();
        let mut last_viewed = last_viewed;
        move || {
            let ModuleDetailView::Ready(ready) = &view_snapshot else {
                return;
            };
            let uuid = ready.module.uuid.clone();
            if last_viewed.read().as_deref() == Some(uuid.as_str()) {
                return;
            }
            last_viewed.set(Some(uuid));
            recently_viewed.add_from_module(&ready.module);
        }
    });

    let (module, versions, related_modules, screenshots, reviews) = match view {
        ModuleDetailView::Loading => {
            let status = RemoteState::<RegistryModule>::Loading;
            return status_banner_with_retry(
                &status,
                "Loading module...",
                "Module unavailable",
                "Please try again.",
                None,
            );
        }
        ModuleDetailView::NotFound => {
            FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);
            return NotFoundRoute(NotFoundRouteProps {
                segments: vec!["modules".to_string(), uuid.to_string()],
            });
        }
        ModuleDetailView::Error { title, message } => {
            let status: RemoteState<RegistryModule> = RemoteState::Error(message);
            return status_banner_with_retry(
                &status,
                "Loading module...",
                &title,
                "Please try again.",
                Some(&retry_href),
            );
        }
        ModuleDetailView::Ready(ready) => {
            let ModuleDetailReady {
                module,
                versions,
                related_modules,
                screenshots,
                reviews,
                ..
            } = *ready;
            (module, versions, related_modules, screenshots, reviews)
        }
    };

    let copied = use_signal(|| false);
    let copy_timeout = use_signal(|| None::<i32>);
    let install_command = format!("barforge install {}", module.uuid);
    let on_copy_install = {
        let command = install_command.clone();
        EventHandler::new(move |_| {
            if *copied.read() {
                return;
            }
            copy_to_clipboard(command.clone(), copied, copy_timeout);
        })
    };
    let install_copied = *copied.read();

    rsx! {
        ModuleDetail {
            module,
            screenshots,
            related_modules,
            related_loading,
            versions,
            reviews,
            now,
            install_copied,
            on_copy_install,
        }
    }
}

#[component]
pub(crate) fn SettingsLayout() -> Element {
    rsx! {
        ProtectedRoute {
            section { class: "settings-layout",
                aside { class: "settings-sidebar tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ SETTINGS ]" }
                    }
                    div { class: "tui-panel-body",
                        h1 { "Settings" }
                        nav { class: "settings-nav", "aria-label": "Settings navigation",
                            a { class: "settings-link", href: "/settings/profile",
                                svg {
                                    class: "settings-icon",
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "aria-hidden": "true",
                                    path { d: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" }
                                    circle { cx: "12", cy: "7", r: "4" }
                                }
                                span { "Profile" }
                            }
                            a { class: "settings-link", href: "/settings/notifications",
                                svg {
                                    class: "settings-icon",
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "aria-hidden": "true",
                                    path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                                    path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
                                }
                                span { "Notifications" }
                            }
                            a { class: "settings-link", href: "/settings/security",
                                svg {
                                    class: "settings-icon",
                                    width: "16",
                                    height: "16",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    "aria-hidden": "true",
                                    path {
                                        d: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"
                                    }
                                }
                                span { "Security" }
                            }
                        }
                        a { class: "settings-back", href: "/dashboard",
                            svg {
                                class: "settings-back-icon",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "m12 19-7-7 7-7" }
                                path { d: "M19 12H5" }
                            }
                            span { "Back to Dashboard" }
                        }
                    }
                }
                div { class: "settings-content",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
pub(crate) fn SettingsIndex() -> Element {
    let navigator = use_navigator();

    use_effect(move || {
        navigator.replace(Route::SettingsProfile {});
    });

    rsx! {
        section { class: "settings-panel",
            h2 { "Settings" }
            p { "Redirecting to settings profile" }
        }
    }
}

#[component]
pub(crate) fn SettingsProfile() -> Element {
    let auth = use_auth();
    let errors = use_signal(Vec::<FieldError>::new);
    let error_list = errors.read().clone();
    let spec = profile_request_spec();
    let username = match &auth.state {
        RemoteState::Ready(session) => session
            .user
            .as_ref()
            .map(|user| user.login.as_str())
            .unwrap_or("barforge"),
        _ => "barforge",
    }
    .to_string();
    let profile_state = use_user_profile_me_state(auth.authenticated);
    let modules_state = use_modules_mine_state(auth.authenticated);
    let module_count = match &modules_state {
        RemoteState::Ready(modules) => modules.len(),
        RemoteState::Unavailable => sample_modules().len(),
        RemoteState::Loading | RemoteState::Error(_) => 0,
    };
    let profile = match &profile_state {
        RemoteState::Ready(profile) => profile.clone(),
        RemoteState::Unavailable => sample_user_profile(&username, module_count),
        RemoteState::Loading | RemoteState::Error(_) => {
            sample_user_profile(&username, module_count)
        }
    };
    let display_name_value = profile.display_name.clone().unwrap_or_default();
    let bio_value = profile.bio.clone().unwrap_or_default();
    let website_value = profile.website_url.clone().unwrap_or_default();
    let github_value = profile.github_url.clone().unwrap_or_default();
    let twitter_value = profile.twitter_url.clone().unwrap_or_default();
    let bluesky_value = profile.bluesky_url.clone().unwrap_or_default();
    let discord_value = profile.discord_url.clone().unwrap_or_default();
    let sponsor_value = profile.sponsor_url.clone().unwrap_or_default();
    let on_submit = {
        let mut errors = errors;
        move |evt: FormEvent| {
            evt.prevent_default();
            let values = profile_form_values(&evt);
            let validation = validate_profile_form(&values);
            if !validation.is_empty() {
                errors.set(validation);
                return;
            }
            errors.set(Vec::new());
            #[cfg(target_arch = "wasm32")]
            {
                let payload = update_profile_request(&values);
                let errors = errors.clone();
                spawn(async move {
                    if submit_profile_update(payload, spec).await.is_err() {
                        let mut errors = errors.clone();
                        errors.set(vec![FieldError {
                            field: "form",
                            message: "Profile update failed. Please try again.",
                        }]);
                    }
                });
            }
        }
    };
    let display_name_error = field_error_message(&error_list, "display_name");
    let bio_error = field_error_message(&error_list, "bio");
    let website_error = field_error_message(&error_list, "website_url");
    let github_error = field_error_message(&error_list, "github_url");
    let twitter_error = field_error_message(&error_list, "twitter_url");
    let bluesky_error = field_error_message(&error_list, "bluesky_url");
    let discord_error = field_error_message(&error_list, "discord_url");
    let sponsor_error = field_error_message(&error_list, "sponsor_url");

    rsx! {
        section { class: "settings-panel tui-panel",
            div { class: "tui-panel-header",
                span { class: "tui-panel-title", "[ PROFILE SETTINGS ]" }
            }
            div { class: "tui-panel-body",
                h2 { "Profile Settings" }
                p { class: "settings-description", "Manage your public profile and social links." }
                form {
                    class: "settings-form",
                    method: "post",
                    action: "{spec.profile_endpoint}",
                    "data-profile-endpoint": "{spec.profile_endpoint}",
                    "data-csrf-endpoint": "{spec.csrf_endpoint}",
                    onsubmit: on_submit,
                    if !error_list.is_empty() {
                        {upload_error_summary(&error_list)}
                    }
                    label { r#for: "settings-display-name", "Display Name" }
                    input {
                        id: "settings-display-name",
                        r#type: "text",
                        name: "display_name",
                        value: "{display_name_value}",
                        "aria-invalid": "{display_name_error.is_some()}",
                        "aria-describedby": "settings-display-name-error",
                    }
                    {field_error_element(
                        "settings-display-name-error",
                        display_name_error,
                    )}
                    label { r#for: "settings-bio", "Bio" }
                    textarea {
                        id: "settings-bio",
                        name: "bio",
                        "aria-invalid": "{bio_error.is_some()}",
                        "aria-describedby": "settings-bio-error",
                        "{bio_value}"
                    }
                    {field_error_element("settings-bio-error", bio_error)}
                    label { r#for: "settings-website", "Website" }
                    input {
                        id: "settings-website",
                        r#type: "url",
                        name: "website_url",
                        value: "{website_value}",
                        "aria-invalid": "{website_error.is_some()}",
                        "aria-describedby": "settings-website-error",
                    }
                    {field_error_element(
                        "settings-website-error",
                        website_error,
                    )}
                    label { r#for: "settings-github", "GitHub" }
                    input {
                        id: "settings-github",
                        r#type: "url",
                        name: "github_url",
                        value: "{github_value}",
                        "aria-invalid": "{github_error.is_some()}",
                        "aria-describedby": "settings-github-error",
                    }
                    {field_error_element(
                        "settings-github-error",
                        github_error,
                    )}
                    label { r#for: "settings-x", "X" }
                    input {
                        id: "settings-x",
                        r#type: "url",
                        name: "twitter_url",
                        value: "{twitter_value}",
                        "aria-invalid": "{twitter_error.is_some()}",
                        "aria-describedby": "settings-x-error",
                    }
                    {field_error_element("settings-x-error", twitter_error)}
                    label { r#for: "settings-bluesky", "Bluesky" }
                    input {
                        id: "settings-bluesky",
                        r#type: "url",
                        name: "bluesky_url",
                        value: "{bluesky_value}",
                        "aria-invalid": "{bluesky_error.is_some()}",
                        "aria-describedby": "settings-bluesky-error",
                    }
                    {field_error_element(
                        "settings-bluesky-error",
                        bluesky_error,
                    )}
                    label { r#for: "settings-discord", "Discord" }
                    input {
                        id: "settings-discord",
                        r#type: "url",
                        name: "discord_url",
                        value: "{discord_value}",
                        "aria-invalid": "{discord_error.is_some()}",
                        "aria-describedby": "settings-discord-error",
                    }
                    {field_error_element(
                        "settings-discord-error",
                        discord_error,
                    )}
                    label { r#for: "settings-sponsor", "Sponsor" }
                    input {
                        id: "settings-sponsor",
                        r#type: "url",
                        name: "sponsor_url",
                        value: "{sponsor_value}",
                        "aria-invalid": "{sponsor_error.is_some()}",
                        "aria-describedby": "settings-sponsor-error",
                    }
                    {field_error_element(
                        "settings-sponsor-error",
                        sponsor_error,
                    )}
                }
            }
        }
    }
}

#[component]
pub(crate) fn SettingsNotifications() -> Element {
    let auth = use_auth();
    let notifications = use_notifications();
    let preferences = preferences_or_default(notifications.preferences());
    let saving = notifications.saving();
    let status = notifications.status();
    let disabled = !auth.authenticated;
    let status_message = match status {
        Some(SaveStatus::Saved) => Some("Preferences saved."),
        Some(SaveStatus::Error) => Some("Failed to save preferences."),
        None => None,
    };
    let status_attr = match status {
        Some(SaveStatus::Saved) => "saved",
        Some(SaveStatus::Error) => "error",
        None => "idle",
    };

    rsx! {
        section { class: "settings-panel tui-panel",
            div { class: "tui-panel-header",
                span { class: "tui-panel-title", "[ NOTIFICATIONS ]" }
            }
            div { class: "tui-panel-body",
                h2 { "Notifications" }
                p { class: "settings-description", "Manage how and when you receive notifications." }
                if disabled {
                    p { class: "settings-note", "Log in to manage your notification preferences." }
                }
                div { class: "notification-grid",
                    div { class: "notification-header",
                        span { "" }
                        span { class: "notification-channel",
                            svg {
                                class: "notification-channel-icon",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "m22 7-8.991 5.727a2 2 0 0 1-2.009 0L2 7" }
                                rect { x: "2", y: "4", width: "20", height: "16", rx: "2" }
                            }
                            span { "Email" }
                        }
                        span { class: "notification-channel",
                            svg {
                                class: "notification-channel-icon",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                                path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
                            }
                            span { "In-app" }
                        }
                    }
                    {preferences.into_iter().map(|preference| {
                        let kind = preference.kind;
                        let label = notification_label(kind);
                        let description = notification_description(kind);
                        let icon = match notification_icon(kind) {
                            NotificationIcon::Download => rsx!(svg {
                                class: "notification-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M12 15V3" }
                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                path { d: "m7 10 5 5 5-5" }
                            }),
                            NotificationIcon::Comment => rsx!(svg {
                                class: "notification-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M2.992 16.342a2 2 0 0 1 .094 1.167l-1.065 3.29a1 1 0 0 0 1.236 1.168l3.413-.998a2 2 0 0 1 1.099.092 10 10 0 1 0-4.777-4.719" }
                            }),
                            NotificationIcon::Star => rsx!(svg {
                                class: "notification-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5" }
                            }),
                            NotificationIcon::Update => rsx!(svg {
                                class: "notification-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z" }
                            }),
                            NotificationIcon::Announce => rsx!(svg {
                                class: "notification-icon",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.77 4.78 4 4 0 0 1-6.75 0 4 4 0 0 1-4.78-4.77 4 4 0 0 1 0-6.76Z" }
                                path { d: "m9 12 2 2 4-4" }
                            }),
                        };
                        let key = format!("notification-{label}");
                        let name = notification_key(kind);
                        let email_name = format!("notify_{name}_email");
                        let inapp_name = format!("notify_{name}_inapp");
                        let email_label = format!("Email notifications for {label}");
                        let inapp_label = format!("In-app notifications for {label}");
                        let notifications_email = notifications.clone();
                        let notifications_inapp = notifications.clone();
                        rsx! {
                            div { class: "notification-row", key: "{key}",
                                div { class: "notification-info",
                                    {icon}
                                    div { class: "notification-text",
                                        span { class: "notification-label", "{label}" }
                                        span { class: "notification-description", "{description}" }
                                    }
                                }
                                div { class: "notification-toggles",
                                    label { class: "toggle",
                                        input {
                                            r#type: "checkbox",
                                            name: "{email_name}",
                                            aria_label: "{email_label}",
                                            checked: preference.email,
                                            disabled,
                                            onchange: move |evt| {
                                                notifications_email.update_preference(
                                                    kind,
                                                    PreferenceField::Email,
                                                    evt.checked(),
                                                );
                                            }
                                        }
                                        span { class: "toggle-track",
                                            span { class: "toggle-thumb" }
                                        }
                                        span { class: "sr-only", "{email_label}" }
                                    }
                                    label { class: "toggle",
                                        input {
                                            r#type: "checkbox",
                                            name: "{inapp_name}",
                                            aria_label: "{inapp_label}",
                                            checked: preference.in_app,
                                            disabled,
                                            onchange: move |evt| {
                                                notifications_inapp.update_preference(
                                                    kind,
                                                    PreferenceField::InApp,
                                                    evt.checked(),
                                                );
                                            }
                                        }
                                        span { class: "toggle-track",
                                            span { class: "toggle-thumb" }
                                        }
                                        span { class: "sr-only", "{inapp_label}" }
                                    }
                                }
                            }
                        }
                    })}
                }
                if let Some(message) = status_message {
                    div { class: "settings-status", "aria-live": "polite", "data-state": "{status_attr}",
                        "{message}"
                    }
                }
                div { class: "form-actions",
                    button {
                        class: "form-submit",
                        r#type: "button",
                        disabled: disabled || saving,
                        onclick: move |_| notifications.save_preferences(),
                        if saving {
                            "Saving..."
                        } else {
                            "Save preferences"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn SettingsSecurity() -> Element {
    let auth = use_auth();
    #[cfg(target_arch = "wasm32")]
    let navigator = use_navigator();
    let delete_confirm = use_signal(String::new);
    let delete_pending = use_signal(|| false);
    let delete_error = use_signal(|| None::<String>);

    let expected_username = match &auth.state {
        RemoteState::Ready(session) => session.user.as_ref().map(|user| user.login.clone()),
        _ => None,
    };

    let confirm_value = delete_confirm.read();
    let confirm_matches = expected_username
        .as_ref()
        .map(|name| confirm_value.trim() == name.as_str())
        .unwrap_or(false);
    let is_deleting = *delete_pending.read();
    let export_disabled = !auth.authenticated;
    let delete_disabled = !auth.authenticated || is_deleting || !confirm_matches;
    let delete_message = delete_error.read().clone();
    let delete_label = if is_deleting {
        "Deleting..."
    } else {
        "Delete Account"
    };
    let export_disabled_attr = if export_disabled { "true" } else { "false" };
    let export_tabindex = if export_disabled { "-1" } else { "0" };
    let delete_placeholder = expected_username.as_deref().unwrap_or("username");

    let on_delete_input = {
        let mut delete_confirm = delete_confirm;
        let mut delete_error = delete_error;
        move |evt: FormEvent| {
            delete_confirm.set(evt.value());
            delete_error.set(None);
        }
    };

    #[cfg(target_arch = "wasm32")]
    let delete_account = {
        let mut delete_pending = delete_pending;
        let mut delete_error = delete_error;
        let delete_confirm = delete_confirm;
        let navigator = navigator.clone();
        move |_| {
            delete_pending.set(true);
            delete_error.set(None);
            let mut delete_pending = delete_pending;
            let mut delete_error = delete_error;
            let mut delete_confirm = delete_confirm;
            let navigator = navigator.clone();
            spawn(async move {
                match api::delete_account().await {
                    Ok(_) => {
                        let _ = api::logout().await;
                        delete_pending.set(false);
                        delete_confirm.set(String::new());
                        navigator.replace(Route::Home {});
                    }
                    Err(err) => {
                        delete_pending.set(false);
                        delete_error.set(Some(err.to_string()));
                    }
                }
            });
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let delete_account = move |_| {};

    rsx! {
        section { class: "settings-panel tui-panel",
            div { class: "tui-panel-header",
                span { class: "tui-panel-title", "[ SECURITY ]" }
            }
            div { class: "tui-panel-body",
                h2 { "Security Settings" }
                p { class: "settings-description", "Review active sessions." }
                if !auth.authenticated {
                    p { class: "settings-note", "Log in to export data or delete your account." }
                }
                div { class: "security-section",
                    h3 { "Connected Accounts" }
                    p { "GitHub OAuth connected" }
                }
                div { class: "security-section",
                    h3 { "Sessions" }
                    p { "Active session in Cloudflare Pages" }
                }
                div { class: "security-section",
                    h3 { "Data Export" }
                    p { "Download a copy of your registry data." }
                    a {
                        class: "ghost-button",
                        href: "/api/users/me/export",
                        "aria-disabled": "{export_disabled_attr}",
                        tabindex: "{export_tabindex}",
                        "Export Data"
                    }
                }
                div { class: "security-section danger-zone",
                    h3 { "Danger Zone" }
                    p { "Delete your account and registry data." }
                    label { r#for: "settings-delete-confirm", "Type your username to confirm." }
                    input {
                        id: "settings-delete-confirm",
                        r#type: "text",
                        name: "delete_confirm",
                        placeholder: "{delete_placeholder}",
                        disabled: !auth.authenticated || is_deleting,
                        oninput: on_delete_input,
                    }
                    button {
                        class: "danger-button",
                        r#type: "button",
                        disabled: delete_disabled,
                        onclick: delete_account,
                        "{delete_label}"
                    }
                    if let Some(message) = delete_message {
                        div { class: "settings-status", "data-state": "error",
                            "{message}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn LoginRoute(redirect_to: Option<String>) -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let auth_href = github_auth_href(redirect_to.as_deref());
    let redirect_target = login_redirect_target(auth.authenticated, redirect_to.as_deref());

    use_effect(move || {
        if let Some(target) = redirect_target.clone() {
            let _ = match target.parse::<NavigationTarget<Route>>() {
                Ok(target) => navigator.replace(target),
                Err(_) => navigator.replace(Route::Home {}),
            };
        }
    });

    rsx! {
        section { class: "login-route",
            div { class: "login-card tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ LOGIN ]" }
                }
                div { class: "tui-panel-body",
                    h1 { class: "login-title", "Log in to Barforge" }
                    p { class: "login-text", "Access your modules and settings." }
                    a { class: "github-btn", href: "{auth_href}",
                        svg {
                            class: "github-icon",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "currentColor",
                            "aria-hidden": "true",
                            path {
                                d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                            }
                        }
                        "Continue with GitHub"
                    }
                    p { class: "terms",
                        "By continuing, you agree to our "
                        a { href: "/terms", "Terms of Service" }
                        " and "
                        a { href: "/privacy", "Privacy Policy" }
                        "."
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DashboardRoute() -> Element {
    let auth = use_auth();
    let username = match &auth.state {
        RemoteState::Ready(session) => session
            .user
            .as_ref()
            .map(|user| user.login.as_str())
            .unwrap_or("barforge"),
        _ => "barforge",
    }
    .to_string();
    let profile_state = use_user_profile_me_state(auth.authenticated);
    let modules_state = use_modules_mine_state(auth.authenticated);
    let collections_state = use_collections_state(auth.authenticated);
    let view = user_profile_view_from_state(
        &username,
        &profile_state,
        &modules_state,
        &collections_state,
    );
    let profile = view.profile;
    let user_modules = view.modules;
    let collections = view.collections;
    let total_downloads = view.total_downloads;
    let profile_errors = use_signal(Vec::<FieldError>::new);
    let profile_error_list = profile_errors.read().clone();
    let profile_spec = profile_request_spec();
    let on_profile_submit = {
        let mut errors = profile_errors;
        move |evt: FormEvent| {
            evt.prevent_default();
            let values = profile_form_values(&evt);
            let validation = validate_profile_form(&values);
            if !validation.is_empty() {
                errors.set(validation);
                return;
            }
            errors.set(Vec::new());
            #[cfg(target_arch = "wasm32")]
            {
                let payload = update_profile_request(&values);
                let errors = errors.clone();
                spawn(async move {
                    if submit_profile_update(payload, profile_spec).await.is_err() {
                        let mut errors = errors.clone();
                        errors.set(vec![FieldError {
                            field: "form",
                            message: "Profile update failed. Please try again.",
                        }]);
                    }
                });
            }
        }
    };
    let collection_errors = use_signal(Vec::<FieldError>::new);
    let collection_error_list = collection_errors.read().clone();
    let on_collection_create = {
        let mut errors = collection_errors;
        move |evt: FormEvent| {
            evt.prevent_default();
            let values = evt.values();
            let name = form_value(&values, "collection_name");
            if name.trim().is_empty() {
                errors.set(vec![FieldError {
                    field: "collection_name",
                    message: "Collection name is required.",
                }]);
                return;
            }
            errors.set(Vec::new());
            #[cfg(target_arch = "wasm32")]
            {
                let name = name.trim().to_string();
                let description = form_value(&values, "collection_description");
                let visibility = form_value(&values, "collection_visibility");
                let description = description.trim().to_string();
                let visibility = visibility.trim().to_string();
                let payload = serde_json::json!({
                    "name": name,
                    "description": if description.is_empty() { None::<String> } else { Some(description) },
                    "visibility": if visibility.is_empty() { None::<String> } else { Some(visibility) },
                });
                if let Ok(body) = serde_json::to_string(&payload) {
                    spawn(async move {
                        let _ = api::create_collection(body).await;
                    });
                }
            }
        }
    };
    let display_name_error = field_error_message(&profile_error_list, "display_name");
    let bio_error = field_error_message(&profile_error_list, "bio");
    let website_error = field_error_message(&profile_error_list, "website_url");
    let display_name_value = profile.display_name.clone().unwrap_or_default();
    let bio_value = profile.bio.clone().unwrap_or_default();
    let website_value = profile.website_url.clone().unwrap_or_default();
    let now = app_now();

    rsx! {
        ProtectedRoute {
            section { class: "dashboard-layout",
                h1 { "Dashboard" }
                p { class: "dashboard-subtitle", "Manage your profile, modules, and collections." }
                div { class: "dashboard-grid",
                    section { class: "dashboard-panel dashboard-profile tui-panel",
                        div { class: "tui-panel-header",
                            span { class: "tui-panel-title", "[ PROFILE ]" }
                        }
                        div { class: "tui-panel-body",
                            h2 { "Profile overview" }
                            div { class: "dashboard-metrics",
                                div { class: "metric",
                                    span { class: "metric-label", "Total downloads" }
                                    strong { "{total_downloads}" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Modules" }
                                    strong { "{profile.module_count}" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Member since" }
                                    strong { "{profile.created_at}" }
                                }
                            }
                            div { class: "dashboard-actions",
                                a { class: "ghost-button", href: "/settings/profile", "Edit profile" }
                                a { class: "ghost-button", href: "/upload", "Upload module" }
                            }
                            form {
                                class: "dashboard-form",
                                method: "post",
                                action: "{profile_spec.profile_endpoint}",
                                "data-dashboard-action": "profile-update",
                                "data-profile-endpoint": "{profile_spec.profile_endpoint}",
                                "data-csrf-endpoint": "{profile_spec.csrf_endpoint}",
                                onsubmit: on_profile_submit,
                                if !profile_error_list.is_empty() {
                                    {upload_error_summary(&profile_error_list)}
                                }
                                label { r#for: "dashboard-display-name", "Display Name" }
                                input {
                                    id: "dashboard-display-name",
                                    r#type: "text",
                                    name: "display_name",
                                    value: "{display_name_value}",
                                    "aria-invalid": "{display_name_error.is_some()}",
                                    "aria-describedby": "dashboard-display-name-error",
                                }
                                {field_error_element(
                                    "dashboard-display-name-error",
                                    display_name_error,
                                )}
                                label { r#for: "dashboard-bio", "Bio" }
                                textarea {
                                    id: "dashboard-bio",
                                    name: "bio",
                                    "aria-invalid": "{bio_error.is_some()}",
                                    "aria-describedby": "dashboard-bio-error",
                                    "{bio_value}"
                                }
                                {field_error_element("dashboard-bio-error", bio_error)}
                                label { r#for: "dashboard-website", "Website" }
                                input {
                                    id: "dashboard-website",
                                    r#type: "url",
                                    name: "website_url",
                                    value: "{website_value}",
                                    "aria-invalid": "{website_error.is_some()}",
                                    "aria-describedby": "dashboard-website-error",
                                }
                                {field_error_element("dashboard-website-error", website_error)}
                                button { class: "ghost-button", r#type: "submit", "Save profile" }
                            }
                        }
                    }
                    section { class: "dashboard-panel dashboard-modules tui-panel",
                        div { class: "tui-panel-header",
                            span { class: "tui-panel-title", "[ MODULES ]" }
                        }
                        div { class: "tui-panel-body",
                            h2 { "Your modules" }
                            StarredModuleList {
                                modules: user_modules,
                                query: None,
                                sort: Some(ModuleSort::Recent),
                                category: None,
                                page: None,
                                per_page: Some(6),
                                view_mode: None,
                                now,
                            }
                        }
                    }
                }
                if !collections.is_empty() {
                    section { class: "dashboard-panel dashboard-collections tui-panel",
                        div { class: "tui-panel-header",
                            span { class: "tui-panel-title", "[ COLLECTIONS ]" }
                        }
                        div { class: "tui-panel-body",
                            h2 { "Collections" }
                            form {
                                class: "dashboard-form",
                                method: "post",
                                action: "/api/collections",
                                "data-dashboard-action": "collection-create",
                                "data-csrf-endpoint": "/api/csrf-token",
                                onsubmit: on_collection_create,
                                if !collection_error_list.is_empty() {
                                    {upload_error_summary(&collection_error_list)}
                                }
                                label { r#for: "dashboard-collection-name", "Collection name" }
                                input {
                                    id: "dashboard-collection-name",
                                    r#type: "text",
                                    name: "collection_name",
                                }
                                label { r#for: "dashboard-collection-description", "Description" }
                                textarea {
                                    id: "dashboard-collection-description",
                                    name: "collection_description",
                                }
                                label { r#for: "dashboard-collection-visibility", "Visibility" }
                                select { id: "dashboard-collection-visibility", name: "collection_visibility",
                                    option { value: "private", "Private" }
                                    option { value: "unlisted", "Unlisted" }
                                    option { value: "public", "Public" }
                                }
                                button { class: "ghost-button", r#type: "submit", "Create collection" }
                            }
                            div { class: "collections-grid",
                                {collections.into_iter().map(|collection| {
                                    let description = collection.description.as_ref().map(|text| {
                                        rsx!(p { class: "collection-description", "{text}" })
                                    });
                                    let module_label = if collection.module_count == 1 { "module" } else { "modules" };
                                    let collection_id = collection.id;
                                    let collection_name = collection.name.clone();
                                    let collection_description = collection.description.clone().unwrap_or_default();
                                    let collection_visibility = collection.visibility.clone();
                                    let on_update_collection = move |evt: FormEvent| {
                                        evt.prevent_default();
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            let values = evt.values();
                                            let name = form_value(&values, "collection_name");
                                            let description = form_value(&values, "collection_description");
                                            let visibility = form_value(&values, "collection_visibility");
                                            let payload = serde_json::json!({
                                                "name": if name.trim().is_empty() { None::<String> } else { Some(name.trim().to_string()) },
                                                "description": if description.trim().is_empty() { None::<String> } else { Some(description.trim().to_string()) },
                                                "visibility": if visibility.trim().is_empty() { None::<String> } else { Some(visibility.trim().to_string()) },
                                            });
                                            if let Ok(body) = serde_json::to_string(&payload) {
                                                spawn(async move {
                                                    let _ = api::update_collection(collection_id, body).await;
                                                });
                                            }
                                        }
                                    };
                                    let on_delete_collection = move |evt: FormEvent| {
                                        evt.prevent_default();
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            spawn(async move {
                                                let _ = api::delete_collection(collection_id).await;
                                            });
                                        }
                                    };
                                    rsx!(
                                        div { key: "{collection.id}", class: "collection-card",
                                            a { class: "collection-link", href: "/collections/{collection.id}",
                                                div { class: "collection-header",
                                                    span { class: "collection-name", "{collection.name}" }
                                                }
                                            }
                                            {description}
                                            div { class: "collection-meta",
                                                "{collection.module_count} {module_label}"
                                            }
                                            form {
                                                class: "collection-form",
                                                method: "post",
                                                action: "/api/collections/{collection.id}",
                                                "data-dashboard-action": "collection-update",
                                                "data-csrf-endpoint": "/api/csrf-token",
                                                onsubmit: on_update_collection,
                                                label { r#for: "collection-{collection_id}-name", "Name" }
                                                input {
                                                    id: "collection-{collection_id}-name",
                                                    r#type: "text",
                                                    name: "collection_name",
                                                    value: "{collection_name}",
                                                }
                                                label { r#for: "collection-{collection_id}-description", "Description" }
                                                textarea {
                                                    id: "collection-{collection_id}-description",
                                                    name: "collection_description",
                                                    "{collection_description}"
                                                }
                                                label { r#for: "collection-{collection_id}-visibility", "Visibility" }
                                                select {
                                                    id: "collection-{collection_id}-visibility",
                                                    name: "collection_visibility",
                                                    option {
                                                        value: "private",
                                                        selected: collection_visibility == "private",
                                                        "Private"
                                                    }
                                                    option {
                                                        value: "unlisted",
                                                        selected: collection_visibility == "unlisted",
                                                        "Unlisted"
                                                    }
                                                    option {
                                                        value: "public",
                                                        selected: collection_visibility == "public",
                                                        "Public"
                                                    }
                                                }
                                                button { class: "ghost-button", r#type: "submit", "Update" }
                                            }
                                            form {
                                                class: "collection-form",
                                                method: "post",
                                                action: "/api/collections/{collection.id}",
                                                "data-dashboard-action": "collection-delete",
                                                "data-csrf-endpoint": "/api/csrf-token",
                                                onsubmit: on_delete_collection,
                                                button { class: "danger-button", r#type: "submit", "Delete" }
                                            }
                                        }
                                    )
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn StarsRoute() -> Element {
    let auth = use_auth();
    let stars = use_stars();
    let starred = stars.starred_set();
    let modules = stars
        .cached_modules()
        .into_iter()
        .filter(|module| starred.contains(&module.uuid))
        .collect::<Vec<_>>();
    let login_href = auth_redirect::login_redirect_url("/stars");
    let now = app_now();

    rsx! {
        section { class: "stars-layout",
            section { class: "stars-panel tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ STARS ]" }
                }
                div { class: "tui-panel-body",
                    header { class: "stars-header",
                        div { class: "stars-header-text",
                            h1 { "Starred modules" }
                            p { class: "stars-subtitle", "Modules you saved for later" }
                        }
                        if !auth.authenticated {
                            div { class: "stars-auth",
                                p { "Log in to sync your stars" }
                                a { class: "ghost-button", href: "{login_href}", "Log in" }
                            }
                        }
                    }
                    if stars.syncing() {
                        div { class: "stars-loading", "Loading your starred modules..." }
                    } else if !modules.is_empty() {
                        StarredModuleList {
                            modules,
                            query: None,
                            sort: Some(ModuleSort::Recent),
                            category: None,
                            page: None,
                            per_page: Some(6),
                            view_mode: None,
                            now,
                        }
                    } else {
                        div { class: "stars-empty",
                            h2 { "No starred modules yet" }
                            p { "Star modules while browsing to quickly access them later." }
                            a { class: "ghost-button", href: "/modules", "Browse modules" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn CollectionDetailRoute(id: CollectionId) -> Element {
    let id = id.as_str();
    #[cfg(target_arch = "wasm32")]
    let Ok(id_value) = id.parse::<i64>() else {
        FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);
        return NotFoundRoute(NotFoundRouteProps {
            segments: vec!["collections".to_string(), id.to_string()],
        });
    };
    #[cfg(not(target_arch = "wasm32"))]
    let id_value = 0;

    let detail_state = use_collection_detail_state(id_value);
    let auth = use_auth();
    let retry_href = format!("/collections/{id}");
    let modules_override = use_signal(|| None::<Vec<CollectionModule>>);
    let removing_uuid = use_signal(|| None::<String>);
    let remove_error = use_signal(|| None::<String>);

    match &detail_state {
        RemoteState::Loading => {
            return status_banner_with_retry(
                &detail_state,
                "Loading collection...",
                "Collection unavailable",
                "Please try again.",
                Some(&retry_href),
            );
        }
        RemoteState::Error(message) => {
            if message.contains("404") {
                FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);
                return NotFoundRoute(NotFoundRouteProps {
                    segments: vec!["collections".to_string(), id.to_string()],
                });
            }
            if message.contains("403") {
                FullstackContext::commit_http_status(StatusCode::FORBIDDEN, None);
            }
            return status_banner_with_retry(
                &detail_state,
                "Loading collection...",
                "Collection unavailable",
                "Please try again.",
                Some(&retry_href),
            );
        }
        RemoteState::Ready(_) | RemoteState::Unavailable => {}
    }

    let (collection, base_modules) = match detail_state {
        RemoteState::Ready(payload) => (payload.collection, payload.modules),
        RemoteState::Unavailable => (sample_collection(id), sample_collection_modules()),
        RemoteState::Loading | RemoteState::Error(_) => (sample_collection(id), Vec::new()),
    };

    let modules = modules_override.read().clone().unwrap_or(base_modules);
    let viewer_login = match &auth.state {
        RemoteState::Ready(session) => session
            .user
            .as_ref()
            .map(|user| normalize_username(Some(user.login.as_str()))),
        _ => None,
    };
    let owner_login = normalize_username(Some(collection.owner.username.as_str()));
    let is_owner = viewer_login
        .as_ref()
        .map(|login| login == &owner_login)
        .unwrap_or(false);
    let removing_uuid_value = removing_uuid.read().clone();
    let remove_error_value = remove_error.read().clone();

    #[cfg(target_arch = "wasm32")]
    let on_remove = {
        let modules_override = modules_override;
        let mut removing_uuid = removing_uuid;
        let mut remove_error = remove_error;
        let modules_snapshot = modules.clone();
        Some(EventHandler::new(move |uuid: String| {
            remove_error.set(None);
            removing_uuid.set(Some(uuid.clone()));
            let mut modules_override = modules_override;
            let mut removing_uuid = removing_uuid;
            let mut remove_error = remove_error;
            let current_modules = modules_override
                .read()
                .clone()
                .unwrap_or_else(|| modules_snapshot.clone());
            spawn(async move {
                match api::remove_collection_module(id_value, &uuid).await {
                    Ok(_) => {
                        let remaining = current_modules
                            .into_iter()
                            .filter(|module| module.uuid != uuid)
                            .collect();
                        modules_override.set(Some(remaining));
                        removing_uuid.set(None);
                    }
                    Err(err) => {
                        removing_uuid.set(None);
                        remove_error.set(Some(err.to_string()));
                    }
                }
            });
        }))
    };

    #[cfg(not(target_arch = "wasm32"))]
    let on_remove: Option<EventHandler<String>> = None;

    collection_detail_body(
        collection,
        modules,
        is_owner,
        removing_uuid_value,
        remove_error_value,
        on_remove,
    )
}

fn collection_detail_body(
    collection: Collection,
    modules: Vec<CollectionModule>,
    is_owner: bool,
    removing_uuid: Option<String>,
    remove_error: Option<String>,
    on_remove: Option<EventHandler<String>>,
) -> Element {
    let Collection {
        name,
        description,
        owner,
        visibility: _,
        updated_at,
        ..
    } = collection;
    let CollectionOwner {
        username,
        display_name,
        ..
    } = owner;
    let owner_label = display_name.unwrap_or_else(|| username.clone());
    let module_count = modules.len();
    let module_label = if module_count == 1 {
        "module"
    } else {
        "modules"
    };
    let error_banner = remove_error.map(|message| {
        rsx! {
            div { class: "index-status error", role: "alert", "aria-live": "assertive",
                span { class: "index-status-title", "Remove failed" }
                span { class: "index-status-detail", "{message}" }
            }
        }
    });

    let modules_content = if modules.is_empty() {
        let message = if is_owner {
            "Add modules to this collection from their detail pages."
        } else {
            "No modules have been added to this collection yet."
        };
        rsx! {
            div { class: "stars-empty",
                h2 { "This collection is empty" }
                p { "{message}" }
            }
        }
    } else {
        rsx! {
            ul { class: "module-list",
                {modules.into_iter().map(|module| {
                    let CollectionModule {
                        uuid,
                        name,
                        author,
                        category,
                        note,
                        ..
                    } = module;
                    let note_node = note
                        .as_ref()
                        .map(|note| rsx!(p { class: "module-description", "{note}" }));
                    let category_label = category_label(&category);
                    let remove_label = if removing_uuid.as_deref() == Some(uuid.as_str()) {
                        "Removing..."
                    } else {
                        "Remove"
                    };
                    let remove_disabled = removing_uuid.as_deref() == Some(uuid.as_str());
                    let uuid_clone = uuid.clone();
                    let remove_button = if let Some(handler) = on_remove.as_ref() {
                        let handler = *handler;
                        rsx! {
                            button {
                                class: "ghost-button",
                                r#type: "button",
                                disabled: remove_disabled,
                                onclick: move |_| {
                                    handler.call(uuid_clone.clone());
                                },
                                "{remove_label}"
                            }
                        }
                    } else {
                        rsx! {
                            button {
                                class: "ghost-button",
                                r#type: "button",
                                disabled: remove_disabled,
                                "{remove_label}"
                            }
                        }
                    };
                    rsx! {
                        li { key: "{uuid}",
                            a { class: "module-row", href: "/modules/{uuid}",
                                h3 { "{name}" }
                                p { class: "module-author", "by {author}" }
                                p { class: "module-category", "{category_label}" }
                            }
                            {note_node}
                            if is_owner {
                                {remove_button}
                            }
                        }
                    }
                })}
            }
        }
    };

    rsx! {
        section { class: "collection-layout",
            section { class: "collection-panel tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ COLLECTION ]" }
                }
                div { class: "tui-panel-body",
                    header { class: "collection-header",
                        h1 { "{name}" }
                        if let Some(description) = description {
                            p { class: "collection-description", "{description}" }
                        }
                        p { class: "collection-meta",
                            a { href: "/users/{username}", "{owner_label}" }
                            " · {module_count} {module_label} · Updated {updated_at}"
                        }
                    }
                }
            }
            section { class: "collection-modules tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ MODULES ]" }
                }
                div { class: "tui-panel-body",
                    if let Some(error_banner) = error_banner {
                        {error_banner}
                    }
                    {modules_content}
                }
            }
        }
    }
}

#[component]
pub(crate) fn UserProfileRoute(username: GithubUsername) -> Element {
    let username = username.as_str();
    let profile_state = use_user_profile_state(username);
    let modules_state = use_user_modules_state(username);
    let collections_state = use_user_collections_state(username);
    let retry_href = format!("/users/{username}");

    match &profile_state {
        RemoteState::Loading => {
            return status_banner_with_retry(
                &profile_state,
                "Loading profile...",
                "Profile unavailable",
                "Please try again.",
                None,
            );
        }
        RemoteState::Error(message) => {
            if message.contains("404") {
                FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);
                return NotFoundRoute(NotFoundRouteProps {
                    segments: vec!["users".to_string(), username.to_string()],
                });
            }
            let title = "Profile unavailable";
            return status_banner_with_retry(
                &profile_state,
                "Loading profile...",
                title,
                "Please try again.",
                Some(&retry_href),
            );
        }
        RemoteState::Ready(_) | RemoteState::Unavailable => {}
    }

    let view =
        user_profile_view_from_state(username, &profile_state, &modules_state, &collections_state);
    let profile = view.profile;
    let modules = view.modules;
    let collections = view.collections;
    let total_downloads = view.total_downloads;
    let display_name = profile.display_name.as_deref().unwrap_or(&profile.username);
    let downloads_label = format_number(total_downloads);
    let now = app_now();

    rsx! {
        section { class: "user-profile",
            section { class: "profile-panel tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ PROFILE ]" }
                }
                div { class: "tui-panel-body",
                    header { class: "profile-header",
                        div { class: "profile-title",
                            h1 { "User Profile" }
                            span { class: "profile-handle", "@{profile.username}" }
                            p { class: "profile-name", "{display_name}" }
                        }
                        div { class: "profile-stats",
                            span { "{profile.module_count} modules" }
                            if total_downloads > 0 {
                                span { "{downloads_label} downloads" }
                            }
                            span { "Member since {profile.created_at}" }
                        }
                        div { class: "profile-socials",
                            if let Some(url) = profile.github_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        path {
                                            d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                                        }
                                    }
                                }
                            }
                            if let Some(url) = profile.twitter_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        path {
                                            d: "M14.234 10.162 22.977 0h-2.072l-7.591 8.824L7.251 0H.258l9.168 13.343L.258 24H2.33l8.016-9.318L16.749 24h6.993zm-2.837 3.299-.929-1.329L3.076 1.56h3.182l5.965 8.532.929 1.329 7.754 11.09h-3.182z"
                                        }
                                    }
                                }
                            }
                            if let Some(url) = profile.bluesky_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        path { d: "M5.202 2.857C7.954 4.922 10.913 9.11 12 11.358c1.087-2.247 4.046-6.436 6.798-8.501C20.783 1.366 24 .213 24 3.883c0 .732-.42 6.156-.667 7.037-.856 3.061-3.978 3.842-6.755 3.37 4.854.826 6.089 3.562 3.422 6.299-5.065 5.196-7.28-1.304-7.847-2.97-.104-.305-.152-.448-.153-.327 0-.121-.05.022-.153.327-.568 1.666-2.782 8.166-7.847 2.97-2.667-2.737-1.432-5.473 3.422-6.3-2.777.473-5.899-.308-6.755-3.369C.42 10.04 0 4.615 0 3.883c0-3.67 3.217-2.517 5.202-1.026" }
                                    }
                                }
                            }
                            if let Some(url) = profile.discord_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        path { d: "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z" }
                                    }
                                }
                            }
                            if let Some(url) = profile.website_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        "aria-hidden": "true",
                                        circle { cx: "12", cy: "12", r: "10" }
                                        path { d: "M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" }
                                        path { d: "M2 12h20" }
                                    }
                                }
                            }
                            if let Some(url) = profile.sponsor_url.as_deref() {
                                a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                                    svg {
                                        class: "profile-social-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        "aria-hidden": "true",
                                        path { d: "M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "profile-modules tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ MODULES ]" }
                }
                div { class: "tui-panel-body",
                    h2 { "Modules by {display_name}" }
                    StarredModuleList {
                        modules,
                        query: None,
                        sort: Some(ModuleSort::Popular),
                        category: None,
                        page: None,
                        per_page: Some(6),
                        view_mode: None,
                        now,
                    }
                }
            }
            if !collections.is_empty() {
                section { class: "profile-collections tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ COLLECTIONS ]" }
                    }
                    div { class: "tui-panel-body",
                        h2 { "Collections" }
                        div { class: "collections-grid",
                            {collections.into_iter().map(|collection| {
                                let description = collection.description.as_ref().map(|text| {
                                    rsx!(p { class: "collection-description", "{text}" })
                                });
                                let module_label = if collection.module_count == 1 { "module" } else { "modules" };
                                rsx!(
                                    a { key: "{collection.id}", class: "collection-card", href: "/collections/{collection.id}",
                                        div { class: "collection-header",
                                            span { class: "collection-name", "{collection.name}" }
                                        }
                                        {description}
                                        div { class: "collection-meta",
                                            "{collection.module_count} {module_label}"
                                        }
                                    }
                                )
                            })}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn UploadRoute() -> Element {
    let errors = use_signal(Vec::<FieldError>::new);
    let error_list = errors.read().clone();
    let spec = upload_request_spec();
    let on_submit = {
        let mut errors = errors;
        move |evt: FormEvent| {
            evt.prevent_default();
            let values = upload_form_values(&evt);
            let validation = validate_upload_form(&values);
            if !validation.is_empty() {
                errors.set(validation);
                return;
            }
            errors.set(Vec::new());
            #[cfg(target_arch = "wasm32")]
            {
                let data = evt.data();
                let errors = errors.clone();
                spawn(async move {
                    if submit_upload_form(data, spec).await.is_err() {
                        let mut errors = errors.clone();
                        errors.set(vec![FieldError {
                            field: "form",
                            message: "Upload failed. Please try again.",
                        }]);
                    }
                });
            }
        }
    };
    let name_error = field_error_message(&error_list, "name");
    let description_error = field_error_message(&error_list, "description");
    let category_error = field_error_message(&error_list, "category");
    let repo_error = field_error_message(&error_list, "repo_url");
    let version_error = field_error_message(&error_list, "version");
    let license_error = field_error_message(&error_list, "license");
    let package_error = field_error_message(&error_list, "package_file");

    rsx! {
        ProtectedRoute {
            section { class: "upload-layout",
                section { class: "upload-panel tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ UPLOAD ]" }
                    }
                    div { class: "tui-panel-body",
                        h1 { "Upload module" }
                        p { class: "upload-subtitle", "Share your module with the community" }
                        form {
                            class: "upload-form",
                            method: "post",
                            enctype: "multipart/form-data",
                            action: "{spec.upload_endpoint}",
                            "data-upload-endpoint": "{spec.upload_endpoint}",
                            "data-csrf-endpoint": "{spec.csrf_endpoint}",
                            onsubmit: on_submit,
                            if !error_list.is_empty() {
                                {upload_error_summary(&error_list)}
                            }
                            label { r#for: "upload-name", "Module name" }
                            input {
                                id: "upload-name",
                                r#type: "text",
                                name: "name",
                                required: true,
                                "aria-invalid": "{name_error.is_some()}",
                                "aria-describedby": "upload-name-error",
                            }
                            {field_error_element("upload-name-error", name_error)}
                            label { r#for: "upload-description", "Description" }
                            textarea {
                                id: "upload-description",
                                name: "description",
                                required: true,
                                "aria-invalid": "{description_error.is_some()}",
                                "aria-describedby": "upload-description-error",
                            }
                            {field_error_element("upload-description-error", description_error)}
                            label { r#for: "upload-category", "Category" }
                            input {
                                id: "upload-category",
                                r#type: "text",
                                name: "category",
                                required: true,
                                "aria-invalid": "{category_error.is_some()}",
                                "aria-describedby": "upload-category-error",
                            }
                            {field_error_element("upload-category-error", category_error)}
                            label { r#for: "upload-repo-url", "Repository URL" }
                            input {
                                id: "upload-repo-url",
                                r#type: "url",
                                name: "repo_url",
                                required: true,
                                "aria-invalid": "{repo_error.is_some()}",
                                "aria-describedby": "upload-repo-url-error",
                            }
                            {field_error_element("upload-repo-url-error", repo_error)}
                            label { r#for: "upload-version", "Version" }
                            input {
                                id: "upload-version",
                                r#type: "text",
                                name: "version",
                                required: true,
                                "aria-invalid": "{version_error.is_some()}",
                                "aria-describedby": "upload-version-error",
                            }
                            {field_error_element("upload-version-error", version_error)}
                            label { r#for: "upload-license", "License" }
                            input {
                                id: "upload-license",
                                r#type: "text",
                                name: "license",
                                required: true,
                                "aria-invalid": "{license_error.is_some()}",
                                "aria-describedby": "upload-license-error",
                            }
                            {field_error_element("upload-license-error", license_error)}
                            label { r#for: "upload-package", "Package file" }
                            input {
                                id: "upload-package",
                                r#type: "file",
                                name: "package_file",
                                required: true,
                                "aria-invalid": "{package_error.is_some()}",
                                "aria-describedby": "upload-package-error",
                            }
                            {field_error_element("upload-package-error", package_error)}
                            label { r#for: "upload-changelog", "Changelog" }
                            textarea { id: "upload-changelog", name: "changelog" }
                            div { class: "form-actions",
                                button { class: "form-submit", r#type: "submit", "Upload module" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn submit_upload_form(
    data: std::rc::Rc<FormData>,
    spec: UploadRequestSpec,
) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        FormData as WebFormData, Headers, HtmlFormElement, Request, RequestCredentials,
        RequestInit, RequestMode, Response,
    };

    let token = api::fetch_csrf_token()
        .await
        .map_err(|err| err.to_string())?;
    let event = data
        .downcast::<web_sys::Event>()
        .ok_or_else(|| "event unavailable".to_string())?;
    let target = event
        .target()
        .ok_or_else(|| "event target unavailable".to_string())?;
    let form = target
        .dyn_into::<HtmlFormElement>()
        .map_err(|_| "form element unavailable".to_string())?;
    let form_data =
        WebFormData::new_with_form(&form).map_err(|_| "form data unavailable".to_string())?;
    let headers = Headers::new().map_err(|err| format!("headers init failed: {err:?}"))?;
    headers
        .append(spec.csrf_header, &token)
        .map_err(|err| format!("headers init failed: {err:?}"))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_credentials(RequestCredentials::Include);
    opts.set_headers(&headers);
    let body = JsValue::from(form_data);
    opts.set_body(&body);

    let request = Request::new_with_str_and_init(spec.upload_endpoint, &opts)
        .map_err(|err| format!("request init failed: {err:?}"))?;
    let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| format!("fetch failed: {err:?}"))?;
    let response: Response = response
        .dyn_into()
        .map_err(|err| format!("invalid response: {err:?}"))?;
    if !response.ok() {
        return Err(format!("upload failed: {}", response.status()));
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn submit_profile_update(
    payload: UpdateProfileRequest,
    spec: ProfileRequestSpec,
) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode, Response};

    let token = api::fetch_csrf_token()
        .await
        .map_err(|err| err.to_string())?;
    let body = serde_json::to_string(&payload).map_err(|err| err.to_string())?;
    let headers = Headers::new().map_err(|err| format!("headers init failed: {err:?}"))?;
    headers
        .append(spec.csrf_header, &token)
        .map_err(|err| format!("headers init failed: {err:?}"))?;
    headers
        .append("content-type", "application/json")
        .map_err(|err| format!("headers init failed: {err:?}"))?;

    let opts = RequestInit::new();
    opts.set_method("PATCH");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_credentials(RequestCredentials::Include);
    opts.set_headers(&headers);
    opts.set_body(&JsValue::from_str(&body));

    let request = Request::new_with_str_and_init(spec.profile_endpoint, &opts)
        .map_err(|err| format!("request init failed: {err:?}"))?;
    let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| format!("fetch failed: {err:?}"))?;
    let response: Response = response
        .dyn_into()
        .map_err(|err| format!("invalid response: {err:?}"))?;
    if !response.ok() {
        return Err(format!("profile update failed: {}", response.status()));
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn reload_admin() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[component]
fn AdminSubmissionCard(submission: Submission) -> Element {
    let action_loading = use_signal(|| false);
    let action_error = use_signal(|| None::<String>);
    let reject_open = use_signal(|| false);
    let mut reject_reason = use_signal(String::new);

    let is_loading = *action_loading.read();
    let show_reject = *reject_open.read();
    let error_message = action_error.read().clone();
    let reject_disabled = reject_reason.read().trim().is_empty() || is_loading;
    let reject_id = format!("reject-reason-{}", submission.id);

    #[cfg(target_arch = "wasm32")]
    let approve_submission = {
        let mut action_loading = action_loading.clone();
        let mut action_error = action_error.clone();
        let submission_id = submission.id;
        move |_| {
            action_loading.set(true);
            action_error.set(None);
            let mut action_loading = action_loading.clone();
            let mut action_error = action_error.clone();
            spawn(async move {
                match api::approve_submission(submission_id).await {
                    Ok(_) => reload_admin(),
                    Err(err) => {
                        action_loading.set(false);
                        action_error.set(Some(err.to_string()));
                    }
                }
            });
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let approve_submission = move |_| {};

    let toggle_reject = {
        let mut reject_open = reject_open;
        let mut reject_reason = reject_reason;
        let mut action_error = action_error;
        move |_| {
            reject_open.set(true);
            reject_reason.set(String::new());
            action_error.set(None);
        }
    };

    let cancel_reject = {
        let mut reject_open = reject_open;
        let mut reject_reason = reject_reason;
        move |_| {
            reject_open.set(false);
            reject_reason.set(String::new());
        }
    };

    #[cfg(target_arch = "wasm32")]
    let submit_reject = {
        let mut action_loading = action_loading;
        let mut action_error = action_error;
        let reject_open = reject_open;
        let reject_reason = reject_reason;
        let submission_id = submission.id;
        move |_| {
            let reason = reject_reason.read().trim().to_string();
            if reason.is_empty() {
                action_error.set(Some("Please provide a rejection reason.".to_string()));
                return;
            }
            action_loading.set(true);
            action_error.set(None);
            let mut action_loading = action_loading.clone();
            let mut action_error = action_error.clone();
            let mut reject_open = reject_open.clone();
            let mut reject_reason = reject_reason.clone();
            spawn(async move {
                match api::reject_submission(submission_id, &reason).await {
                    Ok(_) => {
                        reject_open.set(false);
                        reject_reason.set(String::new());
                        reload_admin();
                    }
                    Err(err) => {
                        action_loading.set(false);
                        action_error.set(Some(err.to_string()));
                    }
                }
            });
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let submit_reject = move |_| {};

    #[cfg(target_arch = "wasm32")]
    let verify_user = {
        let mut action_loading = action_loading.clone();
        let mut action_error = action_error.clone();
        let submitter_id = submission.submitter_id;
        move |_| {
            action_loading.set(true);
            action_error.set(None);
            let mut action_loading = action_loading.clone();
            let mut action_error = action_error.clone();
            spawn(async move {
                match api::verify_user(submitter_id).await {
                    Ok(_) => reload_admin(),
                    Err(err) => {
                        action_loading.set(false);
                        action_error.set(Some(err.to_string()));
                    }
                }
            });
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let verify_user = move |_| {};

    rsx! {
        li { class: "admin-submission",
            div { class: "admin-submission-main",
                span { class: "admin-submission-name", "{submission.name}" }
                span { class: "admin-submission-version", "{submission.version}" }
            }
            div { class: "admin-submission-meta",
                span { class: "admin-submission-user", "{submission.submitter_username}" }
                span { class: "admin-submission-id", "ID {submission.id}" }
            }
            div { class: "admin-submission-actions",
                button {
                    class: "ghost-button",
                    r#type: "button",
                    disabled: is_loading,
                    onclick: approve_submission,
                    "Approve"
                }
                button {
                    class: "danger-button",
                    r#type: "button",
                    disabled: is_loading,
                    onclick: toggle_reject,
                    "Reject"
                }
                button {
                    class: "ghost-button",
                    r#type: "button",
                    disabled: is_loading,
                    onclick: verify_user,
                    "Toggle verify"
                }
            }
            if show_reject {
                div { class: "admin-reject-panel",
                    label { r#for: "{reject_id}", "Rejection reason" }
                    textarea {
                        id: "{reject_id}",
                        name: "reject_reason",
                        onchange: move |evt| reject_reason.set(evt.value()),
                    }
                    div { class: "admin-reject-actions",
                        button {
                            class: "ghost-button",
                            r#type: "button",
                            disabled: is_loading,
                            onclick: cancel_reject,
                            "Cancel"
                        }
                        button {
                            class: "danger-button",
                            r#type: "button",
                            disabled: reject_disabled,
                            onclick: submit_reject,
                            "Confirm reject"
                        }
                    }
                }
            }
            if let Some(message) = error_message {
                div { class: "settings-status", "data-state": "error",
                    "{message}"
                }
            }
        }
    }
}

#[component]
pub(crate) fn AdminRoute() -> Element {
    let auth = use_auth();
    let stats_state = use_admin_stats_state(auth.is_admin);
    let submissions_state = use_admin_submissions_state(auth.is_admin);

    rsx! {
        AdminGate {
            {admin_route_body(&stats_state, &submissions_state)}
        }
    }
}

fn admin_route_body(
    stats_state: &RemoteState<AdminStats>,
    submissions_state: &RemoteState<SubmissionsResponse>,
) -> Element {
    let stats = admin_stats_from_state(stats_state);
    let submissions = admin_submissions_from_state(submissions_state);
    let pending_count = match submissions_state {
        RemoteState::Ready(payload) => payload.total,
        _ => stats.pending_submissions,
    };
    let total_modules = format_number(stats.total_modules);
    let total_users = format_number(stats.total_users);
    let pending_count = format_number(pending_count);
    let show_empty = matches!(submissions_state, RemoteState::Unavailable)
        || matches!(
            submissions_state,
            RemoteState::Ready(payload) if payload.submissions.is_empty()
        );

    rsx! {
        section { class: "admin-layout",
            h1 { "Admin dashboard" }
            p { class: "admin-subtitle", "Manage submissions and keep the registry healthy." }
            div { class: "admin-grid",
                section { class: "admin-panel admin-overview tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ MANAGE SUBMISSIONS ]" }
                    }
                    div { class: "tui-panel-body",
                        h2 { "Manage submissions" }
                        {status_banner_with_retry(
                            stats_state,
                            "Loading admin stats...",
                            "Admin stats unavailable",
                            "Please try again.",
                            Some("/admin"),
                        )}
                        div { class: "admin-metrics",
                            div { class: "metric",
                                span { class: "metric-label", "Total modules" }
                                strong { "{total_modules}" }
                            }
                            div { class: "metric",
                                span { class: "metric-label", "Total users" }
                                strong { "{total_users}" }
                            }
                            div { class: "metric",
                                span { class: "metric-label", "Pending submissions" }
                                strong { "{pending_count}" }
                            }
                        }
                        div { class: "admin-actions",
                            a { class: "ghost-button", href: "/modules", "Review catalog" }
                            a { class: "ghost-button", href: "/settings/security", "Security settings" }
                        }
                    }
                }
                section { class: "admin-panel admin-queue tui-panel",
                    div { class: "tui-panel-header",
                        span { class: "tui-panel-title", "[ PENDING SUBMISSIONS ]" }
                    }
                    div { class: "tui-panel-body",
                        h2 { "Pending submissions" }
                        {status_banner_with_retry(
                            submissions_state,
                            "Loading submissions...",
                            "Submissions unavailable",
                            "Please try again.",
                            Some("/admin"),
                        )}
                        if matches!(submissions_state, RemoteState::Ready(_)) {
                            if submissions.submissions.is_empty() {
                                p { class: "admin-empty", "No pending submissions" }
                            } else {
                                ul { class: "admin-submissions",
                                    for submission in submissions.submissions {
                                        AdminSubmissionCard { submission }
                                    }
                                }
                            }
                        }
                        if show_empty && !matches!(submissions_state, RemoteState::Ready(_)) {
                            p { class: "admin-empty", "No pending submissions" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn BarforgeRoute() -> Element {
    #[derive(Clone, Copy)]
    enum StatIcon {
        Star,
        Fork,
        Users,
    }

    #[derive(Clone, Copy)]
    enum TechIcon {
        Svelte,
        TypeScript,
        Cloudflare,
        Rust,
        Iced,
    }

    let stats_state = use_github_stats_state();
    let github_stats = github_stats_from_state(&stats_state);
    let stats = [
        ("Stars", github_stats.stars, StatIcon::Star),
        ("Forks", github_stats.forks, StatIcon::Fork),
        ("Contributors", github_stats.contributors, StatIcon::Users),
    ];

    let repos = [
        (
            "barforge-web",
            "Barforge Hub - browse and discover modules",
            "https://github.com/jtaw5649/barforge-web",
            vec![TechIcon::Svelte, TechIcon::TypeScript, TechIcon::Cloudflare],
        ),
        (
            "barforge-app",
            "Barforge App - install and configure modules",
            "https://github.com/jtaw5649/barforge-app",
            vec![TechIcon::Rust, TechIcon::Iced],
        ),
    ];

    let tech_stack = [
        ("Svelte", TechIcon::Svelte),
        ("TypeScript", TechIcon::TypeScript),
        ("Cloudflare", TechIcon::Cloudflare),
        ("Rust", TechIcon::Rust),
        ("Iced", TechIcon::Iced),
    ];

    let stat_icon = |icon: StatIcon| -> Element {
        match icon {
            StatIcon::Star => rsx!(svg {
                class: "stat-icon-svg",
                width: "20",
                height: "20",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                path { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" }
            }),
            StatIcon::Fork => rsx!(svg {
                class: "stat-icon-svg",
                width: "20",
                height: "20",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                circle { cx: "12", cy: "18", r: "3" }
                circle { cx: "6", cy: "6", r: "3" }
                circle { cx: "18", cy: "6", r: "3" }
                path { d: "M18 9v2c0 .6-.4 1-1 1H7c-.6 0-1-.4-1-1V9" }
                path { d: "M12 12v3" }
            }),
            StatIcon::Users => rsx!(svg {
                class: "stat-icon-svg",
                width: "20",
                height: "20",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                path { d: "M16 3.128a4 4 0 0 1 0 7.744" }
                path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                circle { cx: "9", cy: "7", r: "4" }
            }),
        }
    };

    let tech_icon = |icon: TechIcon, class: &'static str, size: &'static str| -> Element {
        match icon {
            TechIcon::Svelte => rsx!(svg {
                class: "{class}",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                path { d: "M10.354 21.125a4.44 4.44 0 0 1-4.765-1.767 4.109 4.109 0 0 1-.703-3.107 3.898 3.898 0 0 1 .134-.522l.105-.321.287.21a7.21 7.21 0 0 0 2.186 1.092l.208.063-.02.208a1.253 1.253 0 0 0 .226.83 1.337 1.337 0 0 0 1.435.533 1.231 1.231 0 0 0 .343-.15l5.59-3.562a1.164 1.164 0 0 0 .524-.778 1.242 1.242 0 0 0-.211-.937 1.338 1.338 0 0 0-1.435-.533 1.23 1.23 0 0 0-.343.15l-2.133 1.36a4.078 4.078 0 0 1-1.135.499 4.44 4.44 0 0 1-4.765-1.766 4.108 4.108 0 0 1-.702-3.108 3.855 3.855 0 0 1 1.742-2.582l5.589-3.563a4.072 4.072 0 0 1 1.135-.499 4.44 4.44 0 0 1 4.765 1.767 4.109 4.109 0 0 1 .703 3.107 3.943 3.943 0 0 1-.134.522l-.105.321-.286-.21a7.204 7.204 0 0 0-2.187-1.093l-.208-.063.02-.207a1.255 1.255 0 0 0-.226-.831 1.337 1.337 0 0 0-1.435-.532 1.231 1.231 0 0 0-.343.15L8.62 9.368a1.162 1.162 0 0 0-.524.778 1.24 1.24 0 0 0 .211.937 1.338 1.338 0 0 0 1.435.533 1.235 1.235 0 0 0 .344-.151l2.132-1.36a4.067 4.067 0 0 1 1.135-.498 4.44 4.44 0 0 1 4.765 1.766 4.108 4.108 0 0 1 .702 3.108 3.857 3.857 0 0 1-1.742 2.583l-5.589 3.562a4.072 4.072 0 0 1-1.135.499m10.358-17.95C18.484-.015 14.082-.96 10.9 1.068L5.31 4.63a6.412 6.412 0 0 0-2.896 4.295 6.753 6.753 0 0 0 .666 4.336 6.43 6.43 0 0 0-.96 2.396 6.833 6.833 0 0 0 1.168 5.167c2.229 3.19 6.63 4.135 9.812 2.108l5.59-3.562a6.41 6.41 0 0 0 2.896-4.295 6.756 6.756 0 0 0-.665-4.336 6.429 6.429 0 0 0 .958-2.396 6.831 6.831 0 0 0-1.167-5.168Z" }
            }),
            TechIcon::TypeScript => rsx!(svg {
                class: "{class}",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                path { d: "M1.125 0C.502 0 0 .502 0 1.125v21.75C0 23.498.502 24 1.125 24h21.75c.623 0 1.125-.502 1.125-1.125V1.125C24 .502 23.498 0 22.875 0zm17.363 9.75c.612 0 1.154.037 1.627.111a6.38 6.38 0 0 1 1.306.34v2.458a3.95 3.95 0 0 0-.643-.361 5.093 5.093 0 0 0-.717-.26 5.453 5.453 0 0 0-1.426-.2c-.3 0-.573.028-.819.086a2.1 2.1 0 0 0-.623.242c-.17.104-.3.229-.393.374a.888.888 0 0 0-.14.49c0 .196.053.373.156.529.104.156.252.304.443.444s.423.276.696.41c.273.135.582.274.926.416.47.197.892.407 1.266.628.374.222.695.473.963.753.268.279.472.598.614.957.142.359.214.776.214 1.253 0 .657-.125 1.21-.373 1.656a3.033 3.033 0 0 1-1.012 1.085 4.38 4.38 0 0 1-1.487.596c-.566.12-1.163.18-1.79.18a9.916 9.916 0 0 1-1.84-.164 5.544 5.544 0 0 1-1.512-.493v-2.63a5.033 5.033 0 0 0 3.237 1.2c.333 0 .624-.03.872-.09.249-.06.456-.144.623-.25.166-.108.29-.234.373-.38a1.023 1.023 0 0 0-.074-1.089 2.12 2.12 0 0 0-.537-.5 5.597 5.597 0 0 0-.807-.444 27.72 27.72 0 0 0-1.007-.436c-.918-.383-1.602-.852-2.053-1.405-.45-.553-.676-1.222-.676-2.005 0-.614.123-1.141.369-1.582.246-.441.58-.804 1.004-1.089a4.494 4.494 0 0 1 1.47-.629 7.536 7.536 0 0 1 1.77-.201zm-15.113.188h9.563v2.166H9.506v9.646H6.789v-9.646H3.375z" }
            }),
            TechIcon::Cloudflare => rsx!(svg {
                class: "{class}",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                path { d: "M16.5088 16.8447c.1475-.5068.0908-.9707-.1553-1.3154-.2246-.3164-.6045-.499-1.0615-.5205l-8.6592-.1123a.1559.1559 0 0 1-.1333-.0713c-.0283-.042-.0351-.0986-.021-.1553.0278-.084.1123-.1484.2036-.1562l8.7359-.1123c1.0351-.0489 2.1601-.8868 2.5537-1.9136l.499-1.3013c.0215-.0561.0293-.1128.0147-.168-.5625-2.5463-2.835-4.4453-5.5499-4.4453-2.5039 0-4.6284 1.6177-5.3876 3.8614-.4927-.3658-1.1187-.5625-1.794-.499-1.2026.119-2.1665 1.083-2.2861 2.2856-.0283.31-.0069.6128.0635.894C1.5683 13.171 0 14.7754 0 16.752c0 .1748.0142.3515.0352.5273.0141.083.0844.1475.1689.1475h15.9814c.0909 0 .1758-.0645.2032-.1553l.12-.4268zm2.7568-5.5634c-.0771 0-.1611 0-.2383.0112-.0566 0-.1054.0415-.127.0976l-.3378 1.1744c-.1475.5068-.0918.9707.1543 1.3164.2256.3164.6055.498 1.0625.5195l1.8437.1133c.0557 0 .1055.0263.1329.0703.0283.043.0351.1074.0214.1562-.0283.084-.1132.1485-.204.1553l-1.921.1123c-1.041.0488-2.1582.8867-2.5527 1.914l-.1406.3585c-.0283.0713.0215.1416.0986.1416h6.5977c.0771 0 .1474-.0489.169-.126.1122-.4082.1757-.837.1757-1.2803 0-2.6025-2.125-4.727-4.7344-4.727" }
            }),
            TechIcon::Rust => rsx!(svg {
                class: "{class}",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "currentColor",
                "aria-hidden": "true",
                path { d: "M23.8346 11.7033l-1.0073-.6236a13.7268 13.7268 0 00-.0283-.2936l.8656-.8069a.3483.3483 0 00-.1154-.578l-1.1066-.414a8.4958 8.4958 0 00-.087-.2856l.6904-.9587a.3462.3462 0 00-.2257-.5446l-1.1663-.1894a9.3574 9.3574 0 00-.1407-.2622l.49-1.0761a.3437.3437 0 00-.0274-.3361.3486.3486 0 00-.3006-.154l-1.1845.0416a6.7444 6.7444 0 00-.1873-.2268l.2723-1.153a.3472.3472 0 00-.417-.4172l-1.1532.2724a14.0183 14.0183 0 00-.2278-.1873l.0415-1.1845a.3442.3442 0 00-.49-.328l-1.076.491c-.0872-.0476-.1742-.0952-.2623-.1407l-.1903-1.1673A.3483.3483 0 0016.256.955l-.9597.6905a8.4867 8.4867 0 00-.2855-.086l-.414-1.1066a.3483.3483 0 00-.5781-.1154l-.8069.8666a9.2936 9.2936 0 00-.2936-.0284L12.2946.1683a.3462.3462 0 00-.5892 0l-.6236 1.0073a13.7383 13.7383 0 00-.2936.0284L9.9803.3374a.3462.3462 0 00-.578.1154l-.4141 1.1065c-.0962.0274-.1903.0567-.2855.086L7.744.955a.3483.3483 0 00-.5447.2258L7.009 2.348a9.3574 9.3574 0 00-.2622.1407l-1.0762-.491a.3462.3462 0 00-.49.328l.0416 1.1845a7.9826 7.9826 0 00-.2278.1873L3.8413 3.425a.3472.3472 0 00-.4171.4171l.2713 1.1531c-.0628.075-.1255.1509-.1863.2268l-1.1845-.0415a.3462.3462 0 00-.328.49l.491 1.0761a9.167 9.167 0 00-.1407.2622l-1.1662.1894a.3483.3483 0 00-.2258.5446l.6904.9587a13.303 13.303 0 00-.087.2855l-1.1065.414a.3483.3483 0 00-.1155.5781l.8656.807a9.2936 9.2936 0 00-.0283.2935l-1.0073.6236a.3442.3442 0 000 .5892l1.0073.6236c.008.0982.0182.1964.0283.2936l-.8656.8079a.3462.3462 0 00.1155.578l1.1065.4141c.0273.0962.0567.1914.087.2855l-.6904.9587a.3452.3452 0 00.2268.5447l1.1662.1893c.0456.088.0922.1751.1408.2622l-.491 1.0762a.3462.3462 0 00.328.49l1.1834-.0415c.0618.0769.1235.1528.1873.2277l-.2713 1.1541a.3462.3462 0 00.4171.4161l1.153-.2713c.075.0638.151.1255.2279.1863l-.0415 1.1845a.3442.3442 0 00.49.327l1.0761-.49c.087.0486.1741.0951.2622.1407l.1903 1.1662a.3483.3483 0 00.5447.2268l.9587-.6904a9.299 9.299 0 00.2855.087l.414 1.1066a.3452.3452 0 00.5781.1154l.8079-.8656c.0972.0111.1954.0203.2936.0294l.6236 1.0073a.3472.3472 0 00.5892 0l.6236-1.0073c.0982-.0091.1964-.0183.2936-.0294l.8069.8656a.3483.3483 0 00.578-.1154l.4141-1.1066a8.4626 8.4626 0 00.2855-.087l.9587.6904a.3452.3452 0 00.5447-.2268l.1903-1.1662c.088-.0456.1751-.0931.2622-.1407l1.0762.49a.3472.3472 0 00.49-.327l-.0415-1.1845a6.7267 6.7267 0 00.2267-.1863l1.1531.2713a.3472.3472 0 00.4171-.416l-.2713-1.1542c.0628-.0749.1255-.1508.1863-.2278l1.1845.0415a.3442.3442 0 00.328-.49l-.49-1.076c.0475-.0872.0951-.1742.1407-.2623l1.1662-.1893a.3483.3483 0 00.2258-.5447l-.6904-.9587.087-.2855 1.1066-.414a.3462.3462 0 00.1154-.5781l-.8656-.8079c.0101-.0972.0202-.1954.0283-.2936l1.0073-.6236a.3442.3442 0 000-.5892zm-6.7413 8.3551a.7138.7138 0 01.2986-1.396.714.714 0 11-.2997 1.396zm-.3422-2.3142a.649.649 0 00-.7715.5l-.3573 1.6685c-1.1035.501-2.3285.7795-3.6193.7795a8.7368 8.7368 0 01-3.6951-.814l-.3574-1.6684a.648.648 0 00-.7714-.499l-1.473.3158a8.7216 8.7216 0 01-.7613-.898h7.1676c.081 0 .1356-.0141.1356-.088v-2.536c0-.074-.0536-.0881-.1356-.0881h-2.0966v-1.6077h2.2677c.2065 0 1.1065.0587 1.394 1.2088.0901.3533.2875 1.5044.4232 1.8729.1346.413.6833 1.2381 1.2685 1.2381h3.5716a.7492.7492 0 00.1296-.0131 8.7874 8.7874 0 01-.8119.9526zM6.8369 20.024a.714.714 0 11-.2997-1.396.714.714 0 01.2997 1.396zM4.1177 8.9972a.7137.7137 0 11-1.304.5791.7137.7137 0 011.304-.579zm-.8352 1.9813l1.5347-.6824a.65.65 0 00.33-.8585l-.3158-.7147h1.2432v5.6025H3.5669a8.7753 8.7753 0 01-.2834-3.348zm6.7343-.5437V8.7836h2.9601c.153 0 1.0792.1772 1.0792.8697 0 .575-.7107.7815-1.2948.7815zm10.7574 1.4862c0 .2187-.008.4363-.0243.651h-.9c-.09 0-.1265.0586-.1265.1477v.413c0 .973-.5487 1.1846-1.0296 1.2382-.4576.0517-.9648-.1913-1.0275-.4717-.2704-1.5186-.7198-1.8436-1.4305-2.4034.8817-.5599 1.799-1.386 1.799-2.4915 0-1.1936-.819-1.9458-1.3769-2.3153-.7825-.5163-1.6491-.6195-1.883-.6195H5.4682a8.7651 8.7651 0 014.907-2.7699l1.0974 1.151a.648.648 0 00.9182.0213l1.227-1.1743a8.7753 8.7753 0 016.0044 4.2762l-.8403 1.8982a.652.652 0 00.33.8585l1.6178.7188c.0283.2875.0425.577.0425.8717zm-9.3006-9.5993a.7128.7128 0 11.984 1.0316.7137.7137 0 01-.984-1.0316zm8.3389 6.71a.7107.7107 0 01.9395-.3625.7137.7137 0 11-.9405.3635z" }
            }),
            TechIcon::Iced => rsx!(svg {
                class: "{class}",
                width: "{size}",
                height: "{size}",
                view_box: "35 20 185 195",
                fill: "currentColor",
                "aria-hidden": "true",
                path { d: "m182.62 65.747-28.136 28.606-6.13-6.0291 28.136-28.606 6.13 6.0291zm-26.344 0.218-42.204 42.909-6.13-6.029 42.204-42.909 6.13 6.0291zm-61.648 23.913c5.3254-5.3831 10.65-10.765 21.569-21.867l6.13 6.0291c-10.927 11.11-16.258 16.498-21.587 21.885-4.4007 4.4488-8.8009 8.8968-16.359 16.573l31.977 8.358 25.968-26.402 6.13 6.0292-25.968 26.402 8.907 31.908 42.138-42.087 6.076 6.083-49.109 49.05-45.837-12.628-13.394-45.646 1.7714-1.801c10.928-11.111 16.258-16.499 21.588-21.886zm28.419 70.99-8.846-31.689-31.831-8.32 9.1945 31.335 31.482 8.674zm47.734-56.517 7.122-7.1221-6.08-6.0797-7.147 7.1474-30.171 30.674 6.13 6.029 30.146-30.649z" }
            }),
        }
    };

    let repo_icon = || -> Element {
        rsx!(svg {
            class: "repo-icon-svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path { d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" }
        })
    };

    let repo_arrow = || -> Element {
        rsx!(svg {
            class: "repo-arrow-icon",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 3h6v6" }
            path { d: "M10 14 21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        })
    };

    rsx! {
        section { class: "barforge-page",
            a { class: "back-link", href: "/", "Back" }
            section { class: "intro-section",
                h1 { "Barforge Ecosystem" }
                p { "A complete toolkit for discovering, installing, and managing Waybar modules." }
            }
            section { class: "stats-section",
                {stats.into_iter().map(|(label, value, icon)| {
                    let key = format!("stat-{label}");
                    let icon_svg = stat_icon(icon);
                    rsx! {
                        div { class: "stat-box", key: "{key}",
                            div { class: "stat-icon",
                                {icon_svg}
                            }
                            div { class: "stat-content",
                                span { class: "stat-value", "{value}" }
                                span { class: "stat-label", "{label}" }
                            }
                        }
                    }
                })}
            }
            section { class: "tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ REPOSITORIES ]" }
                }
                div { class: "tui-panel-body repo-grid",
                    {repos.into_iter().map(|(name, desc, url, tech_icons)| {
                        let key = format!("repo-{name}");
                        rsx! {
                            a {
                                class: "repo-card",
                                href: "{url}",
                                target: "_blank",
                                rel: "noopener",
                                key: "{key}",
                                div { class: "repo-icon",
                                    {repo_icon()}
                                }
                                div { class: "repo-content",
                                    span { class: "repo-name", "{name}" }
                                    span { class: "repo-desc", "{desc}" }
                                    div { class: "repo-tech-icons",
                                        {tech_icons.into_iter().enumerate().map(|(index, icon)| {
                                            let icon_key = format!("{name}-tech-{index}");
                                            rsx!(span { class: "repo-tech-icon", key: "{icon_key}", {tech_icon(icon, "tech-icon-svg", "14")} })
                                        })}
                                    }
                                }
                                {repo_arrow()}
                            }
                        }
                    })}
                }
            }
            section { class: "tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ TECH STACK ]" }
                }
                div { class: "tui-panel-body tech-grid",
                    {tech_stack.into_iter().map(|(name, icon)| {
                        let key = format!("tech-{name}");
                        rsx! {
                            div { class: "tech-item", key: "{key}",
                                span { class: "tech-icon",
                                    {tech_icon(icon, "tech-icon-svg", "16")}
                                }
                                span { class: "tech-name", "{name}" }
                            }
                        }
                    })}
                }
            }
        }
    }
}

#[component]
pub(crate) fn TermsRoute() -> Element {
    rsx! {
        document::Title { "Terms of Service · Barforge" }
        document::Meta { name: "description", content: "Terms of Service for Barforge, the Waybar module registry and ecosystem." }
        section { class: "legal-page",
            a { class: "back-link", href: "/", aria_label: "Back to homepage",
                svg { xmlns: "http://www.w3.org/2000/svg", width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    path { d: "m12 19-7-7 7-7" }
                    path { d: "M19 12H5" }
                }
                span { "Back" }
            }
            article { class: "tui-panel",
                header { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ TERMS ]" }
                    span { class: "tui-panel-status", "Effective: December 27, 2025" }
                }
                div { class: "tui-panel-body",
                    h1 { "Terms of Service" }
                    section {
                        h2 { "1. About Barforge" }
                        p { "Barforge is a community registry for Waybar modules, plus related services like the Barforge desktop app and the Barforge registry API. These Terms govern your access to the Barforge website, API endpoints, and any linked tooling." }
                    }
                    section {
                        h2 { "2. Eligibility and Accounts" }
                        p { "You must be legally able to enter into these Terms. Authentication is provided via GitHub OAuth; your GitHub account remains subject to GitHub's own terms and policies. You are responsible for activity that occurs under your account." }
                    }
                    section {
                        h2 { "3. Module Submissions" }
                        p { "When you submit modules or module metadata, you keep ownership of your code. You grant Barforge a non-exclusive, worldwide, royalty-free license to host, cache, display, distribute, and make reasonable modifications to submission data needed to operate the registry and related services, such as indexing and compatibility or security updates." }
                        p { "You must include a valid SPDX license identifier and a license file or declaration with your submission. We may reject or remove submissions without an explicit license." }
                        p { "If you request account deletion, we may continue to host public submissions to preserve registry history. We will remove or anonymize personal attribution where feasible." }
                        p { "You confirm that you have the right to submit the content, and that your submission does not infringe third-party rights or include malicious code." }
                    }
                    section {
                        h2 { "4. Reviews and Community Content" }
                        p { "Reviews, ratings, and comments are user content. You are responsible for what you post and agree to keep contributions accurate, respectful, and relevant to the modules or services being discussed." }
                    }
                    section {
                        h2 { "5. Acceptable Use" }
                        p { "You agree not to:" }
                        ul {
                            li { "Upload malware, exploit code, or content designed to harm users or systems." }
                            li { "Impersonate others or misrepresent authorship of modules or reviews." }
                            li { "Spam, scrape, or abuse API endpoints beyond reasonable usage." }
                            li { "Violate applicable laws or third-party policies." }
                        }
                    }
                    section {
                        h2 { "6. Intellectual Property and Licensing" }
                        p { "Modules remain licensed by their authors. You must include an appropriate license with your module. Licenses granted to Barforge and end users continue even after account deletion. Barforge branding and UI elements are owned by their respective rights holders and may not be reused without permission." }
                    }
                    section {
                        h2 { "7. Availability and Changes" }
                        p { "Barforge may update, suspend, or discontinue features at any time. We may remove content that violates these Terms or poses security or legal risk." }
                    }
                    section {
                        h2 { "8. Disclaimers" }
                        p { "Barforge is provided \"as is\" without warranties of any kind. Use of modules is at your own risk. We do not guarantee availability, security, or accuracy of registry content." }
                    }
                    section {
                        h2 { "9. Limitation of Liability" }
                        p { "To the maximum extent allowed by law, Barforge and its contributors are not liable for indirect, incidental, or consequential damages arising from your use of the services or modules." }
                    }
                    section {
                        h2 { "10. Termination" }
                        p { "We may suspend or terminate access if you violate these Terms. You may stop using Barforge at any time." }
                    }
                    section {
                        h2 { "11. Contact" }
                        p {
                            "Questions about these Terms can be sent to "
                            a { href: "mailto:support@barforge.dev", "support@barforge.dev" }
                            "."
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn PrivacyRoute() -> Element {
    rsx! {
        document::Title { "Privacy Policy · Barforge" }
        document::Meta { name: "description", content: "Privacy Policy for Barforge, the Waybar module registry and ecosystem." }
        section { class: "legal-page",
            a { class: "back-link", href: "/", aria_label: "Back to homepage",
                svg { xmlns: "http://www.w3.org/2000/svg", width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    path { d: "m12 19-7-7 7-7" }
                    path { d: "M19 12H5" }
                }
                span { "Back" }
            }
            article { class: "tui-panel",
                header { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ PRIVACY ]" }
                    span { class: "tui-panel-status", "Effective: December 27, 2025" }
                }
                div { class: "tui-panel-body",
                    h1 { "Privacy Policy" }
                    section {
                        h2 { "1. Information We Collect" }
                        p { "When you sign in with GitHub OAuth, we receive basic profile information such as your username, display name, avatar, and email address (if provided by GitHub). We also collect the content you submit to the registry, including module metadata, reviews, ratings, and other community contributions." }
                        p { "We may collect limited technical data (such as IP address and user agent) for security, abuse prevention, and operational reliability." }
                    }
                    section {
                        h2 { "2. How We Use Data" }
                        p { "We use your data to authenticate your account, attribute modules and reviews, operate the Barforge registry API, and improve the quality of the service." }
                    }
                    section {
                        h2 { "3. Public Information" }
                        p { "Module listings, reviews, and ratings are public. Your GitHub username and avatar are displayed alongside public contributions so the community can attribute work." }
                    }
                    section {
                        h2 { "4. Session Storage and Preferences" }
                        p { "We use essential cookies for session management. Barforge may also use local storage for preferences such as starred modules or UI settings." }
                    }
                    section {
                        h2 { "5. Data Sharing" }
                        p { "We do not sell personal data. Data is processed by infrastructure providers such as Cloudflare for hosting and delivery. GitHub provides authentication services." }
                    }
                    section {
                        h2 { "6. Data Retention" }
                        p { "We keep account data while your account is active. Public contributions may remain visible to preserve registry history. If you request account deletion, we will remove or anonymize personal data where feasible, but public contributions may remain available. You can request deletion of your account or content, and we will process reasonable requests subject to legal obligations." }
                    }
                    section {
                        h2 { "7. Your Rights" }
                        p {
                            "You may request access, correction, or deletion of your data. To exercise your rights, email "
                            a { href: "mailto:support@barforge.dev", "support@barforge.dev" }
                            "."
                        }
                        p {
                            "For general support, contact "
                            a { href: "mailto:help@barforge.dev", "help@barforge.dev" }
                            ". Administrative requests can be sent to "
                            a { href: "mailto:admin@barforge.dev", "admin@barforge.dev" }
                            "."
                        }
                    }
                    section {
                        h2 { "8. Changes to This Policy" }
                        p { "We may update this policy as the Barforge ecosystem evolves. The effective date will reflect the latest revision." }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn NotFoundRoute(segments: Vec<String>) -> Element {
    FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);
    let path = format!("/{}", segments.join("/"));

    rsx! {
        section { class: "not-found-route",
            div { class: "not-found-panel tui-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ NOT FOUND ]" }
                }
                div { class: "tui-panel-body",
                    h1 { "Page not found" }
                    p { "We couldn't find {path}." }
                    a { class: "ghost-button", href: "/", "Back to Home" }
                }
            }
        }
    }
}

#[cfg(test)]
mod landing_tests {
    use super::{RemoteState, landing_status_banner};
    use barforge_types::LandingData;

    #[test]
    fn landing_status_banner_renders_error() {
        let state: RemoteState<LandingData> = RemoteState::Error("no landing".to_string());

        let html = dioxus_ssr::render_element(landing_status_banner(&state, "/"));

        assert!(html.contains("Landing unavailable"));
        assert!(html.contains("Please try again."));
        assert!(html.contains("Retry"));
    }
}

#[cfg(test)]
mod collection_detail_tests {
    use super::collection_detail_body;
    use barforge_types::{Collection, CollectionModule, CollectionOwner, ModuleCategory};

    fn sample_collection() -> Collection {
        Collection {
            id: 1,
            user_id: 7,
            name: "Ops Essentials".to_string(),
            description: Some("Modules for ops".to_string()),
            visibility: "public".to_string(),
            module_count: 2,
            owner: CollectionOwner {
                username: "barforge".to_string(),
                display_name: Some("Barforge".to_string()),
                avatar_url: None,
            },
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-10T00:00:00Z".to_string(),
        }
    }

    fn sample_module(uuid: &str, note: Option<&str>) -> CollectionModule {
        CollectionModule {
            uuid: uuid.to_string(),
            name: "Weather".to_string(),
            author: "barforge".to_string(),
            category: ModuleCategory::Weather,
            note: note.map(|value| value.to_string()),
            position: 1,
            added_at: "2024-01-02T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn collection_detail_body_renders_empty_state_for_owner() {
        let html = dioxus_ssr::render_element(collection_detail_body(
            sample_collection(),
            Vec::new(),
            true,
            None,
            None,
            None,
        ));

        assert!(html.contains("This collection is empty"));
        assert!(html.contains("Add modules to this collection"));
    }

    #[test]
    fn collection_detail_body_renders_remove_button_for_owner() {
        let html = dioxus_ssr::render_element(collection_detail_body(
            sample_collection(),
            vec![sample_module(
                "weather-wttr@barforge",
                Some("Pin for alerts"),
            )],
            true,
            None,
            None,
            None,
        ));

        assert!(html.contains("Remove"));
        assert!(html.contains("Pin for alerts"));
    }
}

#[cfg(test)]
mod admin_tests {
    use super::{RemoteState, admin_route_body};
    use barforge_types::{AdminStats, Submission, SubmissionsResponse};

    fn sample_admin_stats() -> AdminStats {
        AdminStats {
            total_modules: 1200,
            total_users: 42_000,
            total_downloads: 9001,
            pending_submissions: 1,
        }
    }

    fn sample_submission() -> Submission {
        Submission {
            id: 1,
            submitter_id: 7,
            uuid: "submission-1".to_string(),
            name: "Weather".to_string(),
            description: "Forecasts".to_string(),
            category: "system".to_string(),
            version: "1.0.0".to_string(),
            repo_url: "https://example.com/repo".to_string(),
            status: "pending".to_string(),
            rejection_reason: None,
            submitted_at: "2024-01-01T00:00:00Z".to_string(),
            reviewed_at: None,
            reviewed_by: None,
            submitter_username: "octo".to_string(),
        }
    }

    #[test]
    fn admin_route_body_renders_stats_and_submissions() {
        let stats_state = RemoteState::Ready(sample_admin_stats());
        let submissions_state = RemoteState::Ready(SubmissionsResponse {
            submissions: vec![sample_submission()],
            total: 1,
        });

        let html = dioxus_ssr::render_element(admin_route_body(&stats_state, &submissions_state));

        assert!(html.contains("1,200"));
        assert!(html.contains("42,000"));
        assert!(html.contains("Pending submissions"));
        assert!(html.contains("Weather"));
        assert!(html.contains("octo"));
    }

    #[test]
    fn admin_route_body_renders_action_buttons() {
        let stats_state = RemoteState::Ready(sample_admin_stats());
        let submissions_state = RemoteState::Ready(SubmissionsResponse {
            submissions: vec![sample_submission()],
            total: 1,
        });

        let html = dioxus_ssr::render_element(admin_route_body(&stats_state, &submissions_state));

        assert!(html.contains("Approve"));
        assert!(html.contains("Reject"));
        assert!(html.contains("Toggle verify"));
    }

    #[test]
    fn admin_route_body_renders_empty_state_when_no_submissions() {
        let stats_state = RemoteState::Ready(sample_admin_stats());
        let submissions_state = RemoteState::Ready(SubmissionsResponse {
            submissions: Vec::new(),
            total: 0,
        });

        let html = dioxus_ssr::render_element(admin_route_body(&stats_state, &submissions_state));

        assert!(html.contains("No pending submissions"));
    }

    #[test]
    fn admin_route_body_renders_error_banner_for_stats() {
        let stats_state: RemoteState<AdminStats> = RemoteState::Error("stats failed".to_string());
        let submissions_state = RemoteState::Unavailable;

        let html = dioxus_ssr::render_element(admin_route_body(&stats_state, &submissions_state));

        assert!(html.contains("Admin stats unavailable"));
        assert!(html.contains("Please try again."));
    }
}

#[cfg(test)]
mod settings_security_tests {
    use super::*;

    #[test]
    fn settings_security_renders_export_link() {
        let html = dioxus_ssr::render_element(rsx!(SettingsSecurity {}));

        assert!(html.contains("/api/users/me/export"));
        assert!(html.contains("Export Data"));
    }

    #[test]
    fn settings_security_renders_delete_confirmation() {
        let html = dioxus_ssr::render_element(rsx!(SettingsSecurity {}));

        assert!(html.contains("Delete Account"));
        assert!(html.contains("Type your username"));
    }
}

fn format_number(value: i64) -> String {
    let negative = value.is_negative();
    let digits = value.abs().to_string().chars().rev().collect::<Vec<_>>();
    let mut formatted = String::new();
    for (index, digit) in digits.iter().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(*digit);
    }
    let mut formatted: String = formatted.chars().rev().collect();
    if negative {
        formatted.insert(0, '-');
    }
    formatted
}

fn parse_sort(sort: Option<String>) -> ModuleSort {
    match sort.as_deref() {
        Some("downloads") => ModuleSort::Downloads,
        Some("trending") => ModuleSort::Trending,
        Some("recent") => ModuleSort::Recent,
        Some("alpha") => ModuleSort::Alphabetical,
        _ => ModuleSort::Popular,
    }
}

fn parse_view(view: Option<String>) -> ModuleViewMode {
    match view.as_deref() {
        Some("list") => ModuleViewMode::List,
        _ => ModuleViewMode::Grid,
    }
}

fn parse_category(category: Option<String>) -> Option<ModuleCategory> {
    match category.as_deref() {
        Some("system") => Some(ModuleCategory::System),
        Some("hardware") => Some(ModuleCategory::Hardware),
        Some("network") => Some(ModuleCategory::Network),
        Some("audio") => Some(ModuleCategory::Audio),
        Some("power") => Some(ModuleCategory::Power),
        Some("time") => Some(ModuleCategory::Time),
        Some("workspace") => Some(ModuleCategory::Workspace),
        Some("window") => Some(ModuleCategory::Window),
        Some("tray") => Some(ModuleCategory::Tray),
        Some("weather") => Some(ModuleCategory::Weather),
        Some("productivity") => Some(ModuleCategory::Productivity),
        Some("media") => Some(ModuleCategory::Media),
        Some("custom") => Some(ModuleCategory::Custom),
        _ => None,
    }
}

fn next_tab_index(current: usize, count: usize, key: &Key) -> Option<usize> {
    if count == 0 || current >= count {
        return None;
    }
    match key {
        Key::ArrowLeft => Some((current + count - 1) % count),
        Key::ArrowRight => Some((current + 1) % count),
        Key::Home => Some(0),
        Key::End => Some(count - 1),
        _ => None,
    }
}

fn normalize_username(value: Option<&str>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn build_screenshot_urls(base: &str, uuid: &str, screenshots: &[Screenshot]) -> Vec<String> {
    let base = base.trim_end_matches('/');
    screenshots
        .iter()
        .map(|screenshot| {
            let filename = screenshot_filename(&screenshot.r2_key);
            format!("{base}/screenshots/{uuid}/{filename}")
        })
        .collect()
}

fn screenshot_filename(r2_key: &str) -> &str {
    r2_key.rsplit('/').next().unwrap_or(r2_key)
}

fn recommended_modules(
    recently_viewed: &[RecentModule],
    popular: &[RegistryModule],
    recent_list: &[RegistryModule],
) -> Vec<RegistryModule> {
    if recently_viewed.is_empty() {
        return Vec::new();
    }
    let mut categories = Vec::new();
    let mut viewed_uuids = HashSet::new();
    for module in recently_viewed {
        if !categories.contains(&module.category) {
            categories.push(module.category.clone());
        }
        viewed_uuids.insert(module.uuid.clone());
    }

    let mut seen = HashSet::new();
    let mut combined = Vec::new();
    for module in popular.iter().chain(recent_list.iter()) {
        if seen.insert(module.uuid.clone()) {
            combined.push(module.clone());
        }
    }

    combined
        .into_iter()
        .filter(|module| {
            categories.contains(&module.category) && !viewed_uuids.contains(&module.uuid)
        })
        .take(6)
        .collect()
}

#[cfg(test)]
mod recommended_modules_tests {
    use super::{RecentModule, recommended_modules};
    use barforge_types::{ModuleCategory, RegistryModule};

    fn module(uuid: &str, category: ModuleCategory) -> RegistryModule {
        RegistryModule {
            uuid: uuid.to_string(),
            name: "Module".to_string(),
            description: "Description".to_string(),
            author: "Author".to_string(),
            category,
            icon: None,
            screenshot: None,
            repo_url: "https://example.com".to_string(),
            downloads: 0,
            version: None,
            last_updated: None,
            rating: None,
            verified_author: false,
            tags: Vec::new(),
            checksum: None,
            license: None,
        }
    }

    fn recent_module(uuid: &str, category: ModuleCategory) -> RecentModule {
        RecentModule {
            uuid: uuid.to_string(),
            name: "Module".to_string(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            category,
            downloads: 0,
            verified_author: false,
            version: None,
            viewed_at: 0,
        }
    }

    #[test]
    fn recommended_modules_empty_without_recently_viewed() {
        let recommended = recommended_modules(&[], &[], &[]);

        assert!(recommended.is_empty());
    }

    #[test]
    fn recommended_modules_filters_by_category_and_excludes_viewed() {
        let recent = vec![recent_module("alpha@barforge", ModuleCategory::System)];
        let popular = vec![
            module("alpha@barforge", ModuleCategory::System),
            module("beta@barforge", ModuleCategory::System),
        ];
        let recent_list = vec![
            module("beta@barforge", ModuleCategory::System),
            module("gamma@barforge", ModuleCategory::Weather),
        ];

        let recommended = recommended_modules(&recent, &popular, &recent_list);
        let uuids = recommended
            .iter()
            .map(|module| module.uuid.as_str())
            .collect::<Vec<_>>();

        assert_eq!(uuids, vec!["beta@barforge"]);
    }
}

#[cfg(test)]
mod tab_accessibility_tests {
    use super::{InstallSnippet, modules_tab_panels, next_tab_index};
    use dioxus::prelude::*;

    #[test]
    fn install_snippet_exposes_tab_roles_and_panels() {
        let mut dom = VirtualDom::new(InstallSnippet);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("role=\"tab\""));
        assert!(html.contains("role=\"tabpanel\""));
        assert!(html.contains("id=\"install-tab-1\""));
        assert!(html.contains("aria-selected=\"true\""));
        assert!(html.contains("aria-controls=\"install-panel-1\""));
        assert!(html.contains("id=\"install-panel-1\""));
    }

    #[test]
    fn modules_tabs_render_both_tabpanels() {
        for &is_search in [false, true].iter() {
            let html =
                dioxus_ssr::render_element(modules_tab_panels(is_search, rsx!(div { "Panel" })));

            assert!(html.contains("id=\"modules-discover-panel\""));
            assert!(html.contains("id=\"modules-search-panel\""));
            assert!(html.contains("aria-labelledby=\"modules-tab-discover\""));
            assert!(html.contains("aria-labelledby=\"modules-tab-search\""));
        }
    }

    #[test]
    fn next_tab_index_wraps_and_supports_home_end() {
        assert_eq!(next_tab_index(0, 3, &Key::ArrowRight), Some(1));
        assert_eq!(next_tab_index(2, 3, &Key::ArrowRight), Some(0));
        assert_eq!(next_tab_index(0, 3, &Key::ArrowLeft), Some(2));
        assert_eq!(next_tab_index(1, 3, &Key::Home), Some(0));
        assert_eq!(next_tab_index(1, 3, &Key::End), Some(2));
        assert_eq!(next_tab_index(1, 3, &Key::Enter), None);
        assert_eq!(next_tab_index(0, 0, &Key::ArrowRight), None);
    }
}
