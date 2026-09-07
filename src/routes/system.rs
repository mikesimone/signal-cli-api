use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::state::AppState;
use super::helpers::rpc_ok;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/about", get(about))
        .route("/v1/version", get(version))
}

async fn health() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

async fn about() -> Response {
    let info = json!({
        "versions": {
            "signal-cli-api": env!("CARGO_PKG_VERSION"),
        },
        "build": {
            "target": std::env::consts::ARCH,
            "os": std::env::consts::OS,
        }
    });
    Json(info).into_response()
}

/// GET /v1/version — signal-cli's own reported version, straight from its
/// `version` RPC method (distinct from `/v1/about`, which reports this
/// wrapper's own build info, not signal-cli's).
async fn version(State(st): State<AppState>) -> Response {
    rpc_ok(&st, "version", json!({})).await
}
