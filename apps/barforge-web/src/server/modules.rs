use axum::{
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::Json,
};
use barforge_types::{ScreenshotDeleteResponse, ScreenshotUploadResponse};
use oauth2::url::Url;
use serde::{Deserialize, Serialize};
use std::env;
use tower_sessions::Session;

use super::AuthState;
use super::auth::{session_access_token, session_login};
use super::map_status;
use crate::api;

const API_ACCEPT: &str = "application/json";
const MAX_SCREENSHOT_BYTES: usize = 10 * 1024 * 1024;
const ALLOWED_SCREENSHOT_TYPES: [&str; 3] = ["image/png", "image/jpeg", "image/webp"];
const MAX_PACKAGE_BYTES: usize = 10 * 1024 * 1024;
const ALLOWED_PACKAGE_EXTENSIONS: [&str; 2] = [".tar.gz", ".tgz"];
const MAX_NAME_LEN: usize = 100;
const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_CHANGELOG_LEN: usize = 5000;
const TURNSTILE_VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Default)]
struct UploadFields {
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    version: Option<String>,
    license: Option<String>,
    repo_url: Option<String>,
    changelog: Option<String>,
    turnstile_response: Option<String>,
    package: Option<UploadPackage>,
}

struct UploadPackage {
    content_type: String,
    bytes: Vec<u8>,
}

#[derive(Serialize)]
struct CreateModuleRequest {
    uuid: String,
    name: String,
    description: String,
    category: String,
    repo_url: String,
    license: String,
}

#[derive(Serialize)]
struct PublishRequest {
    changelog: Option<String>,
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
}

pub(super) async fn upload_screenshot(
    State(auth_state): State<AuthState>,
    Path(uuid): Path<String>,
    session: Session,
    mut multipart: Multipart,
) -> Result<Json<ScreenshotUploadResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut screenshot: Option<(String, Vec<u8>)> = None;
    let mut alt_text: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name().unwrap_or_default() {
            "screenshot" => {
                let Some(content_type) = field.content_type().map(|value| value.to_string()) else {
                    return Err(StatusCode::BAD_REQUEST);
                };
                if !ALLOWED_SCREENSHOT_TYPES.contains(&content_type.as_str()) {
                    return Err(StatusCode::BAD_REQUEST);
                }
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                if data.is_empty() || data.len() > MAX_SCREENSHOT_BYTES {
                    return Err(StatusCode::BAD_REQUEST);
                }
                screenshot = Some((content_type, data.to_vec()));
            }
            "alt_text" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    alt_text = Some(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    let Some((content_type, bytes)) = screenshot else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let base = super::api_base_url_from_env();
    let mut url = Url::parse(&api::module_screenshots_url(&base, &uuid))
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if let Some(alt_text) = alt_text {
        url.query_pairs_mut().append_pair("alt_text", &alt_text);
    }
    let response = auth_state
        .http_client
        .post(url.as_str())
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .header(header::CONTENT_TYPE, content_type)
        .body(bytes)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<ScreenshotUploadResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(payload))
}

pub(super) async fn delete_screenshot(
    State(auth_state): State<AuthState>,
    Path((uuid, id)): Path<(String, i64)>,
    session: Session,
) -> Result<Json<ScreenshotDeleteResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::module_screenshot_url(&base, &uuid, id);
    let response = auth_state
        .http_client
        .delete(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<ScreenshotDeleteResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(payload))
}

