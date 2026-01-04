use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::Json,
};
use barforge_types::{
    StarRequest, StarStatusResponse, StarsResponse, SyncStarsRequest, SyncStarsResponse, Versioned,
};
use serde::Serialize;
use tower_sessions::Session;

use super::AuthState;
use super::auth::session_access_token;
use super::map_status;
use crate::api;

const API_ACCEPT: &str = "application/json";

#[derive(Serialize)]
pub(super) struct StarActionResponse {
    success: bool,
    starred: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub(super) async fn stars_index(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<StarsResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state).await?;
    let Some(token) = token else {
        return Ok(Json(empty_stars_payload()));
    };

    let base = super::api_base_url_from_env();
    let url = api::stars_url(&base);
    let response = auth_state
        .http_client
        .get(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Ok(Json(empty_stars_payload()));
    }

    let payload = response
        .json::<Versioned<StarsResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn stars_sync(
    State(auth_state): State<AuthState>,
    session: Session,
    Json(payload): Json<SyncStarsRequest>,
) -> Result<Json<Versioned<SyncStarsResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::stars_sync_url(&base);
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

    let payload = response
        .json::<Versioned<SyncStarsResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn star_status(
    State(auth_state): State<AuthState>,
    Path(uuid): Path<String>,
    session: Session,
) -> Result<Json<Versioned<StarStatusResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state).await?;
    let base = super::api_base_url_from_env();
    let url = api::star_status_url(&base, &uuid);
    let mut request = auth_state
        .http_client
        .get(url)
        .header(header::ACCEPT, API_ACCEPT);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(map_status(response.headers(), response.status()));
    }

    let payload = response
        .json::<Versioned<StarStatusResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn star_module(
    State(auth_state): State<AuthState>,
    Path(uuid): Path<String>,
    session: Session,
) -> Result<Json<StarActionResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::star_status_url(&base, &uuid);
    let response = auth_state
        .http_client
        .post(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .json(&StarRequest {
            is_public: Some(true),
        })
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(Json(StarActionResponse {
            success: false,
            starred: false,
            error: Some("Module not found".to_string()),
        }));
    }

    if !response.status().is_success() {
        return Ok(Json(StarActionResponse {
            success: false,
            starred: false,
            error: Some("Failed to star module".to_string()),
        }));
    }

    Ok(Json(StarActionResponse {
        success: true,
        starred: true,
        error: None,
    }))
}

pub(super) async fn unstar_module(
    State(auth_state): State<AuthState>,
    Path(uuid): Path<String>,
    session: Session,
) -> Result<Json<StarActionResponse>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::star_status_url(&base, &uuid);
    let response = auth_state
        .http_client
        .delete(url)
        .bearer_auth(token)
        .header(header::ACCEPT, API_ACCEPT)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() && response.status() != StatusCode::NOT_FOUND {
        return Ok(Json(StarActionResponse {
            success: false,
            starred: true,
            error: Some("Failed to unstar module".to_string()),
        }));
    }

    Ok(Json(StarActionResponse {
        success: true,
        starred: false,
        error: None,
    }))
}

fn empty_stars_payload() -> Versioned<StarsResponse> {
    Versioned {
        version: 0,
        payload: StarsResponse {
            modules: Vec::new(),
            total: 0,
        },
    }
}
