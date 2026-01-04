use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Json, Response},
};
use barforge_types::{
    MarkAllReadResponse, MarkReadResponse, NotificationPreferences, NotificationsResponse,
    UnreadCountResponse, Versioned,
};
use serde::Deserialize;
use tower_sessions::Session;

use super::AuthState;
use super::auth::session_access_token;
use super::map_status;
use crate::api;

const API_ACCEPT: &str = "application/json";

#[derive(Deserialize)]
pub(super) struct NotificationsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub(super) async fn notifications_index(
    State(auth_state): State<AuthState>,
    session: Session,
    Query(params): Query<NotificationsQuery>,
) -> Result<Json<NotificationsResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let url = api::notifications_url(api::api_base_url());
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    let response = auth_state
        .http_client
        .get(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .query(&[("limit", limit), ("offset", offset)])
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<NotificationsResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn unread_count(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let url = api::notifications_unread_url(api::api_base_url());
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
        .json::<UnreadCountResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn mark_all_read(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<MarkAllReadResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let url = api::notifications_mark_all_read_url(api::api_base_url());
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
        .json::<MarkAllReadResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn mark_read(
    State(auth_state): State<AuthState>,
    Path(id): Path<i64>,
    session: Session,
) -> Result<Json<MarkReadResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let url = api::notification_mark_read_url(api::api_base_url(), id);
    let response = auth_state
        .http_client
        .patch(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<MarkReadResponse>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn notifications_stream(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Response, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let url = api::notifications_stream_url(api::api_base_url());
    let response = auth_state
        .http_client
        .get(url)
        .bearer_auth(token)
        .header(header::ACCEPT, "text/event-stream")
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

pub(super) async fn preferences_get(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<NotificationPreferences>>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::notifications_preferences_url(&base);
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
        .json::<Versioned<NotificationPreferences>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn preferences_patch(
    State(auth_state): State<AuthState>,
    session: Session,
    Json(payload): Json<NotificationPreferences>,
) -> Result<Json<Versioned<NotificationPreferences>>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::notifications_preferences_url(&base);
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

    let payload = response
        .json::<Versioned<NotificationPreferences>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}
