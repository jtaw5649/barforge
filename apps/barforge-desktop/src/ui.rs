use barforge_types::{RegistryModule, ReviewsResponse};
use dioxus::prelude::*;

use crate::state::{BrowseState, InstallStage, InstalledModule};

#[component]
pub fn BrowseShell(initial_modules: Vec<RegistryModule>, initial_search_query: String) -> Element {
    let modules = use_signal(|| initial_modules);
    let state = use_signal(|| BrowseState {
        search_query: initial_search_query,
        ..BrowseState::default()
    });

    let filtered_names = use_memo(move || {
        let state = state.read();
        let modules = modules.read();
        state
            .filtered_modules(&modules)
            .into_iter()
            .map(|module| module.name.clone())
            .collect::<Vec<_>>()
    });

    rsx! {
        ul {
            for name in filtered_names.read().iter() {
                li { "{name}" }
            }
        }
    }
}

#[component]
pub fn ModuleDetailShell(
    reviews: ReviewsResponse,
    installing: bool,
    install_stage: Option<InstallStage>,
) -> Element {
    let stage_text = if installing {
        install_stage.map(|stage| stage.description())
    } else {
        None
    };

    rsx! {
        section {
            if let Some(text) = stage_text {
                p { "{text}" }
            }
            for review in reviews.reviews.iter() {
                article {
                    if let Some(title) = review.title.as_ref() {
                        h3 { "{title}" }
                    }
                    if let Some(body) = review.body.as_ref() {
                        p { "{body}" }
                    }
                    span { "{review.user.username}" }
                    span { "{review.rating}" }
                }
            }
        }
    }
}

#[component]
pub fn UpdatesShell(modules: Vec<InstalledModule>, updating_all: bool) -> Element {
    let updates = modules
        .iter()
        .filter(|module| module.has_update())
        .collect::<Vec<_>>();
    let update_count = updates.len();
    let summary = if update_count > 0 {
        format!(
            "{} update{} available",
            update_count,
            if update_count == 1 { "" } else { "s" }
        )
    } else {
        "All modules are up to date".to_string()
    };
    let action_text = if updating_all {
        "Updating...".to_string()
    } else if update_count > 0 {
        format!("Update All ({})", update_count)
    } else {
        "No Updates".to_string()
    };

    rsx! {
        section {
            h2 { "Updates" }
            p { "{summary}" }
            button { "{action_text}" }
            ul {
                for module in updates {
                    li { "{module.name}" }
                }
            }
        }
    }
}

#[component]
pub fn DesktopApp() -> Element {
    rsx! {
        DesktopEntry {}
    }
}

#[component]
pub fn DesktopEntry() -> Element {
    rsx! {
        BrowseShell {
            initial_modules: Vec::new(),
            initial_search_query: String::new(),
        }
    }
}
