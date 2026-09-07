# AI Project Context

## Purpose
`signal-cli-api` is a native Rust/axum REST, streaming, webhook, and metrics wrapper around signal-cli. It is used by the Signal automation stack but is also a standalone general-purpose project.

## Architecture
- Rust + axum HTTP API.
- Maintains a persistent connection to signal-cli rather than restarting the JVM for every request.
- Supports WebSocket, SSE, webhooks, Prometheus metrics, TLS, groups/contacts/accounts/devices, and message operations.
- Integration tests use a mock signal-cli daemon and should not require a real Signal account.

## Local deployment context
- Six packages this API together with signal-cli for `mikesimone/signal-forwarder`.
- Multiple Signal accounts share the production daemon, so account-selection semantics are important to downstream callers.

## Working rules
- Read `README.md` before API or CLI changes.
- Preserve backward compatibility for existing endpoints unless a breaking change is explicitly requested.
- Keep request tracing/observability intact.
- Do not add real Signal credentials or account state to tests or Git.
- Run `cargo test` for behavioral changes and update docs/OpenAPI-facing behavior when endpoints or flags change.
