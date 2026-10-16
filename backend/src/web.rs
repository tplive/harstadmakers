use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    models::{ApproveMemberRequest, ChallengeRequest, NewMemberRequest, UnlockRequest},
    state::AppState,
};

pub fn app_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route(
            "/api/members",
            get(list_members_handler).post(create_member_handler),
        )
        .route(
            "/api/members/:member_id/approve",
            post(approve_member_handler),
        )
        .route("/api/challenge", post(challenge_handler))
        .route("/api/door/unlock", post(unlock_handler))
        .layer(cors)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "ok", "service": "harstad-makers"})),
    )
}

async fn list_members_handler(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.list_members()))
}

async fn create_member_handler(
    State(state): State<AppState>,
    Json(payload): Json<NewMemberRequest>,
) -> impl IntoResponse {
    match state.add_member(payload) {
        Ok(registration) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(registration).unwrap()),
        ),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

async fn approve_member_handler(
    State(state): State<AppState>,
    Path(member_id): Path<String>,
    Json(payload): Json<ApproveMemberRequest>,
) -> impl IntoResponse {
    match state.approve_member_registration(&member_id, &payload.admin_member_id) {
        Ok(member) => (StatusCode::OK, Json(serde_json::to_value(member).unwrap())),
        Err(error) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

async fn challenge_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChallengeRequest>,
) -> impl IntoResponse {
    match state.generate_challenge(payload) {
        Ok(response) => (
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap()),
        ),
        Err(error) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

async fn unlock_handler(
    State(state): State<AppState>,
    Json(payload): Json<UnlockRequest>,
) -> impl IntoResponse {
    match state.unlock_door(payload) {
        Ok(response) => (
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap()),
        ),
        Err(error) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
