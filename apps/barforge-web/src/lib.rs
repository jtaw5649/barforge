mod api;
mod app;
mod app_time;
mod auth_redirect;
mod auth_ui;
pub mod command_palette;
mod forms;
mod notifications;
mod pages;
mod recently_viewed;
mod routes;
mod sample_data;
#[cfg(feature = "server")]
pub mod server;
mod stars;
mod state;
pub mod theme;

pub use app::{App, AppEntry};
pub use routes::{CollectionId, GithubUsername, ModuleSlug, Route};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::web::Config::new().hydrate(true))
        .launch(AppEntry);
}

#[cfg(feature = "server")]
pub use state::SessionStateOverride;

#[cfg(test)]
pub(crate) use app::app_error_fallback;
#[cfg(test)]
pub(crate) use auth_ui::{AuthGateKind, auth_gate_view};
#[cfg(test)]
pub(crate) use barforge_types::RegistryIndex;
#[cfg(test)]
pub(crate) use forms::{
    FieldError, ProfileFormValues, UploadFormValues, profile_request_spec, update_profile_request,
    upload_error_summary, validate_profile_form, validate_upload_form,
};
#[cfg(test)]
pub(crate) use pages::{
    ModuleDetailReady, ModuleDetailView, resolve_module_detail_view, status_banner_with_retry,
};
#[cfg(test)]
pub(crate) use sample_data::{sample_module_by_uuid, sample_modules};
#[cfg(test)]
pub(crate) use state::{RemoteState, featured_modules_from_state, modules_from_state};

#[cfg(test)]
mod error_boundary_tests {
    use super::*;
    use dioxus::CapturedError;
    use dioxus::prelude::ErrorContext;

    #[test]
    fn error_boundary_renders_fallback() {
        let errors = ErrorContext::new(Some(CapturedError::from_display("boom")));
        let html = dioxus_ssr::render_element(app_error_fallback(errors));

        assert!(
            html.contains("Something went wrong"),
            "fallback missing: {html}"
        );
    }
}

#[cfg(test)]
mod auth_gate_tests {
    use super::*;
    use dioxus::prelude::*;

    fn session(authenticated: bool, is_admin: bool) -> api::SessionResponse {
        api::SessionResponse {
            authenticated,
            is_admin,
            user: authenticated.then(|| api::SessionUser {
                login: "barforge".to_string(),
                email: None,
            }),
        }
    }

    #[test]
    fn auth_gate_prompts_login_when_unauthenticated() {
        let state = RemoteState::Ready(session(false, false));
        let html = dioxus_ssr::render_element(auth_gate_view(
            &state,
            AuthGateKind::User,
            "/dashboard",
            rsx!(div { "Secret" }),
        ));

        assert!(html.contains("Log in to continue"));
        assert!(html.contains("redirect_to=/dashboard"));
    }

    #[test]
    fn auth_gate_allows_authenticated_content() {
        let state = RemoteState::Ready(session(true, false));
        let html = dioxus_ssr::render_element(auth_gate_view(
            &state,
            AuthGateKind::User,
            "/dashboard",
            rsx!(div { "Secret" }),
        ));

        assert!(html.contains("Secret"));
    }

    #[test]
    fn admin_gate_blocks_non_admins() {
        let state = RemoteState::Ready(session(true, false));
        let html = dioxus_ssr::render_element(auth_gate_view(
            &state,
            AuthGateKind::Admin,
            "/admin",
            rsx!(div { "Admin" }),
        ));

        assert!(html.contains("Admin access required"));
    }
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;

    #[test]
    fn status_banner_exposes_live_region_for_loading() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<()>::Loading,
            "Loading",
            "Error",
            "Please try again.",
            None,
        ));

        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-live=\"polite\""));
    }

    #[test]
    fn status_banner_exposes_live_region_for_error() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<()>::Error("Failure".to_string()),
            "Loading",
            "Error",
            "Please try again.",
            None,
        ));

        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("aria-live=\"assertive\""));
    }
}

#[cfg(test)]
mod modules_data_tests {
    use super::{RemoteState, modules_from_state, sample_modules};
    use barforge_types::RegistryIndex;
    use std::collections::BTreeMap;

