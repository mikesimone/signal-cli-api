use crate::state::AppState;
use std::time::Duration;

/// Periodically round-trips a cheap, side-effect-free RPC call through the
/// live signal-cli connection and pings systemd's watchdog only on success.
///
/// The writer/reader loops in `jsonrpc.rs` have no reconnect logic: once the
/// pipe to signal-cli breaks (or the JVM child hangs), they just log and go
/// quiet - the HTTP server keeps running and `/v1/health` keeps returning
/// 204 (it's a pure liveness stub, unrelated to the RPC channel), while every
/// real request fails. If this loop's own RPC call fails or times out, it
/// skips the watchdog ping; after `WatchdogSec` (set in the systemd unit)
/// elapses with no ping, systemd kills and restarts the process itself
/// (`Restart=always` already brings it back cleanly), which is far simpler
/// and safer than reimplementing live TCP reconnect/state rewiring here.
pub async fn heartbeat_loop(state: AppState, interval: Duration, timeout: Duration) {
    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        match crate::jsonrpc::rpc_call(
            &state.writer_tx,
            &state.pending,
            &state.next_id,
            "listAccounts",
            serde_json::json!({}),
            timeout,
        )
        .await
        {
            Ok(_) => {
                if let Err(e) = sd_notify::notify(false, &[sd_notify::NotifyState::Watchdog]) {
                    tracing::warn!("sd_notify watchdog ping failed: {e}");
                }
            }
            Err(e) => {
                tracing::error!("watchdog heartbeat failed, skipping systemd ping: {e}");
            }
        }
    }
}
