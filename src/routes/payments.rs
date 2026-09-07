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
use super::helpers::rpc_created;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/payments/{number}", post(send_payment_notification))
}

#[derive(Deserialize)]
struct PaymentNotificationBody {
    recipient: String,
    /// Base64-encoded MobileCoin receipt blob.
    receipt: String,
    #[serde(default)]
    note: Option<String>,
}

/// POST /v1/payments/{number} — send a payment notification message.
async fn send_payment_notification(
    State(st): State<AppState>,
    Path(number): Path<String>,
    Json(body): Json<PaymentNotificationBody>,
) -> Response {
    let mut params = json!({
        "account": number,
        "recipient": body.recipient,
        "receipt": body.receipt,
    });
    if let Some(note) = &body.note {
        params["note"] = json!(note);
    }
    rpc_created(&st, "sendPaymentNotification", params).await
}
