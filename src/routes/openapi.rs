use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/openapi.json", get(openapi_spec))
}

async fn openapi_spec() -> Response {
    let spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": "signal-cli REST API",
            "description": "REST API bridge for signal-cli",
            "version": env!("CARGO_PKG_VERSION")
        },
        "paths": {
            "/v2/send": {
                "post": {
                    "tags": ["Messages"],
                    "summary": "Send a message",
                    "description": "Pure passthrough to signal-cli's `send` JSON-RPC method - the request body is forwarded almost verbatim (attachments are the one exception: large base64 data URIs are spilled to disk first). Any field signal-cli's `send` accepts works here, documented or not; see the README's \"The `send` contract\" section for the full list of confirmed-working fields and why this is a deliberate design choice, not an oversight.",
                    "operationId": "send",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/SendPayload" }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "Message sent" },
                        "400": { "description": "Invalid request" }
                    }
                }
            },
            "/v1/send": {
                "post": {
                    "tags": ["Messages"],
                    "summary": "Send a message (deprecated alias of /v2/send)",
                    "description": "Identical passthrough behavior to /v2/send. Kept for backward compatibility.",
                    "operationId": "sendV1",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/SendPayload" }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "Message sent" },
                        "400": { "description": "Invalid request" }
                    }
                }
            },
            "/v1/receive/{number}": {
                "get": {
                    "tags": ["Messages"],
                    "summary": "Receive messages",
                    "operationId": "receive",
                    "parameters": [{
                        "name": "number",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string" }
                    }],
                    "responses": {
                        "200": { "description": "Array of messages" }
                    }
                }
            },
            "/v1/remote-delete/{number}": {
                "delete": {
                    "tags": ["Messages"],
                    "summary": "Remotely delete a sent message",
                    "operationId": "remoteDelete",
                    "parameters": [{
                        "name": "number",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string" }
                    }],
                    "responses": {
                        "200": { "description": "Delete sent" }
                    }
                }
            },
            "/v1/health": {
                "get": {
                    "tags": ["System"],
                    "summary": "Health check",
                    "operationId": "health",
                    "responses": {
                        "204": { "description": "Healthy" }
                    }
                }
            },
            "/v1/about": {
                "get": {
                    "tags": ["System"],
                    "summary": "API version info",
                    "operationId": "about",
                    "responses": {
                        "200": { "description": "Version information" }
                    }
                }
            },
            "/v1/version": {
                "get": {
                    "tags": ["System"],
                    "summary": "signal-cli's own reported version",
                    "description": "Backed by signal-cli's `version` JSON-RPC method, distinct from /v1/about which reports this wrapper's own build info.",
                    "operationId": "version",
                    "responses": {
                        "200": { "description": "signal-cli version information" }
                    }
                }
            },
            "/v1/groups/{number}": {
                "get": {
                    "tags": ["Groups"],
                    "summary": "List groups for an account",
                    "operationId": "listGroups",
                    "parameters": [{
                        "name": "number",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string" }
                    }],
                    "responses": {
                        "200": { "description": "Array of groups" }
                    }
                }
            },
            "/v1/groups/{number}/{groupid}/avatar": {
                "get": {
                    "tags": ["Groups"],
                    "summary": "Get a group's avatar",
                    "description": "Backed by signal-cli's `getAvatar` JSON-RPC method.",
                    "operationId": "getGroupAvatar",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "groupid", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Base64-encoded avatar image",
                            "content": { "application/json": { "schema": { "$ref": "#/components/schemas/AttachmentData" } } }
                        }
                    }
                }
            },
            "/v1/groups/{number}/{groupid}/messages": {
                "delete": {
                    "tags": ["Groups"],
                    "summary": "Admin-delete a message in a group",
                    "description": "Backed by signal-cli's `sendAdminDelete` JSON-RPC method. Requires this account to be a group admin.",
                    "operationId": "adminDeleteGroupMessage",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "groupid", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["target_author", "target_timestamp"],
                            "properties": {
                                "target_author": { "type": "string" },
                                "target_timestamp": { "type": "integer" },
                                "story": { "type": "boolean" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Admin-delete sent" } }
                }
            },
            "/v1/contacts/{number}/{recipient}": {
                "delete": {
                    "tags": ["Contacts"],
                    "summary": "Remove a contact",
                    "description": "Backed by signal-cli's `removeContact` JSON-RPC method.",
                    "operationId": "removeContact",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "recipient", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": false,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "hide": { "type": "boolean", "description": "Hide the contact but keep its data" },
                                "forget": { "type": "boolean", "description": "Delete all data, including identity keys and sessions" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Contact removed" } }
                }
            },
            "/v1/contacts/{number}/{recipient}/avatar": {
                "get": {
                    "tags": ["Contacts"],
                    "summary": "Get a contact's avatar",
                    "description": "Backed by signal-cli's `getAvatar` JSON-RPC method.",
                    "operationId": "getContactAvatar",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "recipient", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Base64-encoded avatar image",
                            "content": { "application/json": { "schema": { "$ref": "#/components/schemas/AttachmentData" } } }
                        }
                    }
                }
            },
            "/v1/contacts/{number}/{recipient}/block": {
                "post": {
                    "tags": ["Contacts"],
                    "summary": "Block a contact",
                    "operationId": "blockContact",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "recipient", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "Contact blocked" } }
                },
                "delete": {
                    "tags": ["Contacts"],
                    "summary": "Unblock a contact",
                    "operationId": "unblockContact",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "recipient", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "Contact unblocked" } }
                }
            },
            "/v1/contacts/{number}/message-requests": {
                "put": {
                    "tags": ["Contacts"],
                    "summary": "Accept or delete a pending message request",
                    "description": "Backed by signal-cli's `sendMessageRequestResponse` JSON-RPC method.",
                    "operationId": "messageRequestResponse",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "group_id": { "type": "string" },
                                "type": { "type": "string", "enum": ["accept", "delete"] }
                            }
                        } } }
                    },
                    "responses": { "204": { "description": "Response sent" } }
                }
            },
            "/v1/accounts/{number}/settings": {
                "put": {
                    "tags": ["Accounts"],
                    "summary": "Update account attributes",
                    "description": "Backed by signal-cli's `updateAccount` JSON-RPC method - device name, unidentified-sender/discoverability/number-sharing preferences. Not a runtime 'trust mode' setting (signal-cli has no such RPC; `--trust-new-identities` is a CLI/daemon startup flag only).",
                    "operationId": "updateAccountSettings",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "device_name": { "type": "string" },
                                "unrestricted_unidentified_sender": { "type": "boolean" },
                                "discoverable_by_number": { "type": "boolean" },
                                "number_sharing": { "type": "boolean" }
                            }
                        } } }
                    },
                    "responses": { "204": { "description": "Updated" } }
                }
            },
            "/v1/accounts/{number}/sync-request": {
                "post": {
                    "tags": ["Accounts"],
                    "summary": "Request a full sync from the primary device",
                    "operationId": "sendSyncRequest",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "204": { "description": "Sync requested" } }
                }
            },
            "/v1/accounts/{number}/number": {
                "post": {
                    "tags": ["Accounts"],
                    "summary": "Start a phone number change",
                    "operationId": "startChangeNumber",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["number"],
                            "properties": {
                                "number": { "type": "string", "description": "New E164 phone number" },
                                "voice": { "type": "boolean" },
                                "captcha": { "type": "string" }
                            }
                        } } }
                    },
                    "responses": { "204": { "description": "Verification code sent" } }
                }
            },
            "/v1/accounts/{number}/number/verify": {
                "post": {
                    "tags": ["Accounts"],
                    "summary": "Finish a phone number change",
                    "operationId": "finishChangeNumber",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["number", "verification_code"],
                            "properties": {
                                "number": { "type": "string" },
                                "verification_code": { "type": "string" },
                                "pin": { "type": "string" }
                            }
                        } } }
                    },
                    "responses": { "204": { "description": "Number changed" } }
                }
            },
            "/v1/devices/{number}/{device_id}": {
                "put": {
                    "tags": ["Devices"],
                    "summary": "Rename a linked device",
                    "operationId": "updateDevice",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "device_id", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["device_name"],
                            "properties": { "device_name": { "type": "string" } }
                        } } }
                    },
                    "responses": { "204": { "description": "Renamed" } }
                }
            },
            "/v1/devices/{number}/add": {
                "post": {
                    "tags": ["Devices"],
                    "summary": "Approve linking a new device (primary device only)",
                    "description": "Backed by signal-cli's `addDevice` JSON-RPC method. Only works if this account is the primary device.",
                    "operationId": "addDevice",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["uri"],
                            "properties": { "uri": { "type": "string", "description": "URI from the QR code shown by the new device" } }
                        } } }
                    },
                    "responses": { "204": { "description": "Device linked" } }
                }
            },
            "/v1/polls/{number}": {
                "post": {
                    "tags": ["Polls"],
                    "summary": "Create and send a poll",
                    "description": "Backed by signal-cli's `sendPollCreate` JSON-RPC method.",
                    "operationId": "createPoll",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/CreatePollPayload" } } }
                    },
                    "responses": { "201": { "description": "Poll sent" } }
                },
                "delete": {
                    "tags": ["Polls"],
                    "summary": "Terminate (close) a poll",
                    "description": "Backed by signal-cli's `sendPollTerminate` JSON-RPC method. Addressed by the poll message's own timestamp - signal-cli has no server-side poll ID concept.",
                    "operationId": "closePoll",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["poll_timestamp"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "group_id": { "type": "string" },
                                "poll_timestamp": { "type": "integer" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Poll terminated" } }
                }
            },
            "/v1/polls/{number}/vote": {
                "post": {
                    "tags": ["Polls"],
                    "summary": "Vote on a poll",
                    "description": "Backed by signal-cli's `sendPollVote` JSON-RPC method.",
                    "operationId": "votePoll",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["poll_author", "poll_timestamp", "option"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "group_id": { "type": "string" },
                                "poll_author": { "type": "string" },
                                "poll_timestamp": { "type": "integer" },
                                "option": { "type": "integer" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Vote sent" } }
                }
            },
            "/v1/sticker-packs/{number}": {
                "get": {
                    "tags": ["Stickers"],
                    "summary": "List installed sticker packs",
                    "operationId": "listStickerPacks",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Array of sticker packs" } }
                },
                "post": {
                    "tags": ["Stickers"],
                    "summary": "Install a sticker pack",
                    "description": "Backed by signal-cli's `addStickerPack` JSON-RPC method. Accepts either a full signal.art URI, or a pack_id + pack_key pair from which the URI is constructed.",
                    "operationId": "installStickerPack",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": {
                                "uri": { "type": "string" },
                                "pack_id": { "type": "string" },
                                "pack_key": { "type": "string" }
                            }
                        } } }
                    },
                    "responses": { "201": { "description": "Sticker pack installed" } }
                }
            },
            "/v1/sticker-packs/{number}/{pack_id}/{sticker_id}": {
                "get": {
                    "tags": ["Stickers"],
                    "summary": "Get a single sticker",
                    "description": "Backed by signal-cli's `getSticker` JSON-RPC method.",
                    "operationId": "getSticker",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "pack_id", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "sticker_id", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Base64-encoded sticker image",
                            "content": { "application/json": { "schema": { "$ref": "#/components/schemas/AttachmentData" } } }
                        }
                    }
                }
            },
            "/v1/stories/{number}": {
                "post": {
                    "tags": ["Stories"],
                    "summary": "Post a story",
                    "description": "Backed by signal-cli's `sendStory` JSON-RPC method.",
                    "operationId": "sendStory",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["attachment"],
                            "properties": {
                                "attachment": { "type": "string", "description": "Data URI or file path" },
                                "group_id": { "type": "string" },
                                "no_replies": { "type": "boolean" }
                            }
                        } } }
                    },
                    "responses": { "201": { "description": "Story posted" } }
                }
            },
            "/v1/pins/{number}": {
                "post": {
                    "tags": ["Messages"],
                    "summary": "Pin a message",
                    "description": "Backed by signal-cli's `sendPinMessage` JSON-RPC method.",
                    "operationId": "pinMessage",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["target_author", "target_timestamp"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "group_id": { "type": "string" },
                                "target_author": { "type": "string" },
                                "target_timestamp": { "type": "integer" },
                                "pin_duration": { "type": "integer", "description": "Seconds; -1 = forever" },
                                "story": { "type": "boolean" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Pin sent" } }
                }
            },
            "/v1/pins/{number}/unpin": {
                "post": {
                    "tags": ["Messages"],
                    "summary": "Unpin a message",
                    "description": "Backed by signal-cli's `sendUnpinMessage` JSON-RPC method.",
                    "operationId": "unpinMessage",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["target_author", "target_timestamp"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "group_id": { "type": "string" },
                                "target_author": { "type": "string" },
                                "target_timestamp": { "type": "integer" },
                                "story": { "type": "boolean" }
                            }
                        } } }
                    },
                    "responses": { "200": { "description": "Unpin sent" } }
                }
            },
            "/v1/payments/{number}": {
                "post": {
                    "tags": ["Messages"],
                    "summary": "Send a payment notification",
                    "description": "Backed by signal-cli's `sendPaymentNotification` JSON-RPC method.",
                    "operationId": "sendPaymentNotification",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["recipient", "receipt"],
                            "properties": {
                                "recipient": { "type": "string" },
                                "receipt": { "type": "string", "description": "Base64-encoded MobileCoin receipt blob" },
                                "note": { "type": "string" }
                            }
                        } } }
                    },
                    "responses": { "201": { "description": "Payment notification sent" } }
                }
            },
            "/v1/calls/{number}": {
                "get": {
                    "tags": ["Calls"],
                    "summary": "List active calls",
                    "operationId": "listCalls",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Array of active calls" } }
                },
                "post": {
                    "tags": ["Calls"],
                    "summary": "Start an outgoing call",
                    "operationId": "startCall",
                    "parameters": [{ "name": "number", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["recipient"],
                            "properties": { "recipient": { "type": "string" } }
                        } } }
                    },
                    "responses": { "201": { "description": "Call started" } }
                }
            },
            "/v1/calls/{number}/{call_id}/accept": {
                "post": {
                    "tags": ["Calls"],
                    "summary": "Accept an incoming call",
                    "operationId": "acceptCall",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "call_id", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": { "200": { "description": "Call accepted" } }
                }
            },
            "/v1/calls/{number}/{call_id}/reject": {
                "post": {
                    "tags": ["Calls"],
                    "summary": "Reject an incoming call",
                    "operationId": "rejectCall",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "call_id", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": { "200": { "description": "Call rejected" } }
                }
            },
            "/v1/calls/{number}/{call_id}/hangup": {
                "post": {
                    "tags": ["Calls"],
                    "summary": "Hang up an active call",
                    "operationId": "hangupCall",
                    "parameters": [
                        { "name": "number", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "call_id", "in": "path", "required": true, "schema": { "type": "integer" } }
                    ],
                    "responses": { "200": { "description": "Call hung up" } }
                }
            },
            "/v1/webhooks": {
                "get": {
                    "tags": ["Webhooks"],
                    "summary": "List registered webhooks",
                    "operationId": "listWebhooks",
                    "responses": {
                        "200": { "description": "Array of webhook configs" }
                    }
                },
                "post": {
                    "tags": ["Webhooks"],
                    "summary": "Register a webhook",
                    "operationId": "createWebhook",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/WebhookConfig" }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "Webhook registered" }
                    }
                }
            },
            "/v1/events/{number}": {
                "get": {
                    "tags": ["Events"],
                    "summary": "Server-Sent Events stream",
                    "operationId": "sseEvents",
                    "parameters": [{
                        "name": "number",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string" }
                    }],
                    "responses": {
                        "200": { "description": "SSE stream of messages" }
                    }
                }
            },
            "/metrics": {
                "get": {
                    "tags": ["System"],
                    "summary": "Prometheus metrics",
                    "operationId": "metrics",
                    "responses": {
                        "200": {
                            "description": "Prometheus-formatted metrics",
                            "content": {
                                "text/plain": {
                                    "schema": { "type": "string" }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "SendPayload": {
                    "type": "object",
                    "required": ["account", "message"],
                    "description": "Forwarded almost verbatim to signal-cli's `send` JSON-RPC method - this schema documents the commonly-used fields, but any field signal-cli's `send` accepts is passed through, whether listed here or not. See the README's \"The `send` contract\" section.",
                    "properties": {
                        "account": { "type": "string", "description": "Sending account, required. NOT `number` - signal-cli only recognizes `account`, and on a multi-account daemon a `number` field is silently ignored." },
                        "message": { "type": "string", "description": "Message text" },
                        "recipient": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Recipient numbers. `recipients` (plural) also works, via signal-cli's own pluralization fallback."
                        },
                        "groupId": { "type": "string", "description": "Send to a group instead of recipient" },
                        "username": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Recipient usernames or username links"
                        },
                        "noteToSelf": { "type": "boolean" },
                        "attachment": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Data URIs (data:<mime>;filename=<name>;base64,<data>) or file paths. NOT `base64_attachments` - that field is ignored."
                        },
                        "editTimestamp": { "type": "integer", "description": "Edit a previously-sent message in place" },
                        "quoteTimestamp": { "type": "integer" },
                        "quoteAuthor": { "type": "string" },
                        "quoteMessage": { "type": "string" },
                        "storyTimestamp": { "type": "integer" },
                        "storyAuthor": { "type": "string" },
                        "mention": { "type": "array", "items": { "type": "string" }, "description": "start:length:recipient" },
                        "textStyle": { "type": "array", "items": { "type": "string" }, "description": "start:length:STYLE" },
                        "noUrgent": { "type": "boolean" },
                        "viewOnce": { "type": "boolean" },
                        "sticker": { "type": "string", "description": "packId:stickerId" },
                        "previewUrl": { "type": "string" },
                        "previewTitle": { "type": "string" },
                        "previewDescription": { "type": "string" },
                        "previewImage": { "type": "string" },
                        "voiceNote": { "type": "boolean" }
                    }
                },
                "CreatePollPayload": {
                    "type": "object",
                    "required": ["question", "options"],
                    "properties": {
                        "recipient": { "type": "string" },
                        "group_id": { "type": "string" },
                        "question": { "type": "string" },
                        "options": { "type": "array", "items": { "type": "string" } },
                        "allow_multiple": { "type": "boolean", "description": "Default true; set false to allow only one selection per recipient" }
                    }
                },
                "AttachmentData": {
                    "type": "object",
                    "properties": {
                        "data": { "type": "string", "description": "Base64-encoded file contents" }
                    }
                },
                "WebhookConfig": {
                    "type": "object",
                    "required": ["url"],
                    "properties": {
                        "id": { "type": "string", "description": "Webhook ID (server-generated)" },
                        "url": { "type": "string", "description": "Callback URL" },
                        "events": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Event types to subscribe to (empty = all)"
                        }
                    }
                }
            }
        }
    });

    Json(spec).into_response()
}
