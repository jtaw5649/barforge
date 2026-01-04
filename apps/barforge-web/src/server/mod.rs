use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{Next, from_fn, from_fn_with_state},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post},
};
use axum_tower_sessions_csrf::CsrfMiddleware;
use oauth2::url::Url;
use reqwest::Client;
use reqwest::redirect::Policy;
use std::env;
use std::sync::OnceLock;
use time::Duration;
use tower_sessions::cookie::SameSite;
use tower_sessions::session_store::SessionStore;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_sqlx_store::{SqliteStore, sqlx::SqlitePool};
use tracing_subscriber::EnvFilter;

use crate::api;
use crate::auth_redirect;

pub mod admin;
pub mod auth;
pub mod collections;
pub mod modules;
pub mod notifications;
pub mod stars;
pub mod users;

const GITHUB_STATS_CACHE_CONTROL: &str = "private, max-age=60";
const GITHUB_STATS_VARY: &str = "cookie";
const RESEND_BASE_URL: &str = "https://api.resend.com/";

#[derive(Clone)]
pub struct AuthState {
    pub github_client: auth::GithubClient,
    pub github_api_base: Url,
    pub http_client: Client,
    pub admin_logins: Vec<String>,
    pub token_key: [u8; 32],
    pub resend: ResendConfig,
}

#[derive(Clone)]
pub struct ResendConfig {
    pub api_key: Option<String>,
    pub from: Option<String>,
    pub base_url: Url,
}

impl AuthState {
    pub fn new(
        github_client: auth::GithubClient,
        github_api_base: Url,
        http_client: Client,
        token_key: [u8; 32],
        resend: ResendConfig,
    ) -> Self {
        Self::new_with_admins(
            github_client,
            github_api_base,
            http_client,
            Vec::new(),
            token_key,
            resend,
        )
    }

    pub fn new_with_admins(
        github_client: auth::GithubClient,
        github_api_base: Url,
        http_client: Client,
        admin_logins: Vec<String>,
        token_key: [u8; 32],
        resend: ResendConfig,
    ) -> Self {
        Self {
            github_client,
            github_api_base,
            http_client,
            admin_logins,
            token_key,
            resend,
        }
    }

    pub fn is_admin(&self, login: &str) -> bool {
        self.admin_logins.iter().any(|value| value == login)
    }
}

pub fn init_tracing() -> bool {
    static INIT: OnceLock<bool> = OnceLock::new();
    *INIT.get_or_init(|| {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .try_init()
            .is_ok()
    })
}

pub fn app_router<S>(auth_state: AuthState, session_layer: SessionManagerLayer<S>) -> Router
where
    S: SessionStore + Clone + Send + Sync + 'static,
{
    let app = dioxus::server::router(crate::AppEntry)
        .layer(from_fn_with_state(auth_state.clone(), auth_guard))
        .layer(session_layer.clone())
        .layer(from_fn(github_stats_cache_headers));
    let auth_router = router(auth_state, session_layer);

    app.merge(auth_router)
}

