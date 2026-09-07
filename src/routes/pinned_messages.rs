use axum::{
    Router,
    extract::{Path, State},
    response::Response,
    routing::post,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::rpc_ok;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/pins/{number}", post(pin_message))
        .route("/v1/pins/{number}/unpin", post(unpin_message))
}

#[derive(Deserialize, Default)]
struct RecipientEnvelope {
    #[serde(default)]
    recipient: Option<String>,
    #[serde(default)]
    group_id: Option<String>,
}

impl RecipientEnvelope {
    fn apply(&self, params: &mut serde_json::Value) {
        if let Some(r) = &self.recipient {
            params["recipient"] = json!([r]);
        }
        if let Some(g) = &self.group_id {
            params["group-id"] = json!([g]);
        }
    }
}

#[derive(Deserialize)]
struct PinMessageBody {
    #[serde(flatten)]
    to: RecipientEnvelope,
    target_author: String,
    target_timestamp: i64,
    #[serde(default)]
    pin_duration: Option<i64>,
    #[serde(default)]
    story: Option<bool>,
}

/// POST /v1/pins/{number} — pin a previously sent/received message in a
/// conversation (signal-cli's `sendPinMessage`).
async fn pin_message(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<PinMessageBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "target-author": body.target_author,
        "target-timestamp": body.target_timestamp,
    });
    body.to.apply(&mut params);
    if let Some(d) = body.pin_duration {
        params["pin-duration"] = json!(d);
    }
    if let Some(true) = body.story {
        params["story"] = json!(true);
    }
    rpc_ok(&st, "sendPinMessage", params).await
}

#[derive(Deserialize)]
struct UnpinMessageBody {
    #[serde(flatten)]
    to: RecipientEnvelope,
    target_author: String,
    target_timestamp: i64,
    #[serde(default)]
    story: Option<bool>,
}

/// POST /v1/pins/{number}/unpin — unpin a previously pinned message
/// (signal-cli's `sendUnpinMessage`).
async fn unpin_message(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<UnpinMessageBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "target-author": body.target_author,
        "target-timestamp": body.target_timestamp,
    });
    body.to.apply(&mut params);
    if let Some(true) = body.story {
        params["story"] = json!(true);
    }
    rpc_ok(&st, "sendUnpinMessage", params).await
}
