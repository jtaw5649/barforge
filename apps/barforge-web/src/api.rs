#[cfg(target_arch = "wasm32")]
use barforge_types::CollectionDetailResponse;
#[cfg(target_arch = "wasm32")]
use barforge_types::NotificationPreferences;
#[cfg(target_arch = "wasm32")]
use barforge_types::StarRequest;
#[cfg(any(all(test, feature = "server"), target_arch = "wasm32"))]
use barforge_types::StarsResponse;
#[cfg(all(test, feature = "server"))]
use barforge_types::UnreadCountResponse;
#[cfg(target_arch = "wasm32")]
use barforge_types::{AdminStats, RejectRequest, SubmissionsResponse, VerifyResponse};
use barforge_types::{
    CollectionsResponse, FeaturedModulesResponse, LandingData, ModulesResponse, RegistryIndex,
    RegistryModule, ReviewsResponse, ScreenshotsResponse, UserProfile, Versioned, VersionsResponse,
};
#[cfg(any(all(test, feature = "server"), target_arch = "wasm32"))]
use barforge_types::{MarkAllReadResponse, MarkReadResponse, NotificationsResponse};
use dioxus::prelude::{ServerFnError, get};
#[cfg(any(feature = "server", target_arch = "wasm32"))]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const DEFAULT_API_BASE_URL: &str = "https://api.barforge.dev";
#[cfg(feature = "server")]
const GITHUB_API_BASE: &str = "https://api.github.com/repos/jtaw5649";
#[cfg(feature = "server")]
const GITHUB_API_ACCEPT: &str = "application/vnd.github+json";
#[cfg(feature = "server")]
const GITHUB_USER_AGENT: &str = "barforge";

#[cfg(target_arch = "wasm32")]
pub const LIVE_API_ENABLED: bool = match option_env!("BARFORGE_API_LIVE") {
    Some(_) => true,
    None => !cfg!(debug_assertions),
};

pub fn api_base_url() -> &'static str {
    option_env!("BARFORGE_API_BASE_URL")
        .or_else(|| option_env!("PUBLIC_API_BASE_URL"))
        .unwrap_or(DEFAULT_API_BASE_URL)
}

#[cfg(feature = "server")]
fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    format!("{base}/api/v1/{path}")
}

#[cfg(any(feature = "server", target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    message: String,
}

#[cfg(any(feature = "server", target_arch = "wasm32"))]
impl ApiError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(any(feature = "server", target_arch = "wasm32"))]
impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[cfg(any(feature = "server", target_arch = "wasm32"))]
impl std::error::Error for ApiError {}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SessionUser {
    pub login: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SessionResponse {
    pub authenticated: bool,
    #[serde(default)]
    pub is_admin: bool,
    pub user: Option<SessionUser>,
}

#[cfg(any(target_arch = "wasm32", all(test, feature = "server")))]
#[derive(Debug, Clone, Deserialize)]
struct CsrfTokenResponse {
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubStats {
    pub stars: i64,
    pub forks: i64,
    pub contributors: i64,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize)]
struct GithubRepo {
    stargazers_count: i64,
    forks_count: i64,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Deserialize)]
struct GithubContributor {
    login: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StarActionResponse {
    pub success: bool,
    pub starred: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[cfg(target_arch = "wasm32")]
fn map_server_fn_error(err: ServerFnError) -> ApiError {
    ApiError::new(err.to_string())
}

#[cfg(feature = "server")]
pub fn index_url(base: &str) -> String {
    api_url(base, "index")
}

#[cfg(feature = "server")]
pub fn featured_url(base: &str) -> String {
    api_url(base, "featured")
}

#[cfg(feature = "server")]
pub fn landing_url(base: &str) -> String {
    api_url(base, "landing")
}

#[cfg(feature = "server")]
pub fn module_detail_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}"))
}

#[cfg(feature = "server")]
pub fn module_versions_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}/versions"))
}

#[cfg(feature = "server")]
pub fn module_related_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}/related"))
}

#[cfg(feature = "server")]
pub fn module_reviews_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}/reviews"))
}

#[cfg(feature = "server")]
pub fn module_screenshots_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}/screenshots"))
}

#[cfg(feature = "server")]
pub fn module_screenshot_url(base: &str, uuid: &str, id: i64) -> String {
    api_url(base, &format!("modules/{uuid}/screenshots/{id}"))
}

#[cfg(feature = "server")]
pub fn user_profile_url(base: &str, username: &str) -> String {
    api_url(base, &format!("users/{username}"))
}

#[cfg(feature = "server")]
pub fn user_modules_url(base: &str, username: &str) -> String {
    api_url(base, &format!("users/{username}/modules"))
}

#[cfg(feature = "server")]
pub fn user_collections_url(base: &str, username: &str) -> String {
    api_url(base, &format!("users/{username}/collections"))
}

#[cfg(feature = "server")]
pub fn collections_url(base: &str) -> String {
    api_url(base, "collections")
}

#[cfg(feature = "server")]
pub fn collection_url(base: &str, id: i64) -> String {
    api_url(base, &format!("collections/{id}"))
}

#[cfg(feature = "server")]
pub fn collection_modules_url(base: &str, id: i64) -> String {
    api_url(base, &format!("collections/{id}/modules"))
}

#[cfg(feature = "server")]
pub fn collection_module_url(base: &str, id: i64, uuid: &str) -> String {
    api_url(base, &format!("collections/{id}/modules/{uuid}"))
}

