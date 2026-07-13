use axum::{
    Router,
    extract::{Path, Query, State},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;
use super::helpers::rpc_ok;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/search/{number}", get(search_numbers))
}

#[derive(Deserialize)]
struct SearchQuery {
    #[serde(default)]
    numbers: String,
    #[serde(default)]
    usernames: String,
}

/// GET /v1/search/{number}?numbers=...&usernames=... — check if phone numbers
/// and/or usernames are registered on Signal. getUserStatus accepts both
/// "recipient" (numbers) and "username" params independently.
async fn search_numbers(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Query(q): Query<SearchQuery>,
) -> Response {
    let recipients: Vec<&str> = q.numbers.split(',').filter(|s| !s.is_empty()).collect();
    let usernames: Vec<&str> = q.usernames.split(',').filter(|s| !s.is_empty()).collect();
    rpc_ok(
        &st,
        "getUserStatus",
        json!({ "account": number, "recipient": recipients, "username": usernames }),
    )
    .await
}
