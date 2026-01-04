#[cfg(any(target_arch = "wasm32", test))]
use barforge_types::Versioned;
use barforge_types::{
    AdminStats, Collection, CollectionDetailResponse, FeaturedModulesResponse, LandingData,
    PublicStats, RegistryIndex, RegistryModule, Review, Screenshot, SubmissionsResponse,
    UserProfile, VersionHistoryEntry,
};
use dioxus::prelude::*;

use crate::api;
use crate::sample_data::{
    sample_collections, sample_landing_data, sample_modules, sample_user_profile,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RemoteState<T> {
    Loading,
    Ready(T),
    Error(String),
    Unavailable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionStateOverride {
    pub authenticated: bool,
    pub is_admin: bool,
    pub login: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct SessionStateContext(pub Signal<RemoteState<SessionStateOverride>>);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UserProfileView {
    pub profile: UserProfile,
    pub modules: Vec<RegistryModule>,
    pub collections: Vec<Collection>,
    pub total_downloads: i64,
}

#[cfg(not(target_arch = "wasm32"))]
fn touch_remote_state_ready() {
    let _ = RemoteState::Ready(());
}

pub(crate) fn use_registry_index_state() -> RemoteState<RegistryIndex> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_registry_index().await });
        match resource.read().as_ref() {
            Some(Ok(index)) => RemoteState::Ready(index.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        touch_remote_state_ready();
        RemoteState::Unavailable
    }
}

pub(crate) fn use_featured_state() -> RemoteState<FeaturedModulesResponse> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_featured_modules().await });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        RemoteState::Unavailable
    }
}

pub(crate) fn use_landing_state() -> RemoteState<LandingData> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_landing_data().await });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        RemoteState::Unavailable
    }
}

pub(crate) fn use_github_stats_state() -> RemoteState<api::GithubStats> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_github_stats().await });
        match resource.read().as_ref() {
            Some(Ok(stats)) => RemoteState::Ready(stats.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        RemoteState::Unavailable
    }
}

pub(crate) fn landing_stats_from_state(state: &RemoteState<LandingData>) -> PublicStats {
    match state {
        RemoteState::Ready(payload) => payload.stats.clone(),
        RemoteState::Unavailable => sample_landing_data().stats,
        RemoteState::Loading | RemoteState::Error(_) => PublicStats {
            total_modules: 0,
            total_downloads: 0,
            total_authors: 0,
        },
    }
}

fn session_response_from_override(override_state: &SessionStateOverride) -> api::SessionResponse {
    let user = if override_state.authenticated {
        override_state.login.clone().map(|login| api::SessionUser {
            login,
            email: override_state.email.clone(),
        })
    } else {
        None
    };

    api::SessionResponse {
        authenticated: override_state.authenticated,
        is_admin: override_state.is_admin,
        user,
    }
}

#[cfg(target_arch = "wasm32")]
fn session_override_from_response(session: &api::SessionResponse) -> SessionStateOverride {
    SessionStateOverride {
        authenticated: session.authenticated,
        is_admin: session.is_admin,
        login: session.user.as_ref().map(|user| user.login.clone()),
        email: session.user.as_ref().and_then(|user| user.email.clone()),
    }
}

#[component]
pub(crate) fn SessionStateProvider(children: Element) -> Element {
    let override_state = try_use_context::<SessionStateOverride>();
    let initial_state = override_state
        .as_ref()
        .map(|state| RemoteState::Ready(state.clone()))
        .unwrap_or_else(|| {
            if cfg!(target_arch = "wasm32") {
                RemoteState::Loading
            } else {
                RemoteState::Unavailable
            }
        });
    let state = use_signal(move || initial_state);

    #[cfg(target_arch = "wasm32")]
    if override_state.is_none() {
        let resource = use_resource(|| async move { api::fetch_session_status().await });
        use_effect({
            let mut state = state;
            move || {
                let next_state = match resource.read().as_ref() {
                    Some(Ok(session)) => {
                        RemoteState::Ready(session_override_from_response(session))
                    }
                    Some(Err(err)) => RemoteState::Error(err.to_string()),
                    None => RemoteState::Loading,
                };
                if *state.peek() != next_state {
                    state.set(next_state);
                }
            }
        });
    }

    use_context_provider(|| SessionStateContext(state));

    rsx! { {children} }
}

