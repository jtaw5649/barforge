use super::profile_hover::ProfileHover;
use crate::categories::{category_color, category_label};
use barforge_types::RegistryModule;
use dioxus::prelude::*;
use time::{Duration, OffsetDateTime};

#[derive(Props, Clone, PartialEq)]
pub struct ModuleCardProps {
    pub module: RegistryModule,
    pub now: OffsetDateTime,
    #[props(optional)]
    pub is_starred: bool,
    #[props(optional)]
    pub on_toggle_star: Option<EventHandler<String>>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ModuleCardRowProps {
    pub module: RegistryModule,
    pub now: OffsetDateTime,
    #[props(optional)]
    pub is_starred: bool,
    #[props(optional)]
    pub on_toggle_star: Option<EventHandler<String>>,
}

#[allow(non_snake_case)]
pub fn ModuleCard(
    ModuleCardProps {
        module,
        now,
        is_starred,
        on_toggle_star,
    }: ModuleCardProps,
) -> Element {
    let RegistryModule {
        uuid,
        name,
        author,
        description,
        category,
        downloads,
        version,
        icon,
        last_updated,
        verified_author,
        ..
    } = module;
    let category_label = category_label(&category);
    let category_color = category_color(&category);
    let downloads_label = format_downloads(downloads);
    let is_new = is_recent(last_updated.as_deref(), now);
    let author_label = format!("by {author}");
    let icon_node = icon.as_ref().map(|icon| {
        rsx!(img {
            class: "module-card-icon-image",
            src: "{icon}",
            alt: "",
            width: "52",
            height: "52",
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
    let star_button = render_star_button(&uuid, is_starred, on_toggle_star, "favorite-action");
    let new_tag = is_new.then(|| rsx!(span { class: "module-tag tag-new", "New" }));
    let verified_tag =
        verified_author.then(|| rsx!(span { class: "module-tag tag-verified", "Verified" }));
    let starred_tag = is_starred.then(|| rsx!(span { class: "module-tag tag-starred", "Starred" }));

    rsx! {
        li { class: "module-card-wrapper", key: "{uuid}",
            {star_button}
            div { class: "module-card", style: "--card-color: {category_color};",
                a {
                    class: "module-card-link",
                    href: "/modules/{uuid}",
                    aria_label: "View {name} module",
                }
                div { class: "module-card-content",
                    div { class: "module-card-icon",
                        div { class: "module-card-icon-frame",
                            {if let Some(icon) = icon_node {
                                rsx! { {icon} }
                            } else {
                                rsx!(span { class: "module-card-icon-initial", "{initial}" })
                            }}
                        }
                    }
                    div { class: "module-card-main",
                        div { class: "module-card-header",
                            h3 { "{name}" }
                            {version_badge}
                        }
                        ProfileHover {
                            username: author.clone(),
                            label: Some(author_label),
                            link_class: Some("module-author".to_string()),
                            avatar_url: None,
                        }
                        p { class: "module-description", "{description}" }
                    }
                }
                div { class: "module-card-footer",
                    div { class: "module-tags",
                        span {
                            class: "module-tag tag-category",
                            style: "--tag-color: {category_color};",
                            "{category_label}"
                        }
                        {new_tag}
                        {verified_tag}
                        {starred_tag}
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
                    span { class: "module-downloads", "{downloads_label}" }
                }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ModuleCardRow(
    ModuleCardRowProps {
        module,
        now,
        is_starred,
        on_toggle_star,
    }: ModuleCardRowProps,
) -> Element {
    let RegistryModule {
        uuid,
        name,
        author,
        category,
        downloads,
        version,
        icon,
        last_updated,
        verified_author,
        ..
    } = module;
    let category_label = category_label(&category);
    let category_color = category_color(&category);
    let downloads_label = format_downloads(downloads);
    let is_new = is_recent(last_updated.as_deref(), now);
    let author_label = format!("by {author}");
    let icon_node = icon.as_ref().map(|icon| {
        rsx!(img {
            class: "row-icon-image",
            src: "{icon}",
            alt: "",
            width: "40",
            height: "40",
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
        .map(|version| rsx!(span { class: "version-badge", "v{version}" }));
    let new_badge = is_new.then(|| rsx!(span { class: "new-badge", "New" }));
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
    let starred_badge = is_starred.then(|| {
        rsx!(span { class: "starred-badge", "aria-label": "Starred",
            svg {
                width: "14",
                height: "14",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a.122.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a.122.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a.122.122 0 0 0 1.597-1.16z" }
            }
        })
    });
    let star_button = render_star_button(&uuid, is_starred, on_toggle_star, "row-favorite");

    rsx! {
        li { class: "row-wrapper", key: "{uuid}",
            div { class: "row", style: "--row-color: {category_color};",
                a {
                    class: "row-link",
                    href: "/modules/{uuid}",
                    aria_label: "View {name} module",
                }
                div { class: "row-icon",
                    div { class: "row-icon-frame",
                        {if let Some(icon) = icon_node {
                            rsx! { {icon} }
                        } else {
                            rsx!(span { class: "row-icon-initial", "{initial}" })
                        }}
                    }
                }
                div { class: "row-main",
                    div { class: "row-title",
                        h3 { "{name}" }
                        {version_badge}
                        {new_badge}
                        {verified_badge}
                        {starred_badge}
                    }
                    ProfileHover {
                        username: author.clone(),
                        label: Some(author_label),
                        link_class: Some("row-author".to_string()),
                        avatar_url: None,
                    }
                }
                div { class: "row-category",
                    span {
                        class: "module-tag tag-category",
                        style: "--tag-color: {category_color};",
                        "{category_label}"
                    }
                }
                div { class: "row-downloads",
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
                    "{downloads_label}"
                }
            }
            {star_button}
        }
    }
}

#[component]
pub fn ModuleCardSkeleton() -> Element {
    rsx! {
        div { class: "module-card-skeleton",
            div { class: "module-skeleton-badges",
                div { class: "skeleton-block skeleton-pill" }
            }
            div { class: "module-skeleton-content",
                div { class: "skeleton-block skeleton-avatar" }
                div { class: "module-skeleton-main",
                    div { class: "module-skeleton-header",
                        div { class: "skeleton-block skeleton-title" }
                        div { class: "skeleton-block skeleton-chip" }
                    }
                    div { class: "skeleton-block skeleton-subtitle" }
                    div { class: "module-skeleton-description",
                        div { class: "skeleton-block skeleton-line" }
                        div { class: "skeleton-block skeleton-line short" }
                    }
                }
            }
            div { class: "module-skeleton-footer",
                div { class: "skeleton-block skeleton-pill" }
                div { class: "skeleton-block skeleton-square" }
            }
        }
    }
}

pub(crate) fn format_downloads(downloads: i64) -> String {
    if downloads >= 1_000_000 {
        return format!("{:.1}M", downloads as f64 / 1_000_000.0);
    }
    if downloads >= 1_000 {
        return format!("{:.1}k", downloads as f64 / 1_000.0);
    }
    downloads.to_string()
}

fn is_recent(last_updated: Option<&str>, now: OffsetDateTime) -> bool {
    let last_updated = match last_updated {
        Some(value) => value,
        None => return false,
    };
    let parsed =
        match OffsetDateTime::parse(last_updated, &time::format_description::well_known::Rfc3339) {
            Ok(value) => value,
            Err(_) => return false,
        };
    let window = Duration::days(7);
    now - parsed < window
}

fn render_star_button(
    uuid: &str,
    is_starred: bool,
    on_toggle_star: Option<EventHandler<String>>,
    wrapper_class: &str,
) -> Option<Element> {
    let handler = on_toggle_star?;
    let uuid = uuid.to_string();
    let label = if is_starred {
        "Remove from stars"
    } else {
        "Add to stars"
    };
    let class_name = if is_starred {
        "module-star is-starred"
    } else {
        "module-star"
    };

    Some(rsx! {
        div { class: "{wrapper_class}",
            button {
                class: "{class_name}",
                r#type: "button",
                "aria-label": "{label}",
                "aria-pressed": "{is_starred}",
                onclick: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    handler.call(uuid.clone());
                },
                svg {
                    class: "module-star-icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a.122.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a.122.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a.122.122 0 0 0 1.597-1.16z" }
                }
            }
        }
    })
}
