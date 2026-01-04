use super::module_card::{ModuleCard, ModuleCardRow};
use barforge_types::{ModuleCategory, RegistryModule};
use dioxus::prelude::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSort {
    Popular,
    Trending,
    Recent,
    Alphabetical,
    Downloads,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleViewMode {
    Grid,
    List,
}

#[derive(Props, Clone, PartialEq)]
pub struct ModuleListProps {
    modules: Vec<RegistryModule>,
    query: Option<String>,
    sort: Option<ModuleSort>,
    category: Option<ModuleCategory>,
    page: Option<usize>,
    per_page: Option<usize>,
    #[props(optional)]
    starred: Option<HashSet<String>>,
    #[props(optional)]
    on_toggle_star: Option<EventHandler<String>>,
    #[props(optional)]
    view_mode: Option<ModuleViewMode>,
    now: OffsetDateTime,
}

#[allow(non_snake_case)]
pub fn ModuleList(
    ModuleListProps {
        modules,
        query,
        sort,
        category,
        page,
        per_page,
        starred,
        on_toggle_star,
        view_mode,
        now,
    }: ModuleListProps,
) -> Element {
    let query = query.unwrap_or_default().trim().to_lowercase();
    let view_mode = view_mode.unwrap_or(ModuleViewMode::Grid);
    let mut visible_modules = if query.is_empty() {
        modules
    } else {
        modules
            .into_iter()
            .filter(|module| {
                module.name.to_lowercase().contains(&query)
                    || module.description.to_lowercase().contains(&query)
                    || module.author.to_lowercase().contains(&query)
            })
            .collect::<Vec<_>>()
    };

    if let Some(category) = category {
        visible_modules.retain(|module| module.category == category);
    }

    if let Some(sort) = sort {
        match sort {
            ModuleSort::Popular => {
                sort_by_score(&mut visible_modules, |module| {
                    calculate_popularity_score(module, &now)
                });
            }
            ModuleSort::Trending => {
                sort_by_score(&mut visible_modules, |module| {
                    calculate_trending_score(module, &now)
                });
            }
            ModuleSort::Recent => {
                visible_modules.sort_by(|a, b| {
                    let a_ts = parse_last_updated(a);
                    let b_ts = parse_last_updated(b);
                    b_ts.cmp(&a_ts)
                });
            }
            ModuleSort::Alphabetical => {
                visible_modules.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            ModuleSort::Downloads => {
                visible_modules.sort_by(|a, b| b.downloads.cmp(&a.downloads));
            }
        }
    }

    if let Some(per_page) = per_page.filter(|value| *value > 0) {
        let page = page.unwrap_or(1).max(1);
        let start = (page - 1) * per_page;
        visible_modules = visible_modules
            .into_iter()
            .skip(start)
            .take(per_page)
            .collect();
    }

    if visible_modules.is_empty() {
        return rsx!(div { class: "module-list-empty", "No modules found" });
    }

    let container_class = match view_mode {
        ModuleViewMode::Grid => "module-list module-grid module-container grid",
        ModuleViewMode::List => "module-list module-container list",
    };

    rsx! {
        ul { class: "{container_class}",
            {visible_modules.into_iter().map(|module| {
                let is_starred = starred
                    .as_ref()
                    .map(|set| set.contains(&module.uuid))
                    .unwrap_or(false);
                if view_mode == ModuleViewMode::List {
                    rsx!(ModuleCardRow {
                        module,
                        now,
                        is_starred,
                        on_toggle_star,
                    })
                } else {
                    rsx!(ModuleCard {
                        module,
                        now,
                        is_starred,
                        on_toggle_star,
                    })
                }
            })}
        }
    }
}

fn sort_by_score(modules: &mut [RegistryModule], score_for: impl Fn(&RegistryModule) -> f64) {
    modules.sort_by(|a, b| {
        let a_score = score_for(a);
        let b_score = score_for(b);
        b_score.partial_cmp(&a_score).unwrap_or(Ordering::Equal)
    });
}

fn calculate_popularity_score(module: &RegistryModule, now: &OffsetDateTime) -> f64 {
    let download_score = calculate_download_score(module.downloads);
    let rating_multiplier = calculate_rating_multiplier(module.rating);
    let recency_bonus = calculate_recency_bonus(module.last_updated.as_deref(), now);

    download_score * rating_multiplier * recency_bonus
}

fn calculate_trending_score(module: &RegistryModule, now: &OffsetDateTime) -> f64 {
    let download_score = calculate_download_score(module.downloads);
    let rating_multiplier = calculate_rating_multiplier(module.rating);
    let trending_bonus = calculate_trending_bonus(module.last_updated.as_deref(), now);

    download_score * rating_multiplier * trending_bonus
}

fn calculate_download_score(downloads: i64) -> f64 {
    if downloads <= 0 {
        return 0.0;
    }

    ((downloads + 1) as f64).log10() * 10.0
}

fn calculate_rating_multiplier(rating: Option<f32>) -> f64 {
    let rating = match rating {
        Some(value) => value as f64,
        None => return 1.0,
    };

    0.7 + (rating - 1.0) * 0.15
}

fn calculate_recency_bonus(last_updated: Option<&str>, now: &OffsetDateTime) -> f64 {
    let age_in_days = match age_in_days(last_updated, now) {
        Some(value) => value,
        None => return 1.0,
    };

    let decay_days = 90.0;
    let max_bonus = 0.5;

    if !age_in_days.is_finite() || age_in_days >= decay_days {
        return 1.0;
    }

    let decay_factor = 1.0 - age_in_days / decay_days;
    1.0 + max_bonus * decay_factor
}

fn calculate_trending_bonus(last_updated: Option<&str>, now: &OffsetDateTime) -> f64 {
    let age_in_days = match age_in_days(last_updated, now) {
        Some(value) => value,
        None => return 1.0,
    };

    let half_life = 7.0;
    let max_multiplier = 3.0;
    let min_multiplier = 0.5;

    if !age_in_days.is_finite() {
        return 1.0;
    }

    let decay = 0.5_f64.powf(age_in_days / half_life);
    min_multiplier + (max_multiplier - min_multiplier) * decay
}

fn age_in_days(last_updated: Option<&str>, now: &OffsetDateTime) -> Option<f64> {
    let last_updated = last_updated?;
    let parsed = OffsetDateTime::parse(last_updated, &Rfc3339).ok()?;
    let now_timestamp = now.unix_timestamp() as f64;
    let parsed_timestamp = parsed.unix_timestamp() as f64;
    let age_seconds = now_timestamp - parsed_timestamp;
    Some((age_seconds / 86_400.0).max(0.0))
}

fn parse_last_updated(module: &RegistryModule) -> i64 {
    module
        .last_updated
        .as_deref()
        .and_then(|value| OffsetDateTime::parse(value, &Rfc3339).ok())
        .map_or(0, |value| value.unix_timestamp())
}
