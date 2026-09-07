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
        .route("/v1/accounts", get(list_accounts))
        .route("/v1/register/{number}", post(register))
        .route("/v1/register/{number}/verify/{token}", post(verify))
        .route("/v1/unregister/{number}", post(unregister))
        .route(
            "/v1/accounts/{number}/rate-limit-challenge",
            post(rate_limit_challenge),
        )
        .route("/v1/accounts/{number}/settings", put(update_settings))
        .route(
            "/v1/accounts/{number}/pin",
            post(set_pin).delete(remove_pin),
        )
        .route(
            "/v1/accounts/{number}/username",
            post(set_username).delete(remove_username),
        )
        .route(
            "/v1/accounts/{number}/sync-request",
            post(sync_request),
        )
        .route("/v1/accounts/{number}/number", post(start_change_number))
        .route(
            "/v1/accounts/{number}/number/verify",
            post(finish_change_number),
        )
}

async fn list_accounts(State(st): State<AppState>) -> Response {
    rpc_ok(&st, "listAccounts", json!({})).await
}

#[derive(Deserialize)]
struct RegisterBody {
    #[serde(default)]
    captcha: Option<String>,
    #[serde(default)]
    voice: Option<bool>,
}

async fn register(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Response {
    let mut params = json!({ "account": number });
    if let Some(captcha) = body.captcha {
        params["captcha"] = json!(captcha);
    }
    if let Some(voice) = body.voice {
        params["voice"] = json!(voice);
    }
    rpc_no_content(&st, "register", params).await
}

async fn verify(
    Path((number, token)): Path<(String, String)>,
    State(st): State<AppState>,
) -> Response {
    rpc_no_content(&st, "verify", json!({ "account": number, "verificationCode": token })).await
}

async fn unregister(Path(number): Path<String>, State(st): State<AppState>) -> Response {
    rpc_no_content(&st, "unregister", json!({ "account": number })).await
}

#[derive(Deserialize)]
struct RateLimitBody {
    challenge: String,
    captcha: String,
}

async fn rate_limit_challenge(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<RateLimitBody>,
) -> Response {
    rpc_no_content(&st, "submitRateLimitChallenge", json!({
        "account": number,
        "challenge": body.challenge,
        "captcha": body.captcha,
    })).await
}

/// signal-cli has no "trust mode" / configurable-at-runtime account
/// settings RPC at all (`--trust-new-identities` is a CLI/daemon startup
/// flag baked in at launch, not something toggled per-account over
/// JSON-RPC). The real per-account attributes signal-cli *does* let you
/// update at runtime live on `updateAccount` - device name, unidentified
/// sender/discoverability/number-sharing preferences. This previously
/// called a nonexistent "updateAccountSettings" RPC method with a
/// "trustMode" field that signal-cli has never supported; both the method
/// name and the field were fictional and returned "Method not found" from
/// real signal-cli.
#[derive(Deserialize)]
struct SettingsBody {
    #[serde(default)]
    device_name: Option<String>,
    #[serde(default)]
    unrestricted_unidentified_sender: Option<bool>,
    #[serde(default)]
    discoverable_by_number: Option<bool>,
    #[serde(default)]
    number_sharing: Option<bool>,
}

async fn update_settings(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<SettingsBody>,
) -> Response {
    let mut params = json!({ "account": number });
    if let Some(name) = &body.device_name {
        params["device-name"] = json!(name);
    }
    if let Some(v) = body.unrestricted_unidentified_sender {
        params["unrestricted-unidentified-sender"] = json!(v);
    }
    if let Some(v) = body.discoverable_by_number {
        params["discoverable-by-number"] = json!(v);
    }
    if let Some(v) = body.number_sharing {
        params["number-sharing"] = json!(v);
    }
    rpc_no_content(&st, "updateAccount", params).await
}

#[derive(Deserialize)]
struct PinBody {
    pin: String,
}

async fn set_pin(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<PinBody>,
) -> Response {
    rpc_no_content(&st, "setPin", json!({ "account": number, "pin": body.pin })).await
}

async fn remove_pin(Path(number): Path<String>, State(st): State<AppState>) -> Response {
    rpc_no_content(&st, "removePin", json!({ "account": number })).await
}

#[derive(Deserialize)]
struct UsernameBody {
    username: String,
}

/// signal-cli has no standalone "setUsername"/"removeUsername" RPC methods -
/// username changes go through `updateAccount`'s mutually-exclusive
/// `username`/`delete-username` fields (see UpdateAccountCommand). Both
/// previously named RPC methods here were fictional and would 404 against
/// real signal-cli.
async fn set_username(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<UsernameBody>,
) -> Response {
    rpc_no_content(&st, "updateAccount", json!({ "account": number, "username": body.username })).await
}

async fn remove_username(Path(number): Path<String>, State(st): State<AppState>) -> Response {
    rpc_no_content(&st, "updateAccount", json!({ "account": number, "delete-username": true })).await
}

/// POST /v1/accounts/{number}/sync-request — ask the primary device to
/// resync groups/contacts/etc. to this (linked) device.
async fn sync_request(Path(number): Path<String>, State(st): State<AppState>) -> Response {
    rpc_no_content(&st, "sendSyncRequest", json!({ "account": number })).await
}

#[derive(Deserialize)]
struct StartChangeNumberBody {
    number: String,
    #[serde(default)]
    voice: Option<bool>,
    #[serde(default)]
    captcha: Option<String>,
}

/// POST /v1/accounts/{number}/number — begin changing this account to a new
/// phone number; triggers an SMS/voice verification code.
async fn start_change_number(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<StartChangeNumberBody>,
) -> Response {
    let mut params = json!({ "account": number, "number": body.number });
    if let Some(v) = body.voice {
        params["voice"] = json!(v);
    }
    if let Some(c) = &body.captcha {
        params["captcha"] = json!(c);
    }
    rpc_no_content(&st, "startChangeNumber", params).await
}

#[derive(Deserialize)]
struct FinishChangeNumberBody {
    number: String,
    verification_code: String,
    #[serde(default)]
    pin: Option<String>,
}

/// POST /v1/accounts/{number}/number/verify — complete a number change with
/// the code received via SMS/voice.
async fn finish_change_number(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<FinishChangeNumberBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "number": body.number,
        "verification-code": body.verification_code,
    });
    if let Some(p) = &body.pin {
        params["pin"] = json!(p);
    }
    rpc_no_content(&st, "finishChangeNumber", params).await
}