#[cfg(feature = "server")]
pub fn stars_url(base: &str) -> String {
    api_url(base, "users/me/stars")
}

#[cfg(feature = "server")]
pub fn stars_sync_url(base: &str) -> String {
    api_url(base, "stars/sync")
}

#[cfg(feature = "server")]
pub fn star_status_url(base: &str, uuid: &str) -> String {
    api_url(base, &format!("modules/{uuid}/star"))
}

#[cfg(feature = "server")]
pub fn notifications_preferences_url(base: &str) -> String {
    api_url(base, "notifications/preferences")
}

#[cfg(feature = "server")]
pub fn notifications_url(base: &str) -> String {
    api_url(base, "notifications")
}

#[cfg(feature = "server")]
pub fn notifications_unread_url(base: &str) -> String {
    api_url(base, "notifications/unread-count")
}

#[cfg(feature = "server")]
pub fn notifications_mark_all_read_url(base: &str) -> String {
    api_url(base, "notifications/mark-all-read")
}

#[cfg(feature = "server")]
pub fn notifications_stream_url(base: &str) -> String {
    api_url(base, "notifications/stream")
}

#[cfg(feature = "server")]
pub fn notification_mark_read_url(base: &str, id: i64) -> String {
    api_url(base, &format!("notifications/{id}/read"))
}

#[cfg(feature = "server")]
pub fn admin_stats_url(base: &str) -> String {
    api_url(base, "admin/stats")
}

#[cfg(feature = "server")]
pub fn admin_submissions_url(base: &str) -> String {
    api_url(base, "admin/submissions")
}

#[cfg(feature = "server")]
pub fn admin_submission_approve_url(base: &str, id: i64) -> String {
    api_url(base, &format!("admin/submissions/{id}/approve"))
}

#[cfg(feature = "server")]
pub fn admin_submission_reject_url(base: &str, id: i64) -> String {
    api_url(base, &format!("admin/submissions/{id}/reject"))
}

#[cfg(feature = "server")]
pub fn admin_user_verify_url(base: &str, id: i64) -> String {
    api_url(base, &format!("admin/users/{id}/verify"))
}

#[cfg(any(feature = "server", target_arch = "wasm32"))]
fn parse_json<T: DeserializeOwned>(body: &str, context: &str) -> Result<T, ApiError> {
    serde_json::from_str(body).map_err(|err| ApiError::new(format!("{context}: {err}")))
}