pub(crate) fn use_session_state() -> RemoteState<api::SessionResponse> {
    if let Some(context) = try_use_context::<SessionStateContext>() {
        let state = context.0.read();
        return match &*state {
            RemoteState::Ready(session) => {
                RemoteState::Ready(session_response_from_override(session))
            }
            RemoteState::Loading => RemoteState::Loading,
            RemoteState::Error(err) => RemoteState::Error(err.clone()),
            RemoteState::Unavailable => RemoteState::Unavailable,
        };
    }

    if let Some(override_state) = try_use_context::<SessionStateOverride>() {
        return RemoteState::Ready(session_response_from_override(&override_state));
    }

    #[cfg(target_arch = "wasm32")]
    {
        let resource = use_resource(|| async move { api::fetch_session_status().await });
        match resource.read().as_ref() {
            Some(Ok(session)) => RemoteState::Ready(session.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        RemoteState::Unavailable
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn admin_stats_state_from_response(
    response: Option<Result<Option<Versioned<AdminStats>>, String>>,
    is_admin: bool,
) -> RemoteState<AdminStats> {
    if !is_admin {
        return RemoteState::Unavailable;
    }
    match response {
        Some(Ok(Some(payload))) => RemoteState::Ready(payload.payload),
        Some(Ok(None)) => RemoteState::Unavailable,
        Some(Err(err)) => RemoteState::Error(err),
        None => RemoteState::Loading,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn admin_submissions_state_from_response(
    response: Option<Result<Option<Versioned<SubmissionsResponse>>, String>>,
    is_admin: bool,
) -> RemoteState<SubmissionsResponse> {
    if !is_admin {
        return RemoteState::Unavailable;
    }
    match response {
        Some(Ok(Some(payload))) => RemoteState::Ready(payload.payload),
        Some(Ok(None)) => RemoteState::Unavailable,
        Some(Err(err)) => RemoteState::Error(err),
        None => RemoteState::Loading,
    }
}

pub(crate) fn use_admin_stats_state(is_admin: bool) -> RemoteState<AdminStats> {
    #[cfg(target_arch = "wasm32")]
    {
        let resource = use_resource(move || async move {
            if !api::LIVE_API_ENABLED || !is_admin {
                return Ok(None);
            }
            api::fetch_admin_stats().await.map(Some)
        });
        let response = resource.read().as_ref().map(|result| match result {
            Ok(payload) => Ok(payload.clone()),
            Err(err) => Err(err.to_string()),
        });
        admin_stats_state_from_response(response, is_admin)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = is_admin;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_admin_submissions_state(is_admin: bool) -> RemoteState<SubmissionsResponse> {
    #[cfg(target_arch = "wasm32")]
    {
        let resource = use_resource(move || async move {
            if !api::LIVE_API_ENABLED || !is_admin {
                return Ok(None);
            }
            api::fetch_admin_submissions().await.map(Some)
        });
        let response = resource.read().as_ref().map(|result| match result {
            Ok(payload) => Ok(payload.clone()),
            Err(err) => Err(err.to_string()),
        });
        admin_submissions_state_from_response(response, is_admin)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = is_admin;
        RemoteState::Unavailable
    }
}

pub(crate) fn admin_stats_from_state(state: &RemoteState<AdminStats>) -> AdminStats {
    match state {
        RemoteState::Ready(stats) => stats.clone(),
        _ => AdminStats {
            total_modules: 0,
            total_users: 0,
            total_downloads: 0,
            pending_submissions: 0,
        },
    }
}

pub(crate) fn admin_submissions_from_state(
    state: &RemoteState<SubmissionsResponse>,
) -> SubmissionsResponse {
    match state {
        RemoteState::Ready(submissions) => submissions.clone(),
        _ => SubmissionsResponse {
            submissions: Vec::new(),
            total: 0,
        },
    }
}

pub(crate) fn modules_from_state(state: &RemoteState<RegistryIndex>) -> Vec<RegistryModule> {
    match state {
        RemoteState::Ready(index) if !index.modules.is_empty() => index.modules.clone(),
        RemoteState::Unavailable => sample_modules(),
        RemoteState::Ready(_) => Vec::new(),
        RemoteState::Error(_) | RemoteState::Loading => Vec::new(),
    }
}

pub(crate) fn featured_modules_from_state(
    state: &RemoteState<FeaturedModulesResponse>,
    fallback: &[RegistryModule],
) -> FeaturedModulesResponse {
    match state {
        RemoteState::Ready(payload) => payload.clone(),
        _ => FeaturedModulesResponse {
            version: 0,
            featured: fallback.to_vec(),
            popular: fallback.to_vec(),
            recent: fallback.to_vec(),
        },
    }
}

pub(crate) fn github_stats_from_state(state: &RemoteState<api::GithubStats>) -> api::GithubStats {
    match state {
        RemoteState::Ready(stats) => stats.clone(),
        _ => api::GithubStats {
            stars: 1280,
            forks: 96,
            contributors: 18,
        },
    }
}

pub(crate) fn use_module_detail_state(uuid: &str) -> RemoteState<RegistryModule> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let uuid = uuid.to_string();
        let resource = use_resource(move || {
            let uuid = uuid.clone();
            async move { api::fetch_module_detail(&uuid).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = uuid;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_module_versions_state(uuid: &str) -> RemoteState<Vec<VersionHistoryEntry>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let uuid = uuid.to_string();
        let resource = use_resource(move || {
            let uuid = uuid.clone();
            async move { api::fetch_module_versions(&uuid).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.payload.versions.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = uuid;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_related_modules_state(uuid: &str) -> RemoteState<Vec<RegistryModule>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let uuid = uuid.to_string();
        let resource = use_resource(move || {
            let uuid = uuid.clone();
            async move { api::fetch_related_modules(&uuid).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.payload.modules.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = uuid;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_module_screenshots_state(uuid: &str) -> RemoteState<Vec<Screenshot>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let uuid = uuid.to_string();
        let resource = use_resource(move || {
            let uuid = uuid.clone();
            async move { api::fetch_module_screenshots(&uuid).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.payload.screenshots.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = uuid;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_module_reviews_state(uuid: &str) -> RemoteState<Vec<Review>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let uuid = uuid.to_string();
        let resource = use_resource(move || {
            let uuid = uuid.clone();
            async move { api::fetch_module_reviews(&uuid).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.payload.reviews.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = uuid;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_user_profile_state(username: &str) -> RemoteState<UserProfile> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let username = username.to_string();
        let resource = use_resource(move || {
            let username = username.clone();
            async move { api::fetch_user_profile(&username).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = username;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_user_profile_me_state(authenticated: bool) -> RemoteState<UserProfile> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED || !authenticated {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_user_profile_me().await });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = authenticated;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_modules_mine_state(authenticated: bool) -> RemoteState<Vec<RegistryModule>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED || !authenticated {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(|| async move { api::fetch_modules_mine().await });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.modules.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = authenticated;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_user_modules_state(username: &str) -> RemoteState<Vec<RegistryModule>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let username = username.to_string();
        let resource = use_resource(move || {
            let username = username.clone();
            async move { api::fetch_user_modules(&username).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.modules.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = username;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_user_collections_state(username: &str) -> RemoteState<Vec<Collection>> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let username = username.to_string();
        let resource = use_resource(move || {
            let username = username.clone();
            async move { api::fetch_user_collections(&username).await }
        });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.collections.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = username;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_collections_state(authenticated: bool) -> RemoteState<Vec<Collection>> {
    #[cfg(target_arch = "wasm32")]
    {
        let resource = use_resource(move || async move {
            if !api::LIVE_API_ENABLED || !authenticated {
                return Ok(None);
            }
            api::fetch_collections().await.map(Some)
        });
        match resource.read().as_ref() {
            Some(Ok(Some(payload))) => RemoteState::Ready(payload.payload.collections.clone()),
            Some(Ok(None)) => RemoteState::Unavailable,
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = authenticated;
        RemoteState::Unavailable
    }
}

pub(crate) fn use_collection_detail_state(id: i64) -> RemoteState<CollectionDetailResponse> {
    #[cfg(target_arch = "wasm32")]
    {
        if !api::LIVE_API_ENABLED {
            return RemoteState::Unavailable;
        }
        let resource = use_resource(move || async move { api::fetch_collection_detail(id).await });
        match resource.read().as_ref() {
            Some(Ok(payload)) => RemoteState::Ready(payload.payload.clone()),
            Some(Err(err)) => RemoteState::Error(err.to_string()),
            None => RemoteState::Loading,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        RemoteState::Unavailable
    }
}

pub(crate) fn user_profile_view_from_state(
    username: &str,
    profile_state: &RemoteState<UserProfile>,
    modules_state: &RemoteState<Vec<RegistryModule>>,
    collections_state: &RemoteState<Vec<Collection>>,
) -> UserProfileView {
    let modules = match modules_state {
        RemoteState::Ready(modules) => modules.clone(),
        RemoteState::Unavailable => sample_modules(),
        RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
    };
    let profile = match profile_state {
        RemoteState::Ready(profile) => profile.clone(),
        RemoteState::Unavailable => sample_user_profile(username, modules.len()),
        RemoteState::Loading | RemoteState::Error(_) => {
            sample_user_profile(username, modules.len())
        }
    };
    let collections = match collections_state {
        RemoteState::Ready(collections) => collections.clone(),
        RemoteState::Unavailable => sample_collections(&profile.username),
        RemoteState::Loading | RemoteState::Error(_) => Vec::new(),
    };
    let total_downloads = modules.iter().map(|module| module.downloads).sum();

    UserProfileView {
        profile,
        modules,
        collections,
        total_downloads,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteState, SessionStateContext, SessionStateOverride, admin_stats_from_state,
        admin_stats_state_from_response, admin_submissions_from_state,
        admin_submissions_state_from_response, github_stats_from_state, landing_stats_from_state,
        use_modules_mine_state, use_session_state, use_user_profile_me_state,
        user_profile_view_from_state,
    };
    use crate::api::GithubStats;
    use crate::sample_data::{
        sample_collections, sample_landing_data, sample_modules, sample_user_profile,
    };
    use barforge_types::{
        AdminStats, Collection, CollectionOwner, Submission, SubmissionsResponse, UserProfile,
        UserRole, Versioned,
    };
    use dioxus::prelude::*;

    #[test]
    fn github_stats_from_state_uses_ready_stats() {
        let state = RemoteState::Ready(GithubStats {
            stars: 42,
            forks: 7,
            contributors: 5,
        });

        let stats = github_stats_from_state(&state);

        assert_eq!(stats.stars, 42);
        assert_eq!(stats.forks, 7);
        assert_eq!(stats.contributors, 5);
    }

    #[test]
    fn github_stats_from_state_falls_back_when_unavailable() {
        let state = RemoteState::Unavailable;

        let stats = github_stats_from_state(&state);

        assert_eq!(stats.stars, 1280);
        assert_eq!(stats.forks, 96);
        assert_eq!(stats.contributors, 18);
    }

    #[test]
    fn landing_stats_from_state_uses_ready_stats() {
        let state = RemoteState::Ready(sample_landing_data());

        let stats = landing_stats_from_state(&state);

        assert_eq!(stats.total_modules, 128);
        assert_eq!(stats.total_downloads, 24800);
        assert_eq!(stats.total_authors, 42);
    }

    #[test]
    fn landing_stats_from_state_uses_sample_when_unavailable() {
        let state = RemoteState::Unavailable;

        let stats = landing_stats_from_state(&state);
        let sample = sample_landing_data();

        assert_eq!(stats, sample.stats);
    }

    #[test]
    fn landing_stats_from_state_falls_back_when_error() {
        let state = RemoteState::Error("no landing".to_string());

        let stats = landing_stats_from_state(&state);

        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.total_authors, 0);
    }

    #[component]
    fn SessionStateProbe() -> Element {
        let state = use_session_state();
        let label = match state {
            RemoteState::Ready(session) if session.authenticated => "session-authenticated",
            RemoteState::Ready(_) => "session-guest",
            RemoteState::Loading => "session-loading",
            RemoteState::Error(_) => "session-error",
            RemoteState::Unavailable => "session-unavailable",
        };

        rsx!(div { id: "session-probe", "{label}" })
    }

    #[component]
    fn SessionStateTestApp(state: RemoteState<SessionStateOverride>) -> Element {
        let state_signal = use_signal(move || state);
        use_context_provider(|| SessionStateContext(state_signal));

        rsx!(SessionStateProbe {})
    }

    #[test]
    fn session_state_uses_context_override() {
        let state = RemoteState::Ready(SessionStateOverride {
            authenticated: true,
            is_admin: false,
            login: Some("barforge".to_string()),
            email: None,
        });
        let html = dioxus_ssr::render_element(rsx!(SessionStateTestApp { state }));

        assert!(html.contains("session-authenticated"));
    }

    fn sample_admin_stats() -> AdminStats {
        AdminStats {
            total_modules: 128,
            total_users: 42_000,
            total_downloads: 9001,
            pending_submissions: 1,
        }
    }

    fn sample_submission(id: i64) -> Submission {
        Submission {
            id,
            submitter_id: 7,
            uuid: format!("submission-{id}"),
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
    fn admin_stats_state_from_response_requires_admin() {
        let stats = Versioned {
            version: 1,
            payload: sample_admin_stats(),
        };

        let state = admin_stats_state_from_response(Some(Ok(Some(stats))), false);

        assert_eq!(state, RemoteState::Unavailable);
    }

    #[test]
    fn admin_stats_state_from_response_uses_payload() {
        let stats = Versioned {
            version: 1,
            payload: sample_admin_stats(),
        };

        let state = admin_stats_state_from_response(Some(Ok(Some(stats.clone()))), true);

        assert_eq!(state, RemoteState::Ready(stats.payload));
    }

    #[test]
    fn admin_stats_state_from_response_surfaces_error() {
        let state = admin_stats_state_from_response(Some(Err("stats failed".to_string())), true);

        assert_eq!(state, RemoteState::Error("stats failed".to_string()));
    }

    #[test]
    fn admin_stats_from_state_falls_back_to_zero() {
        let state = RemoteState::Unavailable;

        let stats = admin_stats_from_state(&state);

        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.pending_submissions, 0);
    }

    #[test]
    fn admin_submissions_state_from_response_uses_payload() {
        let payload = SubmissionsResponse {
            submissions: vec![sample_submission(1)],
            total: 1,
        };
        let response = Versioned {
            version: 1,
            payload: payload.clone(),
        };

        let state = admin_submissions_state_from_response(Some(Ok(Some(response))), true);

        assert_eq!(state, RemoteState::Ready(payload));
    }

    #[test]
    fn admin_submissions_state_from_response_handles_loading() {
        let state = admin_submissions_state_from_response(None, true);

        assert_eq!(state, RemoteState::Loading);
    }

    #[test]
    fn admin_submissions_from_state_falls_back_to_empty() {
        let state = RemoteState::Unavailable;

        let submissions = admin_submissions_from_state(&state);

        assert!(submissions.submissions.is_empty());
        assert_eq!(submissions.total, 0);
    }

    #[test]
    fn user_profile_view_uses_ready_payloads() {
        let profile = UserProfile {
            id: 7,
            username: "barforge".to_string(),
            display_name: Some("Barforge".to_string()),
            avatar_url: None,
            bio: None,
            website_url: None,
            github_url: None,
            twitter_url: None,
            bluesky_url: None,
            discord_url: None,
            sponsor_url: None,
            verified_author: true,
            role: UserRole::User,
            module_count: 2,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let modules = sample_modules().into_iter().take(2).collect::<Vec<_>>();
        let collections = vec![Collection {
            id: 1,
            user_id: 7,
            name: "Ops Essentials".to_string(),
            description: None,
            visibility: "public".to_string(),
            module_count: 2,
            owner: CollectionOwner {
                username: "barforge".to_string(),
                display_name: None,
                avatar_url: None,
            },
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        }];

        let view = user_profile_view_from_state(
            "barforge",
            &RemoteState::Ready(profile.clone()),
            &RemoteState::Ready(modules.clone()),
            &RemoteState::Ready(collections.clone()),
        );

        let expected_downloads = modules.iter().map(|module| module.downloads).sum::<i64>();

        assert_eq!(view.profile, profile);
        assert_eq!(view.modules, modules);
        assert_eq!(view.collections, collections);
        assert_eq!(view.total_downloads, expected_downloads);
    }

    #[test]
    fn user_profile_view_falls_back_when_unavailable() {
        let view = user_profile_view_from_state(
            "barforge",
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
            &RemoteState::Unavailable,
        );

        let expected_modules = sample_modules();
        let expected_profile = sample_user_profile("barforge", expected_modules.len());

        assert_eq!(view.profile, expected_profile);
        assert_eq!(view.modules, expected_modules);
        assert_eq!(view.total_downloads, 5900);
        assert_eq!(view.collections, sample_collections("barforge"));
    }

    #[test]
    fn dashboard_profile_state_unavailable_without_wasm() {
        let state = use_user_profile_me_state(true);

        assert_eq!(state, RemoteState::Unavailable);
    }

    #[test]
    fn dashboard_modules_state_unavailable_without_wasm() {
        let state = use_modules_mine_state(true);

        assert_eq!(state, RemoteState::Unavailable);
    }
}
