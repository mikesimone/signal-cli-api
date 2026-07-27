use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::put;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::avatar::spill_avatar_to_disk;
use super::helpers::rpc_ok;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/profiles/{number}", put(update_profile))
}

#[derive(Deserialize)]
struct UpdateProfileBody {
    name: Option<String>,
    about: Option<String>,
    base64_avatar: Option<String>,
}

async fn update_profile(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<UpdateProfileBody>,
) -> Response {
    let mut params = json!({ "account": number });
    if let Some(name) = &body.name {
        params["given-name"] = json!(name);
    }
    if let Some(about) = &body.about {
        params["about"] = json!(about);
    }
    let _spilled = if let Some(avatar) = &body.base64_avatar {
        match spill_avatar_to_disk(avatar) {
            Ok((path, guard)) => {
                params["avatar"] = json!(path.to_string_lossy());
                Some(guard)
            }
            Err(e) => {
                return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response();
            }
        }
    } else {
        None
    };
    rpc_ok(&st, "updateProfile", params).await
}