#[cfg(all(test, feature = "server"))]
pub fn parse_registry_index(body: &str) -> Result<RegistryIndex, ApiError> {
    parse_json(body, "invalid registry index")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_featured_response(body: &str) -> Result<FeaturedModulesResponse, ApiError> {
    parse_json(body, "invalid featured response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_landing_response(body: &str) -> Result<Versioned<LandingData>, ApiError> {
    parse_json(body, "invalid landing response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_module_detail_response(body: &str) -> Result<RegistryModule, ApiError> {
    parse_json(body, "invalid module detail response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_module_versions_response(body: &str) -> Result<Versioned<VersionsResponse>, ApiError> {
    parse_json(body, "invalid versions response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_related_modules_response(body: &str) -> Result<Versioned<ModulesResponse>, ApiError> {
    parse_json(body, "invalid related modules response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_reviews_response(body: &str) -> Result<Versioned<ReviewsResponse>, ApiError> {
    parse_json(body, "invalid reviews response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_screenshots_response(body: &str) -> Result<Versioned<ScreenshotsResponse>, ApiError> {
    parse_json(body, "invalid screenshots response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_user_profile_response(body: &str) -> Result<UserProfile, ApiError> {
    parse_json(body, "invalid user profile response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_stars_response(body: &str) -> Result<Versioned<StarsResponse>, ApiError> {
    parse_json(body, "invalid stars response")
}

#[cfg(all(test, feature = "server"))]
pub fn parse_session_response(body: &str) -> Result<SessionResponse, ApiError> {
    parse_json(body, "invalid session response")
}

#[cfg(all(test, feature = "server"))]
fn parse_csrf_token_response(body: &str) -> Result<CsrfTokenResponse, ApiError> {
    parse_json(body, "invalid csrf response")
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json_client<T: DeserializeOwned>(url: &str, context: &str) -> Result<T, ApiError> {
    fetch_json_client_with_method(url, "GET", None, context).await
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json_client_with_method_and_headers<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<String>,
    context: &str,
    csrf_token: Option<String>,
) -> Result<T, ApiError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_credentials(RequestCredentials::Include);

    if body.is_some() || csrf_token.is_some() {
        let headers =
            Headers::new().map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
        if let Some(token) = csrf_token {
            headers
                .append("x-csrf-token", &token)
                .map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
        }
        if let Some(body) = body {
            headers
                .append("Content-Type", "application/json")
                .map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
            opts.set_body(&JsValue::from_str(&body));
        }
        opts.set_headers(&headers);
    }

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|err| ApiError::new(format!("request init failed: {err:?}")))?;
    let window = web_sys::window().ok_or_else(|| ApiError::new("window unavailable"))?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| ApiError::new(format!("fetch failed: {err:?}")))?;
    let response: Response = response
        .dyn_into()
        .map_err(|err| ApiError::new(format!("invalid response: {err:?}")))?;

    if !response.ok() {
        return Err(ApiError::new(format!(
            "unexpected status: {}",
            response.status()
        )));
    }

    let body = JsFuture::from(
        response
            .text()
            .map_err(|err| ApiError::new(format!("response read failed: {err:?}")))?,
    )
    .await
    .map_err(|err| ApiError::new(format!("response read failed: {err:?}")))?;
    let body = body
        .as_string()
        .ok_or_else(|| ApiError::new("response body was not text"))?;

    parse_json(&body, context)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json_client_with_method<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<String>,
    context: &str,
) -> Result<T, ApiError> {
    fetch_json_client_with_method_and_headers(url, method, body, context, None).await
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json_client_with_csrf<T: DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<String>,
    context: &str,
) -> Result<T, ApiError> {
    let csrf = fetch_csrf_token().await?;
    fetch_json_client_with_method_and_headers(url, method, body, context, Some(csrf)).await
}

#[cfg(target_arch = "wasm32")]
async fn fetch_ok_client_with_csrf(
    url: &str,
    method: &str,
    body: Option<String>,
    context: &str,
) -> Result<(), ApiError> {
    let csrf = fetch_csrf_token().await?;
    fetch_ok_client_with_method_and_headers(url, method, body, context, Some(csrf)).await
}

#[cfg(target_arch = "wasm32")]
async fn fetch_ok_client_with_method_and_headers(
    url: &str,
    method: &str,
    body: Option<String>,
    context: &str,
    csrf_token: Option<String>,
) -> Result<(), ApiError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Headers, Request, RequestCredentials, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_credentials(RequestCredentials::Include);
    if body.is_some() || csrf_token.is_some() {
        let headers =
            Headers::new().map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
        if let Some(token) = csrf_token {
            headers
                .append("x-csrf-token", &token)
                .map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
        }
        if let Some(body) = body {
            headers
                .append("Content-Type", "application/json")
                .map_err(|err| ApiError::new(format!("headers init failed: {err:?}")))?;
            opts.set_body(&JsValue::from_str(&body));
        }
        opts.set_headers(&headers);
    }

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|err| ApiError::new(format!("request init failed: {err:?}")))?;
    let window = web_sys::window().ok_or_else(|| ApiError::new("window unavailable"))?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|err| ApiError::new(format!("fetch failed: {err:?}")))?;
    let response: Response = response
        .dyn_into()
        .map_err(|err| ApiError::new(format!("invalid response: {err:?}")))?;

    if !response.ok() {
        return Err(ApiError::new(format!("{context}: {}", response.status())));
    }

    Ok(())
}

#[cfg(feature = "server")]
async fn fetch_json_server<T: DeserializeOwned>(url: &str, context: &str) -> Result<T, ApiError> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| ApiError::new(format!("request failed: {err}")))?;

    let status = response.status();
    if status.as_u16() >= 400 {
        return Err(ApiError::new(format!("unexpected status: {status}")));
    }

    let body = response
        .text()
        .await
        .map_err(|err| ApiError::new(format!("response read failed: {err}")))?;

    parse_json(&body, context)
}

#[cfg(feature = "server")]
async fn fetch_github_repo(
    client: &reqwest::Client,
    repo: &str,
) -> Result<Option<GithubRepo>, ApiError> {
    let url = format!("{GITHUB_API_BASE}/{repo}");
    let response = match client
        .get(url)
        .header(reqwest::header::ACCEPT, GITHUB_API_ACCEPT)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let repo = response
        .json::<GithubRepo>()
        .await
        .map_err(|err| ApiError::new(format!("github repo parse failed: {err}")))?;
    Ok(Some(repo))
}

#[cfg(feature = "server")]
async fn fetch_github_contributors(
    client: &reqwest::Client,
    repo: &str,
) -> Result<Vec<GithubContributor>, ApiError> {
    let url = format!("{GITHUB_API_BASE}/{repo}/contributors");
    let response = match client
        .get(url)
        .header(reqwest::header::ACCEPT, GITHUB_API_ACCEPT)
        .header(reqwest::header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Ok(Vec::new()),
    };

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    response
        .json::<Vec<GithubContributor>>()
        .await
        .map_err(|err| ApiError::new(format!("github contributors parse failed: {err}")))
}

#[cfg(feature = "server")]
fn combine_github_stats(
    web_repo: Option<GithubRepo>,
    app_repo: Option<GithubRepo>,
    web_contributors: Vec<GithubContributor>,
    app_contributors: Vec<GithubContributor>,
) -> GithubStats {
    let stars = web_repo
        .as_ref()
        .map(|repo| repo.stargazers_count)
        .unwrap_or(0)
        + app_repo
            .as_ref()
            .map(|repo| repo.stargazers_count)
            .unwrap_or(0);
    let forks = web_repo.as_ref().map(|repo| repo.forks_count).unwrap_or(0)
        + app_repo.as_ref().map(|repo| repo.forks_count).unwrap_or(0);
    let mut contributors = std::collections::BTreeSet::new();
    contributors.extend(web_contributors.into_iter().map(|contrib| contrib.login));
    contributors.extend(app_contributors.into_iter().map(|contrib| contrib.login));

    GithubStats {
        stars,
        forks,
        contributors: contributors.len() as i64,
    }
}

#[get("/api/v1/github/stats")]
async fn github_stats_server() -> Result<GithubStats, ServerFnError> {
    let client = reqwest::Client::new();
    let web_repo = fetch_github_repo(&client, "barforge-web")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let app_repo = fetch_github_repo(&client, "barforge-app")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let web_contributors = fetch_github_contributors(&client, "barforge-web")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let app_contributors = fetch_github_contributors(&client, "barforge-app")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(combine_github_stats(
        web_repo,
        app_repo,
        web_contributors,
        app_contributors,
    ))
}

#[get("/api/v1/index")]
async fn registry_index_server() -> Result<RegistryIndex, ServerFnError> {
    let url = index_url(api_base_url());
    let payload = fetch_json_server(&url, "invalid registry index")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/featured")]
async fn featured_modules_server() -> Result<FeaturedModulesResponse, ServerFnError> {
    let url = featured_url(api_base_url());
    let payload = fetch_json_server(&url, "invalid featured response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/landing")]
async fn landing_server() -> Result<Versioned<LandingData>, ServerFnError> {
    let url = landing_url(api_base_url());
    let payload = fetch_json_server(&url, "invalid landing response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/modules/{uuid}")]
async fn module_detail_server(uuid: String) -> Result<RegistryModule, ServerFnError> {
    let url = module_detail_url(api_base_url(), &uuid);
    let payload = fetch_json_server(&url, "invalid module detail response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/modules/{uuid}/versions")]
async fn module_versions_server(
    uuid: String,
) -> Result<Versioned<VersionsResponse>, ServerFnError> {
    let url = module_versions_url(api_base_url(), &uuid);
    let payload = fetch_json_server(&url, "invalid versions response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/modules/{uuid}/related")]
async fn module_related_server(uuid: String) -> Result<Versioned<ModulesResponse>, ServerFnError> {
    let url = module_related_url(api_base_url(), &uuid);
    let payload = fetch_json_server(&url, "invalid related modules response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/modules/{uuid}/reviews")]
async fn module_reviews_server(uuid: String) -> Result<Versioned<ReviewsResponse>, ServerFnError> {
    let url = module_reviews_url(api_base_url(), &uuid);
    let payload = fetch_json_server(&url, "invalid reviews response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/modules/{uuid}/screenshots")]
async fn module_screenshots_server(
    uuid: String,
) -> Result<Versioned<ScreenshotsResponse>, ServerFnError> {
    let url = module_screenshots_url(api_base_url(), &uuid);
    let payload = fetch_json_server(&url, "invalid screenshots response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/users/{username}")]
async fn user_profile_server(username: String) -> Result<UserProfile, ServerFnError> {
    let url = user_profile_url(api_base_url(), &username);
    let payload = fetch_json_server(&url, "invalid user profile response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/users/{username}/modules")]
async fn user_modules_server(username: String) -> Result<ModulesResponse, ServerFnError> {
    let url = user_modules_url(api_base_url(), &username);
    let payload = fetch_json_server(&url, "invalid user modules response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[get("/api/v1/users/{username}/collections")]
async fn user_collections_server(username: String) -> Result<CollectionsResponse, ServerFnError> {
    let url = format!(
        "{}?visibility=public",
        user_collections_url(api_base_url(), &username)
    );
    let payload = fetch_json_server(&url, "invalid user collections response")
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(payload)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_registry_index() -> Result<RegistryIndex, ApiError> {
    registry_index_server().await.map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_featured_modules() -> Result<FeaturedModulesResponse, ApiError> {
    featured_modules_server().await.map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_landing_data() -> Result<LandingData, ApiError> {
    let payload = landing_server().await.map_err(map_server_fn_error)?;
    Ok(payload.payload)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_module_detail(uuid: &str) -> Result<RegistryModule, ApiError> {
    module_detail_server(uuid.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_module_versions(uuid: &str) -> Result<Versioned<VersionsResponse>, ApiError> {
    module_versions_server(uuid.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_related_modules(uuid: &str) -> Result<Versioned<ModulesResponse>, ApiError> {
    module_related_server(uuid.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_module_reviews(uuid: &str) -> Result<Versioned<ReviewsResponse>, ApiError> {
    module_reviews_server(uuid.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_module_screenshots(
    uuid: &str,
) -> Result<Versioned<ScreenshotsResponse>, ApiError> {
    module_screenshots_server(uuid.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_user_profile(username: &str) -> Result<UserProfile, ApiError> {
    user_profile_server(username.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_user_profile_me() -> Result<UserProfile, ApiError> {
    fetch_json_client("/api/users/me", "invalid user profile response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_modules_mine() -> Result<ModulesResponse, ApiError> {
    let payload: Versioned<ModulesResponse> =
        fetch_json_client("/api/modules/mine", "invalid modules response").await?;
    Ok(payload.payload)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_user_modules(username: &str) -> Result<ModulesResponse, ApiError> {
    user_modules_server(username.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_user_collections(username: &str) -> Result<CollectionsResponse, ApiError> {
    user_collections_server(username.to_string())
        .await
        .map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_session_status() -> Result<SessionResponse, ApiError> {
    fetch_json_client("/api/session", "invalid session response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_account() -> Result<(), ApiError> {
    fetch_ok_client_with_csrf("/api/users/me", "DELETE", None, "delete account failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn logout() -> Result<(), ApiError> {
    fetch_ok_client_with_csrf("/auth/logout", "POST", None, "logout failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_github_stats() -> Result<GithubStats, ApiError> {
    github_stats_server().await.map_err(map_server_fn_error)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_notification_preferences() -> Result<Versioned<NotificationPreferences>, ApiError>
{
    fetch_json_client(
        "/api/notifications/preferences",
        "invalid notification preferences response",
    )
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn update_notification_preferences(
    payload: &NotificationPreferences,
) -> Result<Versioned<NotificationPreferences>, ApiError> {
    let body = serde_json::to_string(payload)
        .map_err(|err| ApiError::new(format!("notification preferences payload failed: {err}")))?;
    fetch_json_client_with_method(
        "/api/notifications/preferences",
        "PATCH",
        Some(body),
        "invalid notification preferences response",
    )
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_collections() -> Result<Versioned<CollectionsResponse>, ApiError> {
    fetch_json_client("/api/collections", "invalid collections response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_collection_detail(
    id: i64,
) -> Result<Versioned<CollectionDetailResponse>, ApiError> {
    let url = format!("/api/collections/{id}");
    fetch_json_client(&url, "invalid collection detail response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn remove_collection_module(id: i64, uuid: &str) -> Result<(), ApiError> {
    let url = format!("/api/collections/{id}/modules/{uuid}");
    fetch_ok_client_with_csrf(&url, "DELETE", None, "remove collection module failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn create_collection(body: String) -> Result<(), ApiError> {
    fetch_ok_client_with_csrf(
        "/api/collections",
        "POST",
        Some(body),
        "create collection failed",
    )
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn update_collection(id: i64, body: String) -> Result<(), ApiError> {
    let url = format!("/api/collections/{id}");
    fetch_ok_client_with_csrf(&url, "PATCH", Some(body), "update collection failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_collection(id: i64) -> Result<(), ApiError> {
    let url = format!("/api/collections/{id}");
    fetch_ok_client_with_csrf(&url, "DELETE", None, "delete collection failed").await
}

#[cfg(any(all(test, feature = "server"), target_arch = "wasm32"))]
pub async fn fetch_notifications(
    limit: i64,
    offset: i64,
) -> Result<NotificationsResponse, ApiError> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("/api/notifications?limit={limit}&offset={offset}");
        fetch_json_client(&url, "invalid notifications response").await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (limit, offset);
        Ok(NotificationsResponse {
            notifications: Vec::new(),
            total: 0,
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_admin_stats() -> Result<Versioned<AdminStats>, ApiError> {
    fetch_json_client("/api/admin/stats", "invalid admin stats response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_admin_submissions() -> Result<Versioned<SubmissionsResponse>, ApiError> {
    fetch_json_client(
        "/api/admin/submissions",
        "invalid admin submissions response",
    )
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn approve_submission(id: i64) -> Result<(), ApiError> {
    let url = format!("/api/admin/submissions/{id}/approve");
    fetch_ok_client_with_csrf(&url, "POST", None, "approve submission failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn reject_submission(id: i64, reason: &str) -> Result<(), ApiError> {
    let body = serde_json::to_string(&RejectRequest {
        reason: reason.to_string(),
    })
    .map_err(|err| ApiError::new(format!("reject request failed: {err}")))?;
    let url = format!("/api/admin/submissions/{id}/reject");
    fetch_ok_client_with_csrf(&url, "POST", Some(body), "reject submission failed").await
}

#[cfg(target_arch = "wasm32")]
pub async fn verify_user(id: i64) -> Result<Versioned<VerifyResponse>, ApiError> {
    let url = format!("/api/admin/users/{id}/verify");
    fetch_json_client_with_csrf(&url, "POST", None, "invalid verify response").await
}

#[cfg(all(test, feature = "server"))]
pub async fn fetch_unread_count() -> Result<UnreadCountResponse, ApiError> {
    #[cfg(target_arch = "wasm32")]
    {
        fetch_json_client(
            "/api/notifications/unread-count",
            "invalid notifications unread response",
        )
        .await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(UnreadCountResponse { unread_count: 0 })
    }
}

#[cfg(any(all(test, feature = "server"), target_arch = "wasm32"))]
pub async fn mark_notification_read(id: i64) -> Result<MarkReadResponse, ApiError> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("/api/notifications/{id}/read");
        fetch_json_client_with_method(&url, "PATCH", None, "invalid notification read response")
            .await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Ok(MarkReadResponse { success: true })
    }
}

#[cfg(any(all(test, feature = "server"), target_arch = "wasm32"))]
pub async fn mark_all_notifications_read() -> Result<MarkAllReadResponse, ApiError> {
    #[cfg(target_arch = "wasm32")]
    {
        fetch_json_client_with_method(
            "/api/notifications/mark-all-read",
            "POST",
            None,
            "invalid notifications mark all response",
        )
        .await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(MarkAllReadResponse { marked_count: 0 })
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_stars() -> Result<Versioned<StarsResponse>, ApiError> {
    fetch_json_client("/api/stars", "invalid stars response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn star_module(uuid: &str) -> Result<StarActionResponse, ApiError> {
    let body = serde_json::to_string(&StarRequest {
        is_public: Some(true),
    })
    .map_err(|err| ApiError::new(format!("star request failed: {err}")))?;
    let url = format!("/api/modules/{uuid}/star");
    fetch_json_client_with_method(&url, "POST", Some(body), "invalid star response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn unstar_module(uuid: &str) -> Result<StarActionResponse, ApiError> {
    let url = format!("/api/modules/{uuid}/star");
    fetch_json_client_with_method(&url, "DELETE", None, "invalid star response").await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_csrf_token() -> Result<String, ApiError> {
    let response: CsrfTokenResponse =
        fetch_json_client("/api/csrf-token", "invalid csrf response").await?;
    if response.token.is_empty() {
        return Err(ApiError::new("invalid csrf response"));
    }
    Ok(response.token)
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::{
        DEFAULT_API_BASE_URL, GithubContributor, GithubRepo, GithubStats, LandingData,
        admin_stats_url, admin_submission_approve_url, admin_submission_reject_url,
        admin_submissions_url, admin_user_verify_url, api_base_url, collection_module_url,
        collection_modules_url, collection_url, collections_url, combine_github_stats,
        featured_url, fetch_notifications, fetch_unread_count, index_url,
        mark_all_notifications_read, mark_notification_read, module_detail_url, module_related_url,
        module_reviews_url, module_screenshot_url, module_screenshots_url, module_versions_url,
        notification_mark_read_url, notifications_mark_all_read_url, notifications_preferences_url,
        notifications_stream_url, notifications_unread_url, notifications_url,
        parse_csrf_token_response, parse_featured_response, parse_landing_response,
        parse_module_detail_response, parse_module_versions_response, parse_registry_index,
        parse_related_modules_response, parse_reviews_response, parse_screenshots_response,
        parse_session_response, parse_stars_response, parse_user_profile_response, star_status_url,
        stars_sync_url, stars_url, user_collections_url, user_modules_url, user_profile_url,
    };
    use barforge_types::{
        CategoryInfo, FeaturedModulesResponse, LandingInstallMethod, ModuleCategory,
        ModulesResponse, PublicStats, RegistryIndex, RegistryModule, Review, ReviewUser,
        ReviewsResponse, Screenshot, ScreenshotsResponse, StarredModule, StarsResponse,
        UserProfile, UserRole, Versioned, VersionsResponse,
    };
    use std::collections::BTreeMap;

    #[test]
    fn index_url_trims_trailing_slash() {
        let base = "https://api.barforge.dev/";
        let url = index_url(base);

        assert_eq!(url, "https://api.barforge.dev/api/v1/index");
    }

    #[test]
    fn index_url_uses_default_base() {
        let url = index_url(DEFAULT_API_BASE_URL);

        assert_eq!(url, "https://api.barforge.dev/api/v1/index");
    }

    #[test]
    fn api_base_url_defaults_to_constant() {
        assert_eq!(api_base_url(), DEFAULT_API_BASE_URL);
    }

    #[test]
    fn combine_github_stats_sums_and_dedupes() {
        let web_repo = GithubRepo {
            stargazers_count: 12,
            forks_count: 4,
        };
        let app_repo = GithubRepo {
            stargazers_count: 8,
            forks_count: 3,
        };
        let web_contributors = vec![
            GithubContributor {
                login: "alice".to_string(),
            },
            GithubContributor {
                login: "bob".to_string(),
            },
        ];
        let app_contributors = vec![
            GithubContributor {
                login: "bob".to_string(),
            },
            GithubContributor {
                login: "cora".to_string(),
            },
        ];

        let stats = combine_github_stats(
            Some(web_repo),
            Some(app_repo),
            web_contributors,
            app_contributors,
        );

        assert_eq!(
            stats,
            GithubStats {
                stars: 20,
                forks: 7,
                contributors: 3,
            }
        );
    }

    #[test]
    fn parse_registry_index_roundtrip() {
        let mut categories = BTreeMap::new();
        categories.insert(
            "system".to_string(),
            CategoryInfo {
                id: Some("system".to_string()),
                name: "System".to_string(),
                icon: "icon-cpu".to_string(),
            },
        );
        let index = RegistryIndex {
            version: 1,
            modules: vec![sample_registry_module()],
            categories,
        };
        let json = serde_json::to_string(&index).expect("serialize registry index");

        let parsed = parse_registry_index(&json).expect("parse registry index");

        assert_eq!(parsed, index);
    }

    #[test]
    fn featured_url_trims_trailing_slash() {
        let base = "https://api.barforge.dev/";
        let url = featured_url(base);

        assert_eq!(url, "https://api.barforge.dev/api/v1/featured");
    }

    #[test]
    fn parse_featured_response_roundtrip() {
        let payload = FeaturedModulesResponse {
            version: 1,
            featured: vec![sample_registry_module()],
            popular: vec![sample_registry_module()],
            recent: vec![sample_registry_module()],
        };
        let json = serde_json::to_string(&payload).expect("serialize featured response");

        let parsed = parse_featured_response(&json).expect("parse featured response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_landing_response_roundtrip() {
        let json = r#"{"version":1,"stats":{"total_modules":12,"total_downloads":3400,"total_authors":4},"install_methods":[{"id":"aur","label":"AUR","description":"Arch User Repository","commands":["yay -S barforge"]}]}"#;

        let parsed = parse_landing_response(json).expect("landing response");

        assert_eq!(
            parsed,
            Versioned {
                version: 1,
                payload: LandingData {
                    stats: PublicStats {
                        total_modules: 12,
                        total_downloads: 3400,
                        total_authors: 4,
                    },
                    install_methods: vec![LandingInstallMethod {
                        id: "aur".to_string(),
                        label: "AUR".to_string(),
                        description: "Arch User Repository".to_string(),
                        commands: vec!["yay -S barforge".to_string()],
                    }],
                },
            }
        );
    }

    #[test]
    fn module_detail_url_uses_uuid() {
        let url = module_detail_url("https://api.barforge.dev", "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge"
        );
    }

    #[test]
    fn module_versions_url_uses_uuid() {
        let url = module_versions_url("https://api.barforge.dev", "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge/versions"
        );
    }

    #[test]
    fn module_related_url_uses_uuid() {
        let url = module_related_url("https://api.barforge.dev", "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge/related"
        );
    }

    #[test]
    fn module_reviews_url_uses_uuid() {
        let url = module_reviews_url("https://api.barforge.dev", "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge/reviews"
        );
    }

    #[test]
    fn module_screenshots_url_uses_uuid() {
        let url = module_screenshots_url("https://api.barforge.dev", "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge/screenshots"
        );
    }

    #[test]
    fn module_screenshot_url_uses_uuid_and_id() {
        let url = module_screenshot_url("https://api.barforge.dev", "weather-wttr@barforge", 42);

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/modules/weather-wttr@barforge/screenshots/42"
        );
    }

    #[test]
    fn user_profile_url_uses_username() {
        let url = user_profile_url("https://api.barforge.dev", "barforge");

        assert_eq!(url, "https://api.barforge.dev/api/v1/users/barforge");
    }

    #[test]
    fn user_modules_url_uses_username() {
        let url = user_modules_url("https://api.barforge.dev", "barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/users/barforge/modules"
        );
    }

    #[test]
    fn user_collections_url_uses_username() {
        let url = user_collections_url("https://api.barforge.dev", "barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/users/barforge/collections"
        );
    }

    #[test]
    fn collections_url_uses_base() {
        let url = collections_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/collections");
    }

    #[test]
    fn collection_url_uses_id() {
        let url = collection_url("https://api.barforge.dev", 42);

        assert_eq!(url, "https://api.barforge.dev/api/v1/collections/42");
    }

    #[test]
    fn collection_modules_url_uses_id() {
        let url = collection_modules_url("https://api.barforge.dev", 42);

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/collections/42/modules"
        );
    }

    #[test]
    fn collection_module_url_uses_id_and_uuid() {
        let url = collection_module_url("https://api.barforge.dev", 42, "weather-wttr@barforge");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/collections/42/modules/weather-wttr@barforge"
        );
    }

    #[test]
    fn stars_url_uses_base() {
        let url = stars_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/users/me/stars");
    }

    #[test]
    fn stars_sync_url_uses_base() {
        let url = stars_sync_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/stars/sync");
    }

    #[test]
    fn star_status_url_uses_uuid() {
        let url = star_status_url("https://api.barforge.dev", "module-1");

        assert_eq!(url, "https://api.barforge.dev/api/v1/modules/module-1/star");
    }

    #[test]
    fn notifications_preferences_url_uses_base() {
        let url = notifications_preferences_url("https://api.barforge.dev");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/notifications/preferences"
        );
    }

    #[test]
    fn notifications_url_uses_base() {
        let url = notifications_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/notifications");
    }

    #[test]
    fn notifications_unread_url_uses_base() {
        let url = notifications_unread_url("https://api.barforge.dev");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/notifications/unread-count"
        );
    }

    #[test]
    fn notifications_mark_all_read_url_uses_base() {
        let url = notifications_mark_all_read_url("https://api.barforge.dev");

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/notifications/mark-all-read"
        );
    }

    #[test]
    fn notifications_stream_url_uses_base() {
        let url = notifications_stream_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/notifications/stream");
    }

    #[test]
    fn notification_mark_read_url_uses_id() {
        let url = notification_mark_read_url("https://api.barforge.dev", 42);

        assert_eq!(url, "https://api.barforge.dev/api/v1/notifications/42/read");
    }

    #[test]
    fn admin_stats_url_uses_base() {
        let url = admin_stats_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/admin/stats");
    }

    #[test]
    fn admin_submissions_url_uses_base() {
        let url = admin_submissions_url("https://api.barforge.dev");

        assert_eq!(url, "https://api.barforge.dev/api/v1/admin/submissions");
    }

    #[test]
    fn admin_submission_approve_url_uses_id() {
        let url = admin_submission_approve_url("https://api.barforge.dev", 42);

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/admin/submissions/42/approve"
        );
    }

    #[test]
    fn admin_submission_reject_url_uses_id() {
        let url = admin_submission_reject_url("https://api.barforge.dev", 42);

        assert_eq!(
            url,
            "https://api.barforge.dev/api/v1/admin/submissions/42/reject"
        );
    }

    #[test]
    fn admin_user_verify_url_uses_id() {
        let url = admin_user_verify_url("https://api.barforge.dev", 42);

        assert_eq!(url, "https://api.barforge.dev/api/v1/admin/users/42/verify");
    }

    #[tokio::test]
    async fn fetch_notifications_defaults_to_empty() {
        let payload = fetch_notifications(20, 0)
            .await
            .expect("notifications payload");

        assert!(payload.notifications.is_empty());
        assert_eq!(payload.total, 0);
    }

    #[tokio::test]
    async fn fetch_unread_count_defaults_to_zero() {
        let payload = fetch_unread_count().await.expect("unread count payload");

        assert_eq!(payload.unread_count, 0);
    }

    #[tokio::test]
    async fn mark_notification_read_defaults_to_success() {
        let payload = mark_notification_read(42).await.expect("mark read payload");

        assert!(payload.success);
    }

    #[tokio::test]
    async fn mark_all_notifications_read_defaults_to_zero() {
        let payload = mark_all_notifications_read()
            .await
            .expect("mark all payload");

        assert_eq!(payload.marked_count, 0);
    }

    #[test]
    fn parse_module_detail_response_roundtrip() {
        let payload = sample_registry_module();
        let json = serde_json::to_string(&payload).expect("serialize module detail");

        let parsed = parse_module_detail_response(&json).expect("parse module detail");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_module_versions_response_roundtrip() {
        let versions = VersionsResponse {
            versions: vec![sample_version_entry()],
            total: 1,
        };
        let payload = Versioned {
            version: 1,
            payload: versions,
        };
        let json = serde_json::to_string(&payload).expect("serialize versions response");

        let parsed = parse_module_versions_response(&json).expect("parse versions response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_related_modules_response_roundtrip() {
        let payload = Versioned {
            version: 1,
            payload: ModulesResponse {
                modules: vec![sample_registry_module()],
                total: 1,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize related response");

        let parsed = parse_related_modules_response(&json).expect("parse related response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_reviews_response_roundtrip() {
        let payload = Versioned {
            version: 1,
            payload: ReviewsResponse {
                reviews: vec![sample_review()],
                total: 1,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize reviews response");

        let parsed = parse_reviews_response(&json).expect("parse reviews response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_screenshots_response_roundtrip() {
        let payload = Versioned {
            version: 1,
            payload: ScreenshotsResponse {
                screenshots: vec![sample_screenshot()],
                total: 1,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize screenshots response");

        let parsed = parse_screenshots_response(&json).expect("parse screenshots response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_user_profile_response_roundtrip() {
        let payload = UserProfile {
            id: 42,
            username: "barforge".to_string(),
            display_name: Some("Barforge".to_string()),
            avatar_url: None,
            bio: None,
            website_url: None,
            github_url: Some("https://github.com/barforge".to_string()),
            twitter_url: None,
            bluesky_url: None,
            discord_url: None,
            sponsor_url: None,
            verified_author: true,
            role: UserRole::User,
            module_count: 3,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).expect("serialize user profile");

        let parsed = parse_user_profile_response(&json).expect("parse user profile");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_stars_response_roundtrip() {
        let payload = Versioned {
            version: 1,
            payload: StarsResponse {
                modules: vec![sample_starred_module()],
                total: 1,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize stars response");

        let parsed = parse_stars_response(&json).expect("parse stars response");

        assert_eq!(parsed, payload);
    }

    #[test]
    fn parse_session_response_roundtrip() {
        let payload =
            r#"{"authenticated":true,"user":{"login":"octo","email":"octo@example.com"}}"#;

        let parsed = parse_session_response(payload).expect("parse session");

        assert!(parsed.authenticated);
        assert!(!parsed.is_admin);
        assert_eq!(
            parsed.user.as_ref().map(|user| user.login.as_str()),
            Some("octo")
        );
        assert_eq!(
            parsed.user.as_ref().map(|user| user.email.as_deref()),
            Some(Some("octo@example.com"))
        );
    }

    #[test]
    fn parse_csrf_token_response_roundtrip() {
        let payload = r#"{"token":"csrf-token"}"#;

        let parsed = parse_csrf_token_response(payload).expect("parse csrf token");

        assert_eq!(parsed.token, "csrf-token");
    }

    fn sample_registry_module() -> RegistryModule {
        RegistryModule {
            uuid: "clock-time@barforge".to_string(),
            name: "Clock".to_string(),
            description: "Minimal clock module".to_string(),
            author: "Barforge".to_string(),
            category: ModuleCategory::Time,
            icon: None,
            screenshot: None,
            repo_url: "https://github.com/barforge/clock-time".to_string(),
            downloads: 1200,
            version: Some("1.4.0".to_string()),
            last_updated: Some("2025-12-20T12:00:00Z".to_string()),
            rating: Some(4.8),
            verified_author: true,
            tags: vec!["clock".to_string()],
            checksum: None,
            license: Some("MIT".to_string()),
        }
    }

    fn sample_version_entry() -> barforge_types::VersionHistoryEntry {
        barforge_types::VersionHistoryEntry {
            version: "1.2.0".to_string(),
            changelog: Some("Added new layout.".to_string()),
            downloads: 120,
            published_at: "2025-12-01T12:00:00Z".to_string(),
        }
    }

    fn sample_review() -> Review {
        Review {
            id: 1,
            rating: 5,
            title: Some("Great module".to_string()),
            body: Some("Crisp layout and fast.".to_string()),
            helpful_count: 12,
            user: ReviewUser {
                username: "barforge".to_string(),
                avatar_url: None,
            },
            created_at: "2025-12-12T12:00:00Z".to_string(),
            updated_at: None,
        }
    }

    fn sample_screenshot() -> Screenshot {
        Screenshot {
            id: 1,
            r2_key: "screenshots/weather/wttr.png".to_string(),
            alt_text: Some("Weather module".to_string()),
            position: 1,
            created_at: "2025-12-12T12:00:00Z".to_string(),
        }
    }

    fn sample_starred_module() -> StarredModule {
        StarredModule {
            module: sample_registry_module(),
            starred_at: "2025-12-22T12:00:00Z".to_string(),
        }
    }
}
