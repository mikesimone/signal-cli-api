use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::{rpc_ok, rpc_created};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/sticker-packs/{number}", get(list_sticker_packs))
        .route("/v1/sticker-packs/{number}", post(install_sticker_pack))
        .route(
            "/v1/sticker-packs/{number}/{pack_id}/{sticker_id}",
            get(get_sticker),
        )
}

/// GET /v1/sticker-packs/{number} — list installed sticker packs.
async fn list_sticker_packs(
    State(st): State<AppState>,
    Path(number): Path<String>,
) -> Response {
    rpc_ok(&st, "listStickerPacks", json!({ "account": number })).await
}

#[derive(Deserialize)]
struct InstallStickerPackBody {
    /// A full `https://signal.art/addstickers/#pack_id=...&pack_key=...` URI.
    #[serde(default)]
    uri: Option<String>,
    /// Alternative to `uri`: the pack ID and key separately, from which the
    /// URI is constructed.
    #[serde(default)]
    pack_id: Option<String>,
    #[serde(default)]
    pack_key: Option<String>,
}

/// POST /v1/sticker-packs/{number} — install a sticker pack for this
/// account by URI (or pack ID + key).
///
/// signal-cli's `addStickerPack` RPC method takes a `uri` (or list of
/// URIs), not a bare `packId`/`packKey` pair. This previously called
/// `uploadStickerPack`, which is a *different* real command that uploads a
/// brand new custom pack from a local manifest/zip file path - passing
/// `packId`/`packKey` to it did nothing useful and would fail against real
/// signal-cli (`path` argument missing).
async fn install_sticker_pack(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<InstallStickerPackBody>,
) -> Response {
    let uri = match (&body.uri, &body.pack_id, &body.pack_key) {
        (Some(uri), _, _) => uri.clone(),
        (None, Some(pack_id), Some(pack_key)) => {
            format!("https://signal.art/addstickers/#pack_id={pack_id}&pack_key={pack_key}")
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "provide either `uri`, or both `pack_id` and `pack_key`" })),
            )
                .into_response();
        }
    };
    rpc_created(&st, "addStickerPack", json!({ "account": number, "uri": [uri] })).await
}

/// GET /v1/sticker-packs/{number}/{pack_id}/{sticker_id} — retrieve a single
/// sticker's image, base64 encoded.
async fn get_sticker(
    State(st): State<AppState>,
    Path((number, pack_id, sticker_id)): Path<(String, String, i32)>,
) -> Response {
    rpc_ok(
        &st,
        "getSticker",
        json!({ "account": number, "pack-id": pack_id, "sticker-id": sticker_id }),
    )
    .await
}
