use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::state::AppState;
use super::helpers::{rpc_ok, rpc_no_content};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/configuration",
            get(get_global_config).post(set_global_config),
        )
        .route(
            "/v1/configuration/{number}/settings",
            get(get_account_config).post(set_account_config),
        )
}

// KNOWN BROKEN, deliberately left as-is (not fixed as part of this pass):
// `getConfiguration`/`setConfiguration`/`getAccountSettings`/
// `setAccountSettings` are not real signal-cli JSON-RPC methods.
// `getConfiguration`/`setConfiguration` only exist on signal-cli's D-Bus
// interface (confirmed via a full string-constant scan of the installed
// jar - they appear solely in `DbusSignalImpl`, never in the JSON-RPC
// `Commands` registry), and `getAccountSettings`/`setAccountSettings` don't
// exist anywhere in signal-cli at all. The closest real capability is
// `updateConfiguration` (write-only, syncs read-receipts/typing-indicators/
// link-previews/unidentified-delivery-indicators prefs to linked devices -
// no corresponding "get" RPC exists for it), which is unrelated to the
// `trustMode` concept these endpoints were built around (that's
// `--trust-new-identities`, a CLI/daemon startup flag, not settable over
// JSON-RPC at all). Redesigning these four endpoints needs a product
// decision on what "configuration" should mean here, so it's left broken
// and documented rather than papered over with a same-shape rename.
async fn get_global_config(State(st): State<AppState>) -> Response {
    rpc_ok(&st, "getConfiguration", json!({})).await
}

async fn set_global_config(
    State(st): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    rpc_no_content(&st, "setConfiguration", body).await
}

async fn get_account_config(
    Path(number): Path<String>,
    State(st): State<AppState>,
) -> Response {
    rpc_ok(&st, "getAccountSettings", json!({ "account": number })).await
}

async fn set_account_config(
    Path(number): Path<String>,
    State(st): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let mut params = body;
    params["account"] = json!(number);
    rpc_no_content(&st, "setAccountSettings", params).await
}