pub fn router<S>(auth_state: AuthState, session_layer: SessionManagerLayer<S>) -> Router
where
    S: SessionStore + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/auth/github", get(auth::github_auth))
        .route("/auth/github/callback", get(auth::github_callback))
        .route("/auth/logout", post(auth::logout))
        .route("/api/csrf-token", get(auth::csrf_token))
        .route("/api/session", get(auth::session_status))
        .route("/api/admin/stats", get(admin::admin_stats))
        .route("/api/admin/submissions", get(admin::admin_submissions))
        .route(
            "/api/admin/submissions/{id}/approve",
            post(admin::admin_submission_approve),
        )
        .route(
            "/api/admin/submissions/{id}/reject",
            post(admin::admin_submission_reject),
        )
        .route(
            "/api/admin/users/{id}/verify",
            post(admin::admin_user_verify),
        )
        .route(
            "/api/collections",
            get(collections::collections_index).post(collections::create_collection),
        )
        .route(
            "/api/collections/{id}",
            get(collections::collection_detail)
                .patch(collections::update_collection)
                .delete(collections::delete_collection),
        )
        .route(
            "/api/collections/{id}/modules",
            post(collections::add_module),
        )
        .route(
            "/api/collections/{id}/modules/{uuid}",
            delete(collections::remove_module),
        )
        .route(
            "/api/modules/{uuid}/screenshots",
            post(modules::upload_screenshot),
        )
        .route(
            "/api/modules/{uuid}/screenshots/{id}",
            delete(modules::delete_screenshot),
        )
        .route("/api/upload", post(modules::upload_module))
        .route(
            "/api/users/me",
            get(users::profile_get)
                .patch(users::update_profile)
                .delete(users::delete_account),
        )
        .route("/api/modules/mine", get(users::modules_mine))
        .route("/api/users/me/export", get(users::export_data))
        .route("/api/stars", get(stars::stars_index))
        .route("/api/stars/sync", post(stars::stars_sync))
        .route(
            "/api/notifications",
            get(notifications::notifications_index),
        )
        .route(
            "/api/notifications/unread-count",
            get(notifications::unread_count),
        )
        .route(
            "/api/notifications/{id}/read",
            patch(notifications::mark_read),
        )
        .route(
            "/api/notifications/mark-all-read",
            post(notifications::mark_all_read),
        )
        .route(
            "/api/notifications/stream",
            get(notifications::notifications_stream),
        )
        .route(
            "/api/notifications/preferences",
            get(notifications::preferences_get).patch(notifications::preferences_patch),
        )
        .route(
            "/api/modules/{uuid}/star",
            get(stars::star_status)
                .post(stars::star_module)
                .delete(stars::unstar_module),
        )
        .with_state(auth_state)
        .layer(from_fn(CsrfMiddleware::middleware))
        .layer(session_layer)
}

pub(super) fn map_status(headers: &HeaderMap, status: StatusCode) -> StatusCode {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return status;
    }
    let has_body = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        > 0;
    if !status.is_success() && has_body {
        return status;
    }
    StatusCode::BAD_GATEWAY
}

pub(super) fn api_base_url_from_env() -> String {
    env::var("BARFORGE_API_BASE_URL")
        .or_else(|_| env::var("PUBLIC_API_BASE_URL"))
        .unwrap_or_else(|_| api::api_base_url().to_string())
}

pub(crate) fn app_base_url(value: Option<&str>) -> String {
    let base = value.unwrap_or("http://127.0.0.1:8080");
    base.trim_end_matches('/').to_string()
}

pub(crate) fn github_redirect_url(base: &str) -> Result<String, oauth2::url::ParseError> {
    let base = ensure_trailing_slash(base);
    Ok(Url::parse(&base)?.join("auth/github/callback")?.to_string())
}

pub(crate) fn github_api_base_url(value: Option<&str>) -> Result<Url, oauth2::url::ParseError> {
    let base = value.unwrap_or("https://api.github.com/");
    let base = ensure_trailing_slash(base);
    Url::parse(&base)
}

pub(crate) fn session_db_url(value: Option<&str>) -> String {
    value
        .unwrap_or("sqlite://.barforge/sessions.db")
        .to_string()
}

pub(crate) fn session_secure_cookie(value: Option<&str>) -> bool {
    match value.map(|value| value.to_ascii_lowercase()) {
        Some(value) if value == "true" || value == "1" => true,
        Some(value) if value == "false" || value == "0" => false,
        _ => !cfg!(debug_assertions),
    }
}

pub(crate) fn admin_logins_from_env(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect()
}

async fn github_stats_cache_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    if path == "/api/v1/github/stats" {
        let headers = response.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static(GITHUB_STATS_CACHE_CONTROL),
        );
        headers.insert(
            header::VARY,
            header::HeaderValue::from_static(GITHUB_STATS_VARY),
        );
    }

    response
}

