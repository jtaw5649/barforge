use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::Json,
};
use barforge_types::{CollectionDetailResponse, CollectionsResponse, Versioned};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::AuthState;
use super::auth::session_access_token;
use super::map_status;
use crate::api;

const API_ACCEPT: &str = "application/json";

#[derive(Deserialize, Serialize)]
pub(super) struct AddModuleRequest {
    module_uuid: String,
    note: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct CreateCollectionRequest {
    name: String,
    description: Option<String>,
    visibility: String,
}

#[derive(Deserialize, Serialize)]
pub(super) struct UpdateCollectionRequest {
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
}

pub(super) async fn collections_index(
    State(auth_state): State<AuthState>,
    session: Session,
) -> Result<Json<Versioned<CollectionsResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collections_url(&base);
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
        .json::<Versioned<CollectionsResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn collection_detail(
    State(auth_state): State<AuthState>,
    Path(id): Path<i64>,
    session: Session,
) -> Result<Json<Versioned<CollectionDetailResponse>>, StatusCode> {
    let token = session_access_token(&session, &auth_state).await?;
    let base = super::api_base_url_from_env();
    let url = api::collection_url(&base, id);
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
        .json::<Versioned<CollectionDetailResponse>>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Json(payload))
}

pub(super) async fn create_collection(
    State(auth_state): State<AuthState>,
    session: Session,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collections_url(&base);
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

pub(super) async fn update_collection(
    State(auth_state): State<AuthState>,
    Path(id): Path<i64>,
    session: Session,
    Json(payload): Json<UpdateCollectionRequest>,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collection_url(&base, id);
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

pub(super) async fn delete_collection(
    State(auth_state): State<AuthState>,
    Path(id): Path<i64>,
    session: Session,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collection_url(&base, id);
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

pub(super) async fn add_module(
    State(auth_state): State<AuthState>,
    Path(id): Path<i64>,
    session: Session,
    Json(payload): Json<AddModuleRequest>,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collection_modules_url(&base, id);
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

pub(super) async fn remove_module(
    State(auth_state): State<AuthState>,
    Path((id, uuid)): Path<(i64, String)>,
    session: Session,
) -> Result<StatusCode, StatusCode> {
    let token = session_access_token(&session, &auth_state)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let base = super::api_base_url_from_env();
    let url = api::collection_module_url(&base, id, &uuid);
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
