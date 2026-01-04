use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Json, Redirect},
};
use axum_tower_sessions_csrf::get_or_create_token;
use barforge_types::{UserProfile, UserRole};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use ring::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    digest,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::AuthState;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_SCOPE_READ_USER: &str = "read:user";
const GITHUB_SCOPE_USER_EMAIL: &str = "user:email";
const GITHUB_API_ACCEPT: &str = "application/vnd.github+json";
const API_ACCEPT: &str = "application/json";
const GITHUB_USER_AGENT: &str = "barforge";
const SESSION_AUTH_KEY: &str = "github_auth_session";
const SESSION_STATE_KEY: &str = "github_oauth_state";
const SESSION_REDIRECT_KEY: &str = "auth_redirect_to";
const SESSION_TOKEN_KEY: &str = "github_access_token";
const TOKEN_NONCE_LEN: usize = 12;

pub type GithubClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

pub fn github_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<GithubClient, oauth2::url::ParseError> {
    github_client_with_urls(
        client_id,
        client_secret,
        redirect_url,
        GITHUB_AUTH_URL,
        GITHUB_TOKEN_URL,
    )
}

pub fn github_client_with_urls(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    auth_url: &str,
    token_url: &str,
) -> Result<GithubClient, oauth2::url::ParseError> {
    let auth_url = AuthUrl::new(auth_url.to_string())?;
    let token_url = TokenUrl::new(token_url.to_string())?;
    let redirect_url = RedirectUrl::new(redirect_url.to_string())?;

    Ok(BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url))
}

pub fn github_authorize_url(client: &GithubClient) -> (oauth2::url::Url, CsrfToken) {
    client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(GITHUB_SCOPE_READ_USER.to_string()))
        .add_scope(Scope::new(GITHUB_SCOPE_USER_EMAIL.to_string()))
        .url()
}

pub fn token_key_from_secret(secret: &str) -> [u8; 32] {
    let digest = digest::digest(&digest::SHA256, secret.as_bytes());
    let mut key = [0_u8; 32];
    key.copy_from_slice(digest.as_ref());
    key
}

fn token_cipher(key: &[u8; 32]) -> Result<LessSafeKey, StatusCode> {
    let unbound =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(LessSafeKey::new(unbound))
}

