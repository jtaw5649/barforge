use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::Json;
use barforge_types::{AdminStats, RejectRequest, SubmissionsResponse, VerifyResponse, Versioned};
use tower_sessions::Session;

use super::AuthState;
use super::auth::{session_access_token, session_login};
use super::map_status;
use crate::api;

const API_ACCEPT: &str = "application/json";

async fn require_admin(auth_state: &AuthState, session: &Session) -> Result<String, StatusCode> {
    let token = session_access_token(session, auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let login = session_login(session)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !auth_state.is_admin(&login) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(token)
}

pub(super) async fn admin_stats(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<AdminStats>>, StatusCode> {
    let token = require_admin(&auth_state, &session).await?;

    let base = super::api_base_url_from_env();
    let url = api::admin_stats_url(&base);
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
        .json::<Versioned<AdminStats>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn admin_submissions(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<SubmissionsResponse>>, StatusCode> {
    let token = require_admin(&auth_state, &session).await?;

    let base = super::api_base_url_from_env();
    let url = api::admin_submissions_url(&base);
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
        .json::<Versioned<SubmissionsResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn admin_submission_approve(
    State(auth_state): State<AuthState>,
    session: Session,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let token = require_admin(&auth_state, &session).await?;
    let base = super::api_base_url_from_env();
    let url = api::admin_submission_approve_url(&base, id);
    let response = auth_state
        .http_client
        .post(url)
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

pub(super) async fn admin_submission_reject(
    State(auth_state): State<AuthState>,
    session: Session,
    Path(id): Path<i64>,
    Json(payload): Json<RejectRequest>,
) -> Result<StatusCode, StatusCode> {
    let token = require_admin(&auth_state, &session).await?;
    let base = super::api_base_url_from_env();
    let url = api::admin_submission_reject_url(&base, id);
    let response = auth_state
        .http_client
        .post(url)
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

pub(super) async fn admin_user_verify(
    State(auth_state): State<AuthState>,
    session: Session,
    Path(id): Path<i64>,
) -> Result<Json<Versioned<VerifyResponse>>, StatusCode> {
    let token = require_admin(&auth_state, &session).await?;
    let base = super::api_base_url_from_env();
    let url = api::admin_user_verify_url(&base, id);
    let response = auth_state
        .http_client
        .post(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<Versioned<VerifyResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}