async fn auth_guard(
    State(state): State<AuthState>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if !is_protected_path(path) {
        return next.run(request).await;
    }

    let redirect_target = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or(path);
    let login_redirect =
        auth_redirect::login_redirect_url(&auth::resolve_redirect_target(Some(redirect_target)));

    let login = match auth::session_login(&session).await {
        Ok(value) => value,
        Err(status) => return status.into_response(),
    };

    let Some(login) = login else {
        return Redirect::to(&login_redirect).into_response();
    };

    if is_admin_path(path) {
        let is_admin = match auth::is_admin_for_session(&state, &session, &login).await {
            Ok(value) => value,
            Err(status) => return status.into_response(),
        };
        if !is_admin {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    next.run(request).await
}

fn is_protected_path(path: &str) -> bool {
    path == "/dashboard"
        || path == "/upload"
        || path == "/admin"
        || path == "/settings"
        || path.starts_with("/settings/")
}

fn is_admin_path(path: &str) -> bool {
    path == "/admin" || path.starts_with("/admin/")
}

fn ensure_trailing_slash(value: &str) -> String {
    if value.ends_with('/') {
        value.to_string()
    } else {
        format!("{value}/")
    }
}

pub fn auth_state_from_env(env: &impl Fn(&str) -> Option<String>) -> Result<AuthState> {
    let client_id = env("AUTH_GITHUB_ID").context("AUTH_GITHUB_ID is required")?;
    let client_secret = env("AUTH_GITHUB_SECRET").context("AUTH_GITHUB_SECRET is required")?;
    let token_secret = env("BARFORGE_TOKEN_SECRET").unwrap_or_else(|| client_secret.clone());
    let token_key = auth::token_key_from_secret(&token_secret);
    let app_base = app_base_url(env("BARFORGE_PUBLIC_BASE_URL").as_deref());
    let redirect_url = github_redirect_url(&app_base)?;
    let github_api_base = github_api_base_url(env("BARFORGE_GITHUB_API_BASE_URL").as_deref())?;
    let http_client = Client::builder()
        .redirect(Policy::none())
        .build()
        .context("failed to build http client")?;
    let github_client = auth::github_client(&client_id, &client_secret, &redirect_url)?;

    let admin_logins = admin_logins_from_env(env("BARFORGE_ADMIN_LOGINS").as_deref());
    let resend_api_key = env("BARFORGE_RESEND_API_KEY").filter(|value| !value.trim().is_empty());
    let resend_from = env("BARFORGE_RESEND_FROM").filter(|value| !value.trim().is_empty());
    let resend_base = env("BARFORGE_RESEND_BASE_URL")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| RESEND_BASE_URL.to_string());
    let resend_base_url =
        Url::parse(&resend_base).context("BARFORGE_RESEND_BASE_URL is invalid")?;
    let resend = ResendConfig {
        api_key: resend_api_key,
        from: resend_from,
        base_url: resend_base_url,
    };

    Ok(AuthState::new_with_admins(
        github_client,
        github_api_base,
        http_client,
        admin_logins,
        token_key,
        resend,
    ))
}

pub async fn session_layer_from_env(
    env: &impl Fn(&str) -> Option<String>,
) -> Result<SessionManagerLayer<SqliteStore>> {
    let db_url = session_db_url(env("BARFORGE_SESSION_DB_URL").as_deref());
    ensure_sqlite_dir(&db_url).context("failed to prepare sqlite directory")?;
    let pool = SqlitePool::connect(&db_url)
        .await
        .context("failed to connect session database")?;
    let store = SqliteStore::new(pool);
    store
        .migrate()
        .await
        .context("failed to migrate session schema")?;
    let secure = session_secure_cookie(env("BARFORGE_SESSION_SECURE").as_deref());
    let expiry = Expiry::OnInactivity(Duration::days(30));

    Ok(SessionManagerLayer::new(store)
        .with_name("barforge_session")
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(secure)
        .with_expiry(expiry))
}

fn ensure_sqlite_dir(db_url: &str) -> std::io::Result<()> {
    let Some(path) = sqlite_path_from_url(db_url) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn sqlite_path_from_url(db_url: &str) -> Option<std::path::PathBuf> {
    let path = db_url.strip_prefix("sqlite://")?;
    if path.is_empty() || path == ":memory:" {
        return None;
    }
    Some(std::path::PathBuf::from(path))
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    #[test]
    fn app_base_url_defaults_to_localhost() {
        assert_eq!(app_base_url(None), "http://127.0.0.1:8080");
    }

    #[test]
    fn app_base_url_trims_trailing_slash() {
        assert_eq!(
            app_base_url(Some("http://localhost:8080/")),
            "http://localhost:8080"
        );
    }

    #[test]
    fn github_redirect_url_handles_trailing_slash() {
        let url = github_redirect_url("http://localhost:8080/").expect("redirect url");
        assert_eq!(url, "http://localhost:8080/auth/github/callback");
    }

    #[test]
    fn github_redirect_url_handles_missing_trailing_slash() {
        let url = github_redirect_url("http://localhost:8080").expect("redirect url");
        assert_eq!(url, "http://localhost:8080/auth/github/callback");
    }

    #[test]
    fn github_api_base_url_defaults_to_github() {
        let url = github_api_base_url(None).expect("default url");
        assert_eq!(url.as_str(), "https://api.github.com/");
    }

    #[test]
    fn github_api_base_url_appends_trailing_slash() {
        let url = github_api_base_url(Some("https://api.example.com")).expect("parsed url");
        assert_eq!(url.as_str(), "https://api.example.com/");
    }

    #[test]
    fn session_db_url_defaults_to_sqlite_file() {
        assert_eq!(session_db_url(None), "sqlite://.barforge/sessions.db");
    }

    #[test]
    fn session_db_url_uses_override() {
        assert_eq!(session_db_url(Some("sqlite::memory:")), "sqlite::memory:");
    }

    #[test]
    fn session_secure_cookie_parses_values() {
        assert!(session_secure_cookie(Some("true")));
        assert!(session_secure_cookie(Some("1")));
        assert!(!session_secure_cookie(Some("false")));
        assert!(!session_secure_cookie(Some("0")));
    }

    #[test]
    fn admin_logins_from_env_defaults_empty() {
        assert!(admin_logins_from_env(None).is_empty());
    }

    #[test]
    fn admin_logins_from_env_parses_list() {
        let admins = admin_logins_from_env(Some("octo, admin, ,root "));

        assert_eq!(admins, vec!["octo", "admin", "root"]);
    }

    #[test]
    fn auth_state_from_env_requires_credentials() {
        let env = |key: &str| match key {
            "BARFORGE_PUBLIC_BASE_URL" => Some("http://localhost:8080".to_string()),
            _ => None,
        };
        let result = auth_state_from_env(&env);
        assert!(result.is_err());
    }

    #[test]
    fn init_tracing_is_successful() {
        assert!(init_tracing());
    }

    #[test]
    fn auth_state_from_env_builds_redirect_url() {
        let env = |key: &str| match key {
            "AUTH_GITHUB_ID" => Some("client-id".to_string()),
            "AUTH_GITHUB_SECRET" => Some("client-secret".to_string()),
            "BARFORGE_PUBLIC_BASE_URL" => Some("http://localhost:8080/".to_string()),
            _ => None,
        };
        let state = auth_state_from_env(&env).expect("auth state");
        let redirect = state.github_client.redirect_uri().expect("redirect uri");
        assert_eq!(
            redirect.as_str(),
            "http://localhost:8080/auth/github/callback"
        );
    }

    #[tokio::test]
    async fn session_layer_from_env_uses_memory_store() {
        let env = |key: &str| match key {
            "BARFORGE_SESSION_DB_URL" => Some("sqlite::memory:".to_string()),
            "BARFORGE_SESSION_SECURE" => Some("false".to_string()),
            _ => None,
        };
        let layer = session_layer_from_env(&env).await;
        assert!(layer.is_ok());
    }

    #[test]
    fn sqlite_path_from_url_ignores_memory() {
        assert!(sqlite_path_from_url("sqlite::memory:").is_none());
    }

    #[test]
    fn sqlite_path_from_url_parses_file() {
        let path = sqlite_path_from_url("sqlite://.barforge/sessions.db").expect("path");
        assert_eq!(path.to_string_lossy(), ".barforge/sessions.db");
    }

    #[test]
    fn ensure_sqlite_dir_creates_parent() {
        let base = std::env::temp_dir().join(format!(
            "barforge-session-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let db_path = base.join("sessions.db");
        let url = format!("sqlite://{}", db_path.to_string_lossy());
        ensure_sqlite_dir(&url).expect("create dir");
        assert!(base.exists());
        std::fs::remove_dir_all(&base).expect("cleanup dir");
    }

    #[tokio::test]
    async fn github_stats_cache_headers_set_on_stats_route() {
        let app = Router::new()
            .route("/api/v1/github/stats", get(|| async { "ok" }))
            .layer(from_fn(github_stats_cache_headers));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/github/stats")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("stats response");

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, max-age=60")
        );
        assert_eq!(
            headers
                .get(header::VARY)
                .and_then(|value| value.to_str().ok()),
            Some("cookie")
        );
    }

    #[tokio::test]
    async fn github_stats_cache_headers_skip_other_routes() {
        let app = Router::new()
            .route("/api/v1/featured", get(|| async { "ok" }))
            .layer(from_fn(github_stats_cache_headers));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/featured")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("featured response");

        let headers = response.headers();
        assert!(headers.get(header::CACHE_CONTROL).is_none());
        assert!(headers.get(header::VARY).is_none());
    }
}
