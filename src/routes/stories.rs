use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::rpc_created;
use super::messages::spill_attachments_to_disk;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/stories/{number}", post(send_story))
}

#[derive(Deserialize)]
struct SendStoryBody {
    /// A data URI (`data:<mime>;filename=<name>;base64,<data>`) or a local
    /// file path already on the signal-cli host.
    attachment: String,
    #[serde(default)]
    group_id: Option<String>,
    #[serde(default)]
    no_replies: Option<bool>,
}

/// POST /v1/stories/{number} — post a story (photo/video) to this account's
/// Story, or to a group's story if `group_id` is given.
async fn send_story(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<SendStoryBody>,
) -> Response {
    let mut params = json!({ "account": number, "attachment": [body.attachment] });
    let _spilled = match spill_attachments_to_disk(&mut params) {
        Ok(guard) => guard,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response();
        }
    };
    // signal-cli's --attachment for sendStory takes a single path, not a
    // list - unwrap the one-element array spill_attachments_to_disk expects.
    if let Some(arr) = params["attachment"].as_array() {
        if let Some(first) = arr.first().cloned() {
            params["attachment"] = first;
        }
    }
    if let Some(g) = &body.group_id {
        params["group-id"] = json!(g);
    }
    if let Some(true) = body.no_replies {
        params["no-replies"] = json!(true);
    }
    rpc_created(&st, "sendStory", params).await
}
