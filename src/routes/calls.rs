use axum::{
    Router,
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::{rpc_ok, rpc_created};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/calls/{number}", get(list_calls).post(start_call))
        .route("/v1/calls/{number}/{call_id}/accept", post(accept_call))
        .route("/v1/calls/{number}/{call_id}/reject", post(reject_call))
        .route("/v1/calls/{number}/{call_id}/hangup", post(hangup_call))
}

/// GET /v1/calls/{number} — list active voice calls.
async fn list_calls(State(st): State<AppState>, Path(number): Path<String>) -> Response {
    rpc_ok(&st, "listCalls", json!({ "account": number })).await
}

#[derive(Deserialize)]
struct StartCallBody {
    recipient: String,
}

/// POST /v1/calls/{number} — start an outgoing voice call.
async fn start_call(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<StartCallBody>,
) -> Response {
    rpc_created(&st, "startCall", json!({ "account": number, "recipient": body.recipient })).await
}

/// POST /v1/calls/{number}/{call_id}/accept — accept an incoming voice call.
async fn accept_call(
    State(st): State<AppState>,
    Path((number, call_id)): Path<(String, i64)>,
) -> Response {
    rpc_ok(&st, "acceptCall", json!({ "account": number, "call-id": call_id })).await
}

/// POST /v1/calls/{number}/{call_id}/reject — reject an incoming voice call.
async fn reject_call(
    State(st): State<AppState>,
    Path((number, call_id)): Path<(String, i64)>,
) -> Response {
    rpc_ok(&st, "rejectCall", json!({ "account": number, "call-id": call_id })).await
}

/// POST /v1/calls/{number}/{call_id}/hangup — hang up an active voice call.
async fn hangup_call(
    State(st): State<AppState>,
    Path((number, call_id)): Path<(String, i64)>,
) -> Response {
    rpc_ok(&st, "hangupCall", json!({ "account": number, "call-id": call_id })).await
}