fn encrypt_access_token(token: &str, key: &[u8; 32]) -> Result<String, StatusCode> {
    let mut nonce_bytes = [0_u8; TOKEN_NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut in_out = token.as_bytes().to_vec();
    token_cipher(key)?
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut payload = Vec::with_capacity(TOKEN_NONCE_LEN + in_out.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&in_out);
    Ok(URL_SAFE_NO_PAD.encode(payload))
}

fn decrypt_access_token(token: &str, key: &[u8; 32]) -> Result<String, StatusCode> {
    let decoded = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if decoded.len() < TOKEN_NONCE_LEN {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut nonce_bytes = [0_u8; TOKEN_NONCE_LEN];
    let (nonce_slice, ciphertext) = decoded.split_at(TOKEN_NONCE_LEN);
    nonce_bytes.copy_from_slice(nonce_slice);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut in_out = ciphertext.to_vec();
    let plain = token_cipher(key)?
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token = std::str::from_utf8(plain).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(token.to_string())
}

async fn store_access_token(
    session: &Session,
    token: &str,
    auth_state: &AuthState,
) -> Result<(), StatusCode> {
    let encrypted = encrypt_access_token(token, &auth_state.token_key)?;
    session
        .insert(SESSION_TOKEN_KEY, encrypted)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(super) async fn session_access_token(
    session: &Session,
    auth_state: &AuthState,
) -> Result<Option<String>, StatusCode> {
    let encrypted: Option<String> = session
        .get(SESSION_TOKEN_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(encrypted) = encrypted else {
        return Ok(None);
    };
    match decrypt_access_token(&encrypted, &auth_state.token_key) {
        Ok(token) => Ok(Some(token)),
        Err(_) => {
            session
                .remove::<String>(SESSION_TOKEN_KEY)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(None)
        }
    }
}

#[tracing::instrument]
pub fn resolve_redirect_target(redirect_to: Option<&str>) -> String {
    match redirect_to {
        Some(value) if is_safe_redirect(value) => value.to_string(),
        _ => "/".to_string(),
    }
}

fn is_safe_redirect(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//") && !value.contains("://")
}

#[derive(Deserialize)]
pub(super) struct AuthRedirectQuery {
    #[serde(rename = "redirect_to", alias = "redirectTo")]
    redirect_to: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GithubUserResponse {
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GithubEmailResponse {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Serialize, Deserialize)]
struct AuthSession {
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
pub(super) struct SessionUser {
    login: String,
    email: Option<String>,
}

#[derive(Serialize)]
pub(super) struct SessionResponse {
    authenticated: bool,
    is_admin: bool,
    user: Option<SessionUser>,
}

#[tracing::instrument(skip(state, query, session))]
pub(super) async fn github_auth(
    State(state): State<AuthState>,
    Query(query): Query<AuthRedirectQuery>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let (authorize_url, csrf_state) = github_authorize_url(&state.github_client);
    let redirect_to = resolve_redirect_target(query.redirect_to.as_deref());

    if session
        .insert(SESSION_STATE_KEY, csrf_state.secret().to_string())
        .await
        .is_err()
    {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if session
        .insert(SESSION_REDIRECT_KEY, redirect_to)
        .await
        .is_err()
    {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Redirect::to(authorize_url.as_str()))
}

#[tracing::instrument(skip(auth_state, query, session))]
pub(super) async fn github_callback(
    State(auth_state): State<AuthState>,
    Query(query): Query<AuthCallbackQuery>,
    session: Session,
) -> Result<Redirect, StatusCode> {
    let AuthCallbackQuery {
        code,
        state: callback_state,
    } = query;
    let expected: Option<String> = session
        .get(SESSION_STATE_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if expected.as_deref() != Some(callback_state.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }

    session
        .remove::<String>(SESSION_STATE_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = auth_state
        .github_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(&auth_state.http_client)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let access_token = token.access_token().secret();
    let user = fetch_github_user(&auth_state, access_token).await?;
    let emails = fetch_github_emails(&auth_state, access_token).await?;
    let email = select_primary_email(&emails);

    let auth_session = AuthSession {
        login: user.login,
        email,
        name: user.name,
        avatar_url: user.avatar_url,
    };

    store_access_token(&session, access_token, &auth_state).await?;

    session
        .insert(SESSION_AUTH_KEY, auth_session)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirect_to = session
        .remove::<String>(SESSION_REDIRECT_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| "/".to_string());
    let redirect_to = resolve_redirect_target(Some(&redirect_to));

    Ok(Redirect::to(&redirect_to))
}

#[tracing::instrument(skip(auth_state, session))]
pub(super) async fn session_status(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<(HeaderMap, Json<SessionResponse>), StatusCode> {
    let auth: Option<AuthSession> = session
        .get(SESSION_AUTH_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut headers = HeaderMap::new();
    let response = match auth {
        Some(auth) => {
            let token = session_access_token(&session, &auth_state).await?;
            let profile = if let Some(ref token) = token {
                let _ = sync_user(&auth_state, token).await;
                fetch_user_profile(&auth_state, token).await.unwrap_or(None)
            } else {
                None
            };
            let mut is_admin = auth_state.is_admin(&auth.login);
            if let Some(profile) = profile.as_ref() {
                is_admin =
                    is_admin || matches!(profile.role, UserRole::Admin | UserRole::Moderator);
            }
            SessionResponse {
                authenticated: true,
                is_admin,
                user: Some(SessionUser {
                    login: auth.login,
                    email: auth.email,
                }),
            }
        }
        None => {
            let clear_cookie = HeaderValue::from_static("profile_cache=; Path=/; Max-Age=0");
            headers.append(header::SET_COOKIE, clear_cookie);
            SessionResponse {
                authenticated: false,
                is_admin: false,
                user: None,
            }
        }
    };

    Ok((headers, Json(response)))
}

pub(super) async fn session_login(session: &Session) -> Result<Option<String>, StatusCode> {
    let auth: Option<AuthSession> = session
        .get(SESSION_AUTH_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(auth.map(|auth| auth.login))
}

pub(super) async fn session_email(session: &Session) -> Result<Option<String>, StatusCode> {
    let auth: Option<AuthSession> = session
        .get(SESSION_AUTH_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(auth.and_then(|auth| auth.email))
}

#[tracing::instrument(skip(session))]
pub(super) async fn csrf_token(session: Session) -> Result<Json<CsrfTokenResponse>, StatusCode> {
    let token = get_or_create_token(&session)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(CsrfTokenResponse { token }))
}

#[tracing::instrument(skip(session))]
pub(super) async fn logout(session: Session) -> Result<StatusCode, StatusCode> {
    session
        .flush()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn sync_user(auth_state: &AuthState, token: &str) -> Result<(), StatusCode> {
    let base = super::api_base_url_from_env();
    let url = format!("{}/api/v1/auth/sync", base.trim_end_matches('/'));
    let response = auth_state
        .http_client
        .post(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    Ok(())
}

async fn fetch_user_profile(
    auth_state: &AuthState,
    token: &str,
) -> Result<Option<UserProfile>, StatusCode> {
    let base = super::api_base_url_from_env();
    let url = format!("{}/api/v1/users/me", base.trim_end_matches('/'));
    let response = auth_state
        .http_client
        .get(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let profile = response
        .json::<UserProfile>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Some(profile))
}

pub(super) async fn is_admin_for_session(
    auth_state: &AuthState,
    session: &Session,
    login: &str,
) -> Result<bool, StatusCode> {
    if auth_state.is_admin(login) {
        return Ok(true);
    }
    let token = session_access_token(session, auth_state).await?;
    let Some(token) = token else {
        return Ok(false);
    };
    let profile = fetch_user_profile(auth_state, &token).await?;
    Ok(
        profile
            .is_some_and(|profile| matches!(profile.role, UserRole::Admin | UserRole::Moderator)),
    )
}

async fn fetch_github_user(
    state: &AuthState,
    access_token: &str,
) -> Result<GithubUserResponse, StatusCode> {
    let url = state
        .github_api_base
        .join("user")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = state
        .http_client
        .get(url)
        .bearer_auth(access_token)
        .header(header::ACCEPT, GITHUB_API_ACCEPT)
        .header(header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    response
        .json::<GithubUserResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn fetch_github_emails(
    state: &AuthState,
    access_token: &str,
) -> Result<Vec<GithubEmailResponse>, StatusCode> {
    let url = state
        .github_api_base
        .join("user/emails")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = state
        .http_client
        .get(url)
        .bearer_auth(access_token)
        .header(header::ACCEPT, GITHUB_API_ACCEPT)
        .header(header::USER_AGENT, GITHUB_USER_AGENT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    response
        .json::<Vec<GithubEmailResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

fn select_primary_email(emails: &[GithubEmailResponse]) -> Option<String> {
    emails
        .iter()
        .find(|email| email.primary && email.verified)
        .or_else(|| emails.iter().find(|email| email.verified))
        .map(|email| email.email.clone())
}

#[derive(Serialize)]
pub(super) struct CsrfTokenResponse {
    token: String,
}

#[cfg(test)]
mod auth_session_tests {
    use super::{AuthSession, decrypt_access_token, encrypt_access_token, token_key_from_secret};

    #[test]
    fn auth_session_serialization_omits_access_token() {
        let session = AuthSession {
            login: "octo".to_string(),
            email: Some("octo@example.com".to_string()),
            name: Some("Octo Cat".to_string()),
            avatar_url: Some("http://localhost/avatar.png".to_string()),
        };

        let value = serde_json::to_value(session).expect("serialize session");

        assert!(value.get("access_token").is_none());
    }

    #[test]
    fn token_cipher_roundtrips() {
        let key = token_key_from_secret("test-secret");
        let token = "token-123";

        let encrypted = encrypt_access_token(token, &key).expect("encrypt token");
        let decrypted = decrypt_access_token(&encrypted, &key).expect("decrypt token");

        assert_eq!(decrypted, token);
    }

    #[test]
    fn token_cipher_rejects_invalid_payload() {
        let key = token_key_from_secret("test-secret");

        let result = decrypt_access_token("not-base64", &key);

        assert!(result.is_err());
    }
}