    #[test]
    fn modules_from_state_falls_back_to_sample_modules_when_unavailable() {
        let modules = modules_from_state(&RemoteState::Unavailable);

        assert!(
            modules
                .iter()
                .any(|module| module.uuid == "weather-wttr@barforge")
        );
    }

    #[test]
    fn modules_from_state_prefers_registry_index() {
        let modules = sample_modules();
        let index = RegistryIndex {
            version: 1,
            modules: modules.clone(),
            categories: BTreeMap::new(),
        };

        let resolved = modules_from_state(&RemoteState::Ready(index));

        assert_eq!(resolved, modules);
    }

    #[test]
    fn modules_from_state_returns_empty_on_error() {
        let modules = modules_from_state(&RemoteState::Error("nope".to_string()));

        assert!(modules.is_empty());
    }

    #[test]
    fn modules_from_state_returns_empty_on_loading() {
        let modules = modules_from_state(&RemoteState::Loading);

        assert!(modules.is_empty());
    }
}

#[cfg(test)]
mod index_status_tests {
    use super::{RegistryIndex, RemoteState, status_banner_with_retry};

    #[test]
    fn index_status_renders_loading() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<RegistryIndex>::Loading,
            "Loading module index...",
            "Index unavailable",
            "Please try again.",
            None,
        ));

        assert!(html.contains("Loading module index"));
    }

    #[test]
    fn index_status_renders_error() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<RegistryIndex>::Error("fetch failed".to_string()),
            "Loading module index...",
            "Index unavailable",
            "Please try again.",
            Some("/modules"),
        ));

        assert!(html.contains("Index unavailable"));
        assert!(html.contains("Please try again"));
        assert!(html.contains("Retry"));
        assert!(html.contains("href=\"/modules\""));
        assert!(!html.contains("fetch failed"));
    }
}

#[cfg(test)]
mod featured_data_tests {
    use super::{
        RemoteState, featured_modules_from_state, sample_modules, status_banner_with_retry,
    };
    use barforge_types::FeaturedModulesResponse;

    #[test]
    fn featured_modules_from_state_prefers_payload() {
        let payload = FeaturedModulesResponse {
            version: 1,
            featured: vec![sample_modules()[0].clone()],
            popular: vec![sample_modules()[1].clone()],
            recent: vec![sample_modules()[2].clone()],
        };
        let fallback = sample_modules();

        let resolved = featured_modules_from_state(&RemoteState::Ready(payload.clone()), &fallback);

        assert_eq!(resolved, payload);
    }

    #[test]
    fn featured_modules_from_state_falls_back_on_error() {
        let fallback = sample_modules();

        let resolved =
            featured_modules_from_state(&RemoteState::Error("nope".to_string()), &fallback);

        assert_eq!(resolved.featured, fallback);
        assert_eq!(resolved.popular, fallback);
        assert_eq!(resolved.recent, fallback);
    }

    #[test]
    fn featured_status_renders_loading() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<FeaturedModulesResponse>::Loading,
            "Loading featured modules...",
            "Featured modules unavailable",
            "Please try again.",
            None,
        ));

        assert!(html.contains("Loading featured modules"));
    }

    #[test]
    fn featured_status_renders_error() {
        let html = dioxus_ssr::render_element(status_banner_with_retry(
            &RemoteState::<FeaturedModulesResponse>::Error("fetch failed".to_string()),
            "Loading featured modules...",
            "Featured modules unavailable",
            "Please try again.",
            Some("/modules/search"),
        ));

        assert!(html.contains("Featured modules unavailable"));
        assert!(html.contains("Please try again"));
        assert!(html.contains("Retry"));
        assert!(html.contains("href=\"/modules/search\""));
        assert!(!html.contains("fetch failed"));
    }
}

#[cfg(test)]
mod upload_form_validation_tests {
    use super::{FieldError, UploadFormValues, upload_error_summary, validate_upload_form};
    use crate::forms::upload_request_spec;

