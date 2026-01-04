use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Json,
};
use barforge_types::{ModulesResponse, RegistryModule, StarsResponse, UserProfile, Versioned};
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tower_sessions::Session;

use super::AuthState;
use super::auth::{session_access_token, session_email};
use super::map_status;
use crate::api;
use crate::forms::UpdateProfileRequest;

const API_ACCEPT: &str = "application/json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExportData {
    exported_at: String,
    profile: Option<UserProfile>,
    modules: Vec<RegistryModule>,
    stars: Vec<RegistryModule>,
}

#[derive(Serialize)]
struct ResendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    text: String,
}

async fn send_export_email(
    auth_state: &AuthState,
    session: &Session,
    exported_at: &str,
) -> Result<(), StatusCode> {
    let Some(email) = session_email(session).await? else {
        return Ok(());
    };
    let Some(api_key) = auth_state.resend.api_key.as_deref() else {
        return Ok(());
    };
    let Some(from) = auth_state.resend.from.as_deref() else {
        return Ok(());
    };
    let url = auth_state
        .resend
        .base_url
        .join("emails")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let payload = ResendEmailRequest {
        from: from.to_string(),
        to: vec![email],
        subject: "Your Barforge data export".to_string(),
        text: format!("Your Barforge data export was generated at {exported_at}."),
    };
    let response = auth_state
        .http_client
        .post(url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    Ok(())
}

pub(super) async fn profile_get(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<UserProfile>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
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
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<UserProfile>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn modules_mine(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<ModulesResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = format!("{}/api/v1/modules/mine", base.trim_end_matches('/'));
    let response = auth_state
        .http_client
        .get(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<Versioned<ModulesResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn update_profile(
    State(auth_state): State<AuthState>,
    session: Session,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = format!("{}/api/v1/users/me", base.trim_end_matches('/'));
    let response = auth_state
        .http_client
        .patch(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .json(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    Ok(StatusCode::OK)
}

pub(super) async fn delete_account(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = format!("{}/api/v1/users/me", base.trim_end_matches('/'));
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

    Ok(StatusCode::OK)
}

pub(super) async fn export_data(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<(HeaderMap, Json<ExportData>), StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let profile_url = format!("{}/api/v1/users/me", base.trim_end_matches('/'));
    let modules_url = format!("{}/api/v1/modules/mine", base.trim_end_matches('/'));
    let stars_url = api::stars_url(&base);

    let profile = match auth_state
        .http_client
        .get(profile_url)
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response
            .json::<UserProfile>()
            .await
            .map(Some)
            .map_err(|_| StatusCode::BAD_GATEWAY)?,
        Ok(_) => None,
        Err(_) => return Err(StatusCode::BAD_GATEWAY),
    };

    let modules = match auth_state
        .http_client
        .get(modules_url)
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let payload = response
                .json::<Versioned<ModulesResponse>>()
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            payload.payload.modules
        }
        Ok(_) => Vec::new(),
        Err(_) => return Err(StatusCode::BAD_GATEWAY),
    };

    let stars = match auth_state
        .http_client
        .get(stars_url)
        .bearer_auth(&token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let payload = response
                .json::<Versioned<StarsResponse>>()
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            payload
                .payload
                .modules
                .into_iter()
                .map(|star| star.module)
                .collect()
        }
        Ok(_) => Vec::new(),
        Err(_) => return Err(StatusCode::BAD_GATEWAY),
    };

    let exported_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let filename = exported_at
        .split('T')
        .next()
        .map(|date| format!("barforge-export-{date}.json"))
        .unwrap_or_else(|| "barforge-export.json".to_string());

    let payload = ExportData {
        exported_at,
        profile,
        modules,
        stars,
    };
    if let Err(status) = send_export_email(&auth_state, &session, &payload.exported_at).await {
        tracing::warn!(?status, "Failed to send export confirmation email");
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let disposition = format!("attachment; filename=\"{filename}\"");
    let disposition =
        HeaderValue::from_str(&disposition).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    headers.insert(header::CONTENT_DISPOSITION, disposition);

    Ok((headers, Json(payload)))
}
