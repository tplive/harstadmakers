use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::{
    models::{ChallengeRequest, NewMemberRequest, StatisticsRequest, UnlockRequest},
    state::AppState,
};

pub fn app_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/members", post(create_member_handler))
        .route("/api/challenge", post(challenge_handler))
        .route("/api/door/unlock", post(unlock_handler))
        .route("/api/stats", post(statistics_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn statistics_handler(
    State(state): State<AppState>,
    Json(payload): Json<StatisticsRequest>,
) -> impl IntoResponse {
    info!("statistics requested");
    match state.statistics(payload) {
        Ok(statistics) => (StatusCode::OK, Json(serde_json::json!(statistics))),
        Err(error) => {
            warn!(error = %error, "statistics request rejected");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
        }
    }
}

async fn health() -> impl IntoResponse {
    info!("health check");
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "ok", "service": "TTL API"})),
    )
}

async fn create_member_handler(
    State(state): State<AppState>,
    Json(payload): Json<NewMemberRequest>,
) -> impl IntoResponse {
    info!("member registration requested");
    match state.add_member(payload) {
        Ok(registration) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(registration).unwrap()),
        ),
        Err(error) => {
            warn!(error = %error, "member registration rejected");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
        }
    }
}

async fn challenge_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChallengeRequest>,
) -> impl IntoResponse {
    info!("challenge requested");
    match state.generate_challenge(payload) {
        Ok(response) => (
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap()),
        ),
        Err(error) => {
            warn!(error = %error, "challenge request rejected");
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
        }
    }
}

async fn unlock_handler(
    State(state): State<AppState>,
    Json(payload): Json<UnlockRequest>,
) -> impl IntoResponse {
    info!(
        challenge_id = %payload.challenge_id,
        "door unlock requested"
    );
    match state.unlock_door(payload) {
        Ok(response) => (
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap()),
        ),
        Err(error) => {
            warn!(error = %error, "door unlock rejected");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
        }
    }
}
