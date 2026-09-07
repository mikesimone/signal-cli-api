use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::{rpc_ok, rpc_no_content};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/contacts/{number}", get(list_contacts).put(update_contact))
        .route(
            "/v1/contacts/{number}/{recipient}",
            get(get_contact).delete(remove_contact),
        )
        .route("/v1/contacts/{number}/sync", post(sync_contacts))
        .route("/v1/contacts/{number}/{recipient}/avatar", get(get_avatar))
        .route(
            "/v1/contacts/{number}/{recipient}/block",
            post(block_contact).delete(unblock_contact),
        )
        .route(
            "/v1/contacts/{number}/message-requests",
            put(message_request_response),
        )
}

async fn list_contacts(
    State(st): State<AppState>,
    Path(number): Path<String>,
) -> Response {
    rpc_ok(&st, "listContacts", json!({ "account": number })).await
}

async fn get_contact(
    State(st): State<AppState>,
    Path((number, recipient)): Path<(String, String)>,
) -> Response {
    rpc_ok(&st, "listContacts", json!({ "account": number, "recipient": [recipient] })).await
}

#[derive(Deserialize)]
struct UpdateContactBody {
    name: Option<String>,
    expiration: Option<u64>,
    recipient: Option<String>,
}

async fn update_contact(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<UpdateContactBody>,
) -> Response {
    let mut params = json!({ "account": number });
    if let Some(name) = &body.name {
        params["name"] = json!(name);
    }
    if let Some(exp) = body.expiration {
        params["expiration"] = json!(exp);
    }
    if let Some(recipient) = &body.recipient {
        params["recipient"] = json!([recipient]);
    }
    rpc_ok(&st, "updateContact", params).await
}

/// DELETE /v1/contacts/{number}/{recipient} — remove a contact.
/// signal-cli's `removeContact` supports `--hide` (keep data, hide from
/// list) or `--forget` (wipe identity/session data too); default is a
/// plain removal of contact details only.
#[derive(Deserialize, Default)]
struct RemoveContactBody {
    #[serde(default)]
    hide: Option<bool>,
    #[serde(default)]
    forget: Option<bool>,
}

async fn remove_contact(
    State(st): State<AppState>,
    Path((number, recipient)): Path<(String, String)>,
    body: Option<Json<RemoveContactBody>>,
) -> Response {
    let body = body.map(|Json(b)| b).unwrap_or_default();
    let mut params = json!({ "account": number, "recipient": recipient });
    if let Some(true) = body.hide {
        params["hide"] = json!(true);
    }
    if let Some(true) = body.forget {
        params["forget"] = json!(true);
    }
    rpc_ok(&st, "removeContact", params).await
}

async fn sync_contacts(
    State(st): State<AppState>,
    Path(number): Path<String>,
) -> Response {
    rpc_ok(&st, "sendContacts", json!({ "account": number })).await
}

/// GET /v1/contacts/{number}/{recipient}/avatar — retrieve a contact's
/// avatar, base64 encoded, via signal-cli's `getAvatar` RPC method.
async fn get_avatar(
    State(st): State<AppState>,
    Path((number, recipient)): Path<(String, String)>,
) -> Response {
    rpc_ok(&st, "getAvatar", json!({ "account": number, "contact": recipient })).await
}

/// POST /v1/contacts/{number}/{recipient}/block — block a contact (no
/// messages will be received from them). DELETE unblocks. Both use
/// signal-cli's `block`/`unblock` RPC methods, which the existing group
/// routes already call for group-level blocking - this exposes the same
/// methods for individual contacts.
async fn block_contact(
    State(st): State<AppState>,
    Path((number, recipient)): Path<(String, String)>,
) -> Response {
    rpc_ok(&st, "block", json!({ "account": number, "recipient": [recipient] })).await
}

async fn unblock_contact(
    State(st): State<AppState>,
    Path((number, recipient)): Path<(String, String)>,
) -> Response {
    rpc_ok(&st, "unblock", json!({ "account": number, "recipient": [recipient] })).await
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageRequestResponseType {
    Accept,
    Delete,
}

#[derive(Deserialize)]
struct MessageRequestResponseBody {
    #[serde(default)]
    recipient: Option<String>,
    #[serde(default)]
    group_id: Option<String>,
    #[serde(rename = "type")]
    response_type: MessageRequestResponseType,
}

/// PUT /v1/contacts/{number}/message-requests — accept or delete a pending
/// message request from a contact or group, via signal-cli's
/// `sendMessageRequestResponse` RPC method.
async fn message_request_response(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<MessageRequestResponseBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "type": match body.response_type {
            MessageRequestResponseType::Accept => "accept",
            MessageRequestResponseType::Delete => "delete",
        },
    });
    if let Some(r) = &body.recipient {
        params["recipient"] = json!([r]);
    }
    if let Some(g) = &body.group_id {
        params["group-id"] = json!([g]);
    }
    rpc_no_content(&st, "sendMessageRequestResponse", params).await
}