pub(super) async fn upload_module(
    State(auth_state): State<AuthState>,
    session: Session,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let login = session_login(&session)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut fields = UploadFields::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name().unwrap_or_default() {
            "name" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.name = Some(text.trim().to_string());
            }
            "description" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.description = Some(text.trim().to_string());
            }
            "category" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.category = Some(text.trim().to_string());
            }
            "version" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.version = Some(text.trim().to_string());
            }
            "license" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.license = Some(text.trim().to_string());
            }
            "repo_url" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fields.repo_url = Some(text.trim().to_string());
            }
            "changelog" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    fields.changelog = Some(trimmed.to_string());
                }
            }
            "cf-turnstile-response" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    fields.turnstile_response = Some(trimmed.to_string());
                }
            }
            "package" | "package_file" => {
                let filename = field.file_name().unwrap_or_default();
                if filename.is_empty() || !has_allowed_extension(filename) {
                    return Err(StatusCode::BAD_REQUEST);
                }
                let content_type = field
                    .content_type()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                if data.is_empty() || data.len() > MAX_PACKAGE_BYTES {
                    return Err(StatusCode::BAD_REQUEST);
                }
                fields.package = Some(UploadPackage {
                    content_type,
                    bytes: data.to_vec(),
                });
            }
            _ => {}
        }
    }

    let name = fields.name.ok_or(StatusCode::BAD_REQUEST)?;
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err(StatusCode::BAD_REQUEST);
    }
    let description = fields.description.ok_or(StatusCode::BAD_REQUEST)?;
    if description.is_empty() || description.len() > MAX_DESCRIPTION_LEN {
        return Err(StatusCode::BAD_REQUEST);
    }
    let category = fields.category.ok_or(StatusCode::BAD_REQUEST)?;
    if category.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let version = fields.version.ok_or(StatusCode::BAD_REQUEST)?;
    if version.is_empty() || !is_semver(&version) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let repo_url = fields.repo_url.ok_or(StatusCode::BAD_REQUEST)?;
    if repo_url.is_empty() || !is_https_url(&repo_url) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let license = fields.license.ok_or(StatusCode::BAD_REQUEST)?;
    if license.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let package = fields.package.ok_or(StatusCode::BAD_REQUEST)?;
    if let Some(ref changelog) = fields.changelog
        && changelog.len() > MAX_CHANGELOG_LEN
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let slug = slugify(&name).ok_or(StatusCode::BAD_REQUEST)?;
    let uuid = format!("{slug}@{login}");

    if let Some(secret) = turnstile_secret() {
        let response = fields
            .turnstile_response
            .as_deref()
            .ok_or(StatusCode::BAD_REQUEST)?;
        let verified = verify_turnstile(&auth_state.http_client, &secret, response).await?;
        if !verified {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let base = super::api_base_url_from_env();
    let create_response = auth_state
        .http_client
        .post(api_url(&base, "modules"))
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .json(&CreateModuleRequest {
            uuid: uuid.clone(),
            name: name.clone(),
            description: description.clone(),
            category: category.clone(),
            repo_url: repo_url.clone(),
            license: license.clone(),
        })
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !create_response.status().is_success() {
        return Err(map_status(
            create_response.headers(),
            create_response.status(),
        ));
    }

    let upload_response = auth_state
        .http_client
        .post(api_url(
            &base,
            &format!("modules/{uuid}/versions/{version}/upload"),
        ))
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .header(header::CONTENT_TYPE, package.content_type)
        .body(package.bytes)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !upload_response.status().is_success() {
        return Err(map_status(
            upload_response.headers(),
            upload_response.status(),
        ));
    }

    let publish_response = auth_state
        .http_client
        .post(api_url(
            &base,
            &format!("modules/{uuid}/versions/{version}/publish"),
        ))
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .json(&PublishRequest {
            changelog: fields.changelog,
        })
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !publish_response.status().is_success() {
        return Err(map_status(
            publish_response.headers(),
            publish_response.status(),
        ));
    }

    Ok(StatusCode::OK)
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    format!("{base}/api/v1/{path}")
}

fn has_allowed_extension(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    ALLOWED_PACKAGE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn is_semver(value: &str) -> bool {
    let core = value.split(['-', '+']).next().unwrap_or("");
    let mut parts = core.split('.');
    let major = parts.next().unwrap_or("");
    let minor = parts.next().unwrap_or("");
    let patch = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false;
    }
    if major.is_empty() || minor.is_empty() || patch.is_empty() {
        return false;
    }
    major.chars().all(|ch| ch.is_ascii_digit())
        && minor.chars().all(|ch| ch.is_ascii_digit())
        && patch.chars().all(|ch| ch.is_ascii_digit())
}

fn is_https_url(value: &str) -> bool {
    let Ok(parsed) = Url::parse(value) else {
        return false;
    };
    parsed.scheme() == "https"
        && parsed
            .host_str()
            .map(|host| !host.is_empty())
            .unwrap_or(false)
}

fn slugify(value: &str) -> Option<String> {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() { None } else { Some(out) }
}

fn turnstile_secret() -> Option<String> {
    env::var("TURNSTILE_SECRET").ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

async fn verify_turnstile(
    client: &reqwest::Client,
    secret: &str,
    response: &str,
) -> Result<bool, StatusCode> {
    let verify_url = turnstile_verify_url();
    let payload = [("secret", secret), ("response", response)];
    let verify_response = client
        .post(verify_url)
        .form(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !verify_response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    let body = verify_response
        .json::<TurnstileResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(body.success)
}

fn turnstile_verify_url() -> String {
    let value =
        env::var("TURNSTILE_VERIFY_URL").unwrap_or_else(|_| TURNSTILE_VERIFY_URL.to_string());
    let Ok(mut url) = Url::parse(&value) else {
        return value;
    };
    let mut path = url.path().to_string();
    while path.starts_with("//") {
        path.remove(0);
    }
    if path == "/" {
        url.set_path("/turnstile/v0/siteverify");
    } else {
        url.set_path(&path);
    }
    url.to_string()
}