    #[test]
    fn upload_form_requires_expected_fields() {
        let errors = validate_upload_form(&UploadFormValues::default());

        let messages: Vec<&str> = errors.iter().map(|error| error.message).collect();
        assert!(messages.contains(&"Module name is required."));
        assert!(messages.contains(&"Description is required."));
        assert!(messages.contains(&"Category is required."));
        assert!(messages.contains(&"Repository URL is required."));
        assert!(messages.contains(&"Version is required."));
        assert!(messages.contains(&"License is required."));
        assert!(messages.contains(&"Package file is required."));
    }

    #[test]
    fn upload_form_requires_https_repo_url() {
        let values = UploadFormValues {
            name: "Clock".to_string(),
            description: "Minimal clock".to_string(),
            category: "time".to_string(),
            repo_url: "http://example.com".to_string(),
            version: "1.0.0".to_string(),
            license: "MIT".to_string(),
            package_file: "clock.tar.gz".to_string(),
        };
        let errors = validate_upload_form(&values);

        assert!(errors.iter().any(|error| {
            error.field == "repo_url" && error.message == "Repository URL must start with https://"
        }));
    }

    #[test]
    fn upload_error_summary_lists_messages() {
        let errors = vec![FieldError {
            field: "name",
            message: "Module name is required.",
        }];
        let html = dioxus_ssr::render_element(upload_error_summary(&errors));

        assert!(html.contains("Please correct the highlighted fields"));
        assert!(html.contains("Module name is required."));
    }

    #[test]
    fn upload_request_spec_matches_routes() {
        let spec = upload_request_spec();

        assert_eq!(spec.upload_endpoint, "/api/upload");
        assert_eq!(spec.csrf_endpoint, "/api/csrf-token");
        assert_eq!(spec.csrf_header, "x-csrf-token");
        assert_eq!(spec.turnstile_field, "cf-turnstile-response");
    }
}

#[cfg(test)]
mod profile_form_validation_tests {
    use super::{
        ProfileFormValues, profile_request_spec, update_profile_request, validate_profile_form,
    };

    #[test]
    fn profile_form_rejects_long_display_name() {
        let values = ProfileFormValues {
            display_name: "a".repeat(51),
            ..ProfileFormValues::default()
        };
        let errors = validate_profile_form(&values);

        assert!(errors.iter().any(|error| {
            error.field == "display_name"
                && error.message == "Display name must be 50 characters or less"
        }));
    }

    #[test]
    fn profile_form_rejects_long_bio() {
        let values = ProfileFormValues {
            bio: "b".repeat(501),
            ..ProfileFormValues::default()
        };
        let errors = validate_profile_form(&values);

        assert!(
            errors.iter().any(|error| error.field == "bio"
                && error.message == "Bio must be 500 characters or less")
        );
    }

    #[test]
    fn profile_form_rejects_invalid_url() {
        let values = ProfileFormValues {
            website_url: "not-a-url".to_string(),
            ..ProfileFormValues::default()
        };
        let errors = validate_profile_form(&values);

        assert!(errors
            .iter()
            .any(|error| error.field == "website_url" && error.message == "Must be a valid URL"));
    }

    #[test]
    fn update_profile_request_maps_empty_to_none() {
        let values = ProfileFormValues {
            display_name: "Octo".to_string(),
            github_url: "https://github.com/octo".to_string(),
            ..ProfileFormValues::default()
        };
        let payload = update_profile_request(&values);

        assert_eq!(payload.display_name.as_deref(), Some("Octo"));
        assert!(payload.bio.is_none());
        assert_eq!(
            payload.github_url.as_deref(),
            Some("https://github.com/octo")
        );
        assert!(payload.website_url.is_none());
        assert!(payload.twitter_url.is_none());
    }

    #[test]
    fn profile_request_spec_matches_routes() {
        let spec = profile_request_spec();

        assert_eq!(spec.profile_endpoint, "/api/users/me");
        assert_eq!(spec.csrf_endpoint, "/api/csrf-token");
        assert_eq!(spec.csrf_header, "x-csrf-token");
    }
}

