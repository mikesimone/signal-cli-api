use axum::{
    Router,
    extract::{Path, State},
    response::Response,
    routing::{delete, post},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::{rpc_ok, rpc_created};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/polls/{number}", post(create_poll))
        .route("/v1/polls/{number}/vote", post(vote_poll))
        .route("/v1/polls/{number}", delete(close_poll))
}

/// Recipient addressing shared by all three poll endpoints - signal-cli
/// polls are sent like any other message (to a recipient, group, and/or
/// username), not addressed by a server-side poll ID.
#[derive(Deserialize, Default)]
struct RecipientEnvelope {
    #[serde(default)]
    recipient: Option<String>,
    #[serde(default)]
    recipients: Option<Vec<String>>,
    #[serde(default)]
    group_id: Option<String>,
    #[serde(default)]
    username: Option<Vec<String>>,
    #[serde(default)]
    note_to_self: Option<bool>,
    #[serde(default)]
    notify_self: Option<bool>,
}

impl RecipientEnvelope {
    fn apply(&self, params: &mut serde_json::Value) {
        let mut recipients: Vec<String> = self.recipients.clone().unwrap_or_default();
        if let Some(r) = &self.recipient {
            recipients.push(r.clone());
        }
        if !recipients.is_empty() {
            params["recipient"] = json!(recipients);
        }
        if let Some(g) = &self.group_id {
            params["group-id"] = json!([g]);
        }
        if let Some(u) = &self.username {
            params["username"] = json!(u);
        }
        if let Some(true) = self.note_to_self {
            params["note-to-self"] = json!(true);
        }
        if let Some(true) = self.notify_self {
            params["notify-self"] = json!(true);
        }
    }
}

/// POST /v1/polls/{number} — create and send a poll.
///
/// signal-cli's JSON-RPC method for this is `sendPollCreate` (not `sendPoll` -
/// that method doesn't exist and previously returned "Method not found" from
/// real signal-cli, masked in tests because the mock server ignores method
/// names it doesn't recognize).
#[derive(Deserialize)]
struct CreatePollBody {
    #[serde(flatten)]
    to: RecipientEnvelope,
    question: String,
    options: Vec<String>,
    #[serde(default)]
    allow_multiple: Option<bool>,
}

async fn create_poll(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<CreatePollBody>,
) -> Response {
    let mut params = json!({ "account": number, "question": body.question, "option": body.options });
    body.to.apply(&mut params);
    if let Some(false) = body.allow_multiple {
        params["no-multi"] = json!(true);
    }
    rpc_created(&st, "sendPollCreate", params).await
}

/// POST /v1/polls/{number}/vote — vote on an existing poll.
#[derive(Deserialize)]
struct VotePollBody {
    #[serde(flatten)]
    to: RecipientEnvelope,
    poll_author: String,
    poll_timestamp: i64,
    option: i32,
}

async fn vote_poll(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<VotePollBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "poll-author": body.poll_author,
        "poll-timestamp": body.poll_timestamp,
        "option": body.option,
    });
    body.to.apply(&mut params);
    rpc_ok(&st, "sendPollVote", params).await
}

/// DELETE /v1/polls/{number} — terminate (close) a poll this account sent.
///
/// signal-cli's method is `sendPollTerminate`, addressed by the poll
/// message's own timestamp (`pollTimestamp`) - not a `pollId`, which
/// signal-cli has no concept of.
#[derive(Deserialize)]
struct ClosePollBody {
    #[serde(flatten)]
    to: RecipientEnvelope,
    poll_timestamp: i64,
}

async fn close_poll(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<ClosePollBody>,
) -> Response {
    let mut params = json!({ "account": number, "poll-timestamp": body.poll_timestamp });
    body.to.apply(&mut params);
    rpc_ok(&st, "sendPollTerminate", params).await
}
