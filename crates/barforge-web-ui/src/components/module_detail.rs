use super::module_card::{ModuleCard, ModuleCardSkeleton, format_downloads};
use super::profile_hover::ProfileHover;
use crate::categories::{category_color, category_label};
use barforge_types::{RegistryModule, Review, VersionHistoryEntry};
use dioxus::prelude::*;
use std::rc::Rc;
use time::OffsetDateTime;

#[derive(Props, Clone, PartialEq)]
pub struct ModuleDetailProps {
    pub module: RegistryModule,
    pub screenshots: Vec<String>,
    pub related_modules: Vec<RegistryModule>,
    pub related_loading: bool,
    pub versions: Vec<VersionHistoryEntry>,
    pub reviews: Vec<Review>,
    pub now: OffsetDateTime,
    #[props(optional)]
    pub install_copied: bool,
    #[props(optional)]
    pub on_copy_install: Option<EventHandler<()>>,
}

#[derive(Clone)]
struct ScreenshotItem {
    key: String,
    url: Option<String>,
    label: String,
    is_placeholder: bool,
}

#[allow(non_snake_case)]
pub fn ModuleDetail(
    ModuleDetailProps {
        module,
        screenshots,
        related_modules,
        related_loading,
        versions,
        reviews,
        now,
        install_copied,
        on_copy_install,
    }: ModuleDetailProps,
) -> Element {
    let RegistryModule {
        uuid,
        name,
        author,
        description,
        category,
        downloads,
        version,
        repo_url,
        icon,
        verified_author,
        ..
    } = module;
    let category_label = category_label(&category);
    let category_color = category_color(&category);
    let install_command = format!("barforge install {uuid}");
    let downloads_label = format_downloads(downloads);
    let icon_node = icon.as_ref().map(|icon| {
        rsx!(img {
            src: "{icon}",
            alt: "",
            width: "80",
            height: "80",
            loading: "lazy",
            decoding: "async",
        })
    });
    let initial = name
        .chars()
        .next()
        .map(|value| value.to_ascii_uppercase())
        .unwrap_or('M');
    let version_badge = version
        .as_ref()
        .map(|version| rsx!(span { class: "module-version", "v{version}" }));
    let verified_badge = verified_author.then(|| {
        rsx!(span { class: "verified-icon", "aria-label": "Verified author",
            svg {
                width: "14",
                height: "14",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                polyline { points: "20 6 9 17 4 12" }
            }
        })
    });

    let copy_icon = || {
        if install_copied {
            rsx!(svg {
                class: "copy-icon",
                width: "14",
                height: "14",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                path { d: "M20 6 9 17l-5-5" }
            })
        } else {
            rsx!(svg {
                class: "copy-icon",
                width: "14",
                height: "14",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                rect { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" }
                path { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }
            })
        }
    };
    let copy_label = if install_copied {
        "Copied"
    } else {
        "Copy install command"
    };
    let copy_text = if install_copied { "Copied!" } else { "Copy" };
    let copy_class = if install_copied {
        "copy-btn copied"
    } else {
        "copy-btn"
    };
    let copy_disabled = install_copied || on_copy_install.is_none();
    let copy_button = if let Some(handler) = on_copy_install.as_ref() {
        let handler = *handler;
        rsx!(button {
            class: "{copy_class}",
            r#type: "button",
            "aria-label": "{copy_label}",
            disabled: copy_disabled,
            onclick: move |_| handler.call(()),
            {copy_icon()}
            span { "{copy_text}" }
        })
    } else {
        rsx!(button {
            class: "{copy_class}",
            r#type: "button",
            "aria-label": "{copy_label}",
            disabled: true,
            {copy_icon()}
            span { "{copy_text}" }
        })
    };

    let placeholder_items = (0..3).map(|index| ScreenshotItem {
        key: format!("placeholder-{index}"),
        url: None,
        label: format!("Placeholder {}", index + 1),
        is_placeholder: true,
    });
    let screenshot_items = if screenshots.is_empty() {
        placeholder_items.collect::<Vec<_>>()
    } else {
        screenshots
            .into_iter()
            .enumerate()
            .map(|(index, url)| ScreenshotItem {
                key: url.clone(),
                url: Some(url),
                label: format!("{name} screenshot {}", index + 1),
                is_placeholder: false,
            })
            .collect()
    };
    let screenshot_items = Rc::new(screenshot_items);
    let total_screenshots = screenshot_items.len();
    let active_index = use_signal(|| None::<usize>);
    let active_index_value = active_index
        .read()
        .unwrap_or(0)
        .min(total_screenshots.saturating_sub(1));
    let carousel_open = active_index.read().is_some();
    let carousel_state = if carousel_open { "true" } else { "false" };
    let active_item = screenshot_items.get(active_index_value).cloned();
    let active_label = active_item
        .as_ref()
        .map(|item| item.label.clone())
        .unwrap_or_else(|| "Screenshot".to_string());
    let carousel_counter = if total_screenshots == 0 {
        "0 / 0".to_string()
    } else {
        format!("{} / {}", active_index_value + 1, total_screenshots)
    };
    let close_carousel = {
        let mut active_index = active_index;
        move |_| active_index.set(None)
    };
    let show_prev = total_screenshots > 1;
    let show_next = total_screenshots > 1;
    let go_prev = {
        let mut active_index = active_index;
        let total = total_screenshots;
        move |_| {
            if total == 0 {
                return;
            }
            let current = active_index.read().unwrap_or(0);
            let next = if current == 0 { total - 1 } else { current - 1 };
            active_index.set(Some(next));
        }
    };
    let go_next = {
        let mut active_index = active_index;
        let total = total_screenshots;
        move |_| {
            if total == 0 {
                return;
            }
            let current = active_index.read().unwrap_or(0);
            let next = if current + 1 >= total { 0 } else { current + 1 };
            active_index.set(Some(next));
        }
    };

    let screenshots_content = rsx!(div { class: "screenshots-grid",
        {screenshot_items.iter().enumerate().map(|(index, item)| {
            let key = item.key.clone();
            let label = item.label.clone();
            let url = item.url.clone();
            let placeholder_attr = if item.is_placeholder { "true" } else { "false" };
            let mut active_index = active_index;
            rsx!(button {
                class: "screenshot-tile",
                key: "{key}",
                r#type: "button",
                "data-screenshot-index": "{index}",
                "data-placeholder": "{placeholder_attr}",
                "aria-label": "Open screenshot {index + 1}",
                onclick: move |_| active_index.set(Some(index)),
                if let Some(url) = url {
                    img { src: "{url}", alt: "{label}", loading: "lazy" }
                } else {
                    div { class: "screenshot-placeholder",
                        span { class: "screenshot-watermark", "Placeholder" }
                    }
                }
            })
        })}
    });

    let active_view = if let Some(item) = active_item {
        if let Some(url) = item.url {
            rsx!(img {
                src: "{url}",
                alt: "{active_label}"
            })
        } else {
            rsx!(div { class: "screenshot-placeholder",
                span { class: "screenshot-watermark", "Placeholder" }
            })
        }
    } else {
        rsx!(div { class: "screenshot-placeholder",
            span { class: "screenshot-watermark", "Placeholder" }
        })
    };

    let related_content = if related_loading {
        rsx!(div { class: "related-grid",
            ModuleCardSkeleton {}
            ModuleCardSkeleton {}
            ModuleCardSkeleton {}
        })
    } else if related_modules.is_empty() {
        rsx!(p { class: "related-empty", "No related modules" })
    } else {
        rsx!(ul { class: "related-grid",
            {related_modules.into_iter().map(|module| {
                rsx!(ModuleCard {
                    module,
                    now,
                })
            })}
        })
    };

    let versions_content = if versions.is_empty() {
        rsx!(p { class: "versions-empty", "No versions yet" })
    } else {
        rsx!(ul { class: "versions-list",
            {versions.into_iter().map(|entry| {
                let version = entry.version;
                let changelog = entry
                    .changelog
                    .unwrap_or_else(|| "No changelog".to_string());
                rsx!(li { class: "version-entry", key: "{version}",
                    span { class: "version-number", "v{version}" }
                    p { class: "version-changelog", "{changelog}" }
                })
            })}
        })
    };

    let reviews_content = if reviews.is_empty() {
        rsx!(p { class: "reviews-empty", "No reviews yet" })
    } else {
        rsx!(ul { class: "reviews-list",
            {reviews.into_iter().map(|review| {
                let Review {
                    id,
                    rating,
                    title,
                    body,
                    helpful_count,
                    user,
                    ..
                } = review;
                let username = user.username;
                let avatar_url = user.avatar_url;
                let helpful = helpful_count;
                let helpful_label = if helpful == 1 {
                    "1 helpful".to_string()
                } else {
                    format!("{helpful} helpful")
                };
                let title_node = title
                    .as_ref()
                    .map(|title| rsx!(h3 { class: "review-title", "{title}" }));
                let body_node =
                    body.as_ref().map(|body| rsx!(p { class: "review-body", "{body}" }));
                rsx!(li { class: "review-entry", key: "{id}",
                    div { class: "review-meta",
                        span { class: "review-rating", "{rating}/5" }
                        ProfileHover {
                            username,
                            label: None,
                            avatar_url,
                            link_class: Some("review-user".to_string()),
                        }
                        span { class: "review-helpful", "{helpful_label}" }
                    }
                    {title_node}
                    {body_node}
                })
            })}
        })
    };

    rsx! {
        section { class: "module-detail module-detail-layout",
            a { class: "back-link module-detail-back", href: "/modules",
                svg {
                    class: "module-back-icon",
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
                span { "Back to modules" }
            }
            section { class: "tui-panel module-detail-panel",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ MODULE ]" }
                    span { class: "tui-panel-status", "{category_label}" }
                }
                div { class: "tui-panel-body",
                    div { class: "module-detail-header",
                        div { class: "module-detail-icon",
                            {if let Some(icon) = icon_node {
                                rsx! { {icon} }
                            } else {
                                rsx!(span { class: "module-detail-initial", "{initial}" })
                            }}
                        }
                        div { class: "module-detail-info",
                            div { class: "module-detail-title",
                                h1 { "{name}" }
                                {version_badge}
                            }
                            div { class: "module-detail-meta",
                                ProfileHover {
                                    username: author.clone(),
                                    label: Some(format!("by {author}")),
                                    avatar_url: None,
                                    link_class: Some("module-author".to_string()),
                                }
                                {verified_badge}
                                span {
                                    class: "module-tag tag-category",
                                    style: "--tag-color: {category_color};",
                                    "{category_label}"
                                }
                                div { class: "module-stats",
                                    svg {
                                        width: "14",
                                        height: "14",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                                        polyline { points: "7 10 12 15 17 10" }
                                        line { x1: "12", y1: "15", x2: "12", y2: "3" }
                                    }
                                    span { class: "module-downloads", "{downloads_label} downloads" }
                                }
                            }
                            p { class: "module-description-full", "{description}" }
                            div { class: "module-detail-actions",
                                div { class: "module-install",
                                    span { class: "module-install-label", "Install" }
                                    div { class: "module-install-row",
                                        code { class: "module-install-command", "{install_command}" }
                                        {copy_button}
                                    }
                                }
                                a {
                                    class: "module-repo",
                                    href: "{repo_url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    svg {
                                        class: "module-repo-icon",
                                        width: "16",
                                        height: "16",
                                        view_box: "0 0 24 24",
                                        fill: "currentColor",
                                        "aria-hidden": "true",
                                        path {
                                            d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                                        }
                                    }
                                    span { "View repository" }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "tui-panel module-screenshots",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ SCREENSHOTS ]" }
                }
                div { class: "tui-panel-body",
                    h2 { class: "sr-only", "Screenshots" }
                    {screenshots_content}
                }
            }
            section { class: "tui-panel module-versions",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ VERSION HISTORY ]" }
                }
                div { class: "tui-panel-body",
                    h2 { class: "sr-only", "Version history" }
                    {versions_content}
                }
            }
            section { class: "tui-panel module-reviews",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ REVIEWS ]" }
                }
                div { class: "tui-panel-body",
                    h2 { class: "sr-only", "Reviews" }
                    {reviews_content}
                }
            }
            section { class: "tui-panel module-related",
                div { class: "tui-panel-header",
                    span { class: "tui-panel-title", "[ RELATED MODULES ]" }
                }
                div { class: "tui-panel-body",
                    h2 { class: "sr-only", "Related modules" }
                    {related_content}
                }
            }
            div {
                class: "screenshot-carousel",
                "data-open": "{carousel_state}",
                aria_hidden: if carousel_open { "false" } else { "true" },
                button {
                    class: "screenshot-carousel-backdrop",
                    r#type: "button",
                    onclick: close_carousel,
                }
                div {
                    class: "screenshot-carousel-dialog",
                    role: "dialog",
                    "aria-modal": "true",
                    "aria-label": "Screenshot carousel",
                    header { class: "screenshot-carousel-header",
                        span { class: "screenshot-carousel-title", "{active_label}" }
                        span { class: "screenshot-carousel-count", "{carousel_counter}" }
                        button {
                            class: "carousel-close",
                            r#type: "button",
                            "aria-label": "Close carousel",
                            onclick: close_carousel,
                            svg {
                                class: "carousel-close-icon",
                                width: "14",
                                height: "14",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "M18 6 6 18" }
                                path { d: "m6 6 12 12" }
                            }
                        }
                    }
                    div { class: "screenshot-carousel-frame",
                        {active_view}
                    }
                    if show_prev {
                        button {
                            class: "carousel-nav carousel-prev",
                            r#type: "button",
                            "aria-label": "Previous screenshot",
                            onclick: go_prev,
                            svg {
                                class: "carousel-nav-icon",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                "aria-hidden": "true",
                                path { d: "m15 18-6-6 6-6" }
                            }
                        }
                    }
                    if show_next {
                        button {
                            class: "carousel-nav carousel-next",
                            r#type: "button",
                            "aria-label": "Next screenshot",
                            onclick: go_next,
                            svg {
                                class: "carousel-nav-icon",
                                width: "16",
                                height: "16",
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
                    }
                }
            }
        }
    }
}