#[cfg(test)]
mod module_detail_view_tests {
    use super::{
        ModuleDetailReady, ModuleDetailView, RemoteState, resolve_module_detail_view,
        sample_module_by_uuid,
    };
    use crate::api::DEFAULT_API_BASE_URL;
    use crate::sample_data::sample_collection;
    use barforge_types::{Review, ReviewUser, Screenshot};

    #[test]
    fn module_detail_view_returns_loading() {
        let view = resolve_module_detail_view(
            "weather-wttr@barforge",
            &RemoteState::Loading,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
        );

        assert_eq!(view, ModuleDetailView::Loading);
    }

    #[test]
    fn module_detail_view_returns_not_found_for_404() {
        let view = resolve_module_detail_view(
            "missing@barforge",
            &RemoteState::Error("unexpected status: 404".to_string()),
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
        );

        assert_eq!(view, ModuleDetailView::NotFound);
    }

    #[test]
    fn module_detail_view_unavailable_uses_sample_data() {
        let view = resolve_module_detail_view(
            "weather-wttr@barforge",
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
        );

        match view {
            ModuleDetailView::Ready(ready) => {
                let ModuleDetailReady {
                    module,
                    versions,
                    screenshots,
                    reviews,
                    ..
                } = *ready;
                assert_eq!(module.uuid, "weather-wttr@barforge");
                assert!(!versions.is_empty());
                assert!(!screenshots.is_empty());
                assert!(reviews.is_empty());
            }
            _ => panic!("expected ready view"),
        }
    }

    #[test]
    fn module_detail_view_builds_screenshot_urls() {
        let screenshot = sample_screenshot("screenshots/weather-wttr@barforge/shot-1.png");
        let view = resolve_module_detail_view(
            "weather-wttr@barforge",
            &RemoteState::Ready(sample_module_by_uuid("weather-wttr@barforge")),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Ready(vec![screenshot]),
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
        );

        match view {
            ModuleDetailView::Ready(ready) => {
                let ModuleDetailReady { screenshots, .. } = *ready;
                let expected = format!(
                    "{}/screenshots/weather-wttr@barforge/shot-1.png",
                    DEFAULT_API_BASE_URL
                );
                assert_eq!(screenshots, vec![expected]);
            }
            _ => panic!("expected ready view"),
        }
    }

    #[test]
    fn module_detail_view_prefers_review_payload() {
        let review = sample_review("alice", 4);
        let view = resolve_module_detail_view(
            "weather-wttr@barforge",
            &RemoteState::Ready(sample_module_by_uuid("weather-wttr@barforge")),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Unavailable,
            &RemoteState::Ready(vec![review.clone()]),
            &RemoteState::Unavailable,
        );

        match view {
            ModuleDetailView::Ready(ready) => {
                let ModuleDetailReady { reviews, .. } = *ready;
                assert_eq!(reviews, vec![review]);
            }
            _ => panic!("expected ready view"),
        }
    }

    #[test]
    fn module_detail_view_prefers_collections_payload() {
        let collections = vec![sample_collection("1")];
        let view = resolve_module_detail_view(
            "weather-wttr@barforge",
            &RemoteState::Ready(sample_module_by_uuid("weather-wttr@barforge")),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Ready(Vec::new()),
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Ready(collections.clone()),
        );

        match view {
            ModuleDetailView::Ready(ready) => {
                let ModuleDetailReady {
                    collections: loaded,
                    ..
                } = *ready;
                assert_eq!(loaded, collections);
            }
            _ => panic!("expected ready view"),
        }
    }

    fn sample_review(username: &str, rating: i64) -> Review {
        Review {
            id: 1,
            rating,
            title: Some("Solid module".to_string()),
            body: Some("Dependable in production.".to_string()),
            helpful_count: 2,
            user: ReviewUser {
                username: username.to_string(),
                avatar_url: None,
            },
            created_at: "2025-01-02T00:00:00Z".to_string(),
            updated_at: None,
        }
    }

    fn sample_screenshot(r2_key: &str) -> Screenshot {
        Screenshot {
            id: 1,
            r2_key: r2_key.to_string(),
            alt_text: None,
            position: 1,
            created_at: "2025-01-02T00:00:00Z".to_string(),
        }
    }
}
