//! WebSocket agent chat handler.
//!
//! Protocol:
//! ```text
//! Client -> Server: {"type":"message","content":"Hello"}
//! Server -> Client: {"type":"chunk","content":"Hi! "}
//! Server -> Client: {"type":"tool_call","name":"shell","args":{...}}
//! Server -> Client: {"type":"tool_result","name":"shell","output":"..."}
//! Server -> Client: {"type":"done","full_response":"..."}
//! ```

use super::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// Parse incoming WebSocket text and return the user content if valid, or an
/// error JSON string for invalid input.  Factored out of `handle_socket` so the
/// protocol parsing logic is directly unit-testable.
fn parse_ws_message(raw: &str) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(raw).map_err(|_| {
        serde_json::json!({"type": "error", "message": "Invalid JSON"}).to_string()
    })?;

    let msg_type = parsed["type"].as_str().unwrap_or("");
    if msg_type != "message" {
        return Err(String::new()); // silently skip
    }

    let content = parsed["content"].as_str().unwrap_or("").to_string();
    if content.is_empty() {
        return Err(String::new()); // silently skip
    }

    Ok(content)
}

/// GET /ws/chat — WebSocket upgrade for agent chat
pub async fn handle_ws_chat(
    State(state): State<AppState>,
    Query(params): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Auth via query param (browser WebSocket limitation)
    if state.pairing.require_pairing() {
        let token = params.token.as_deref().unwrap_or("");
        if !state.pairing.is_authenticated(token) {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Unauthorized — provide ?token=<bearer_token>",
            )
                .into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
        .into_response()
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => continue,
        };

        let content = match parse_ws_message(&msg) {
            Ok(c) => c,
            Err(err_json) => {
                if !err_json.is_empty() {
                    let _ = sender.send(Message::Text(err_json.into())).await;
                }
                continue;
            }
        };

        // Process message with the LLM provider
        let provider_label = state
            .config
            .lock()
            .default_provider
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // Broadcast agent_start event
        let _ = state.event_tx.send(serde_json::json!({
            "type": "agent_start",
            "provider": provider_label,
            "model": state.model,
        }));

        // Simple single-turn chat (no streaming for now — use provider.chat_with_system)
        let system_prompt = {
            let config_guard = state.config.lock();
            crate::channels::build_system_prompt(
                &config_guard.workspace_dir,
                &state.model,
                &[],
                &[],
                Some(&config_guard.identity),
                None,
            )
        };

        let messages = vec![
            crate::providers::ChatMessage::system(system_prompt),
            crate::providers::ChatMessage::user(&content),
        ];

        let multimodal_config = state.config.lock().multimodal.clone();
        let prepared =
            match crate::multimodal::prepare_messages_for_provider(&messages, &multimodal_config)
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    let err = serde_json::json!({
                        "type": "error",
                        "message": format!("Multimodal prep failed: {e}")
                    });
                    let _ = sender.send(Message::Text(err.to_string().into())).await;
                    continue;
                }
            };

        match state
            .provider
            .chat_with_history(&prepared.messages, &state.model, state.temperature)
            .await
        {
            Ok(response) => {
                // Send the full response as a done message
                let done = serde_json::json!({
                    "type": "done",
                    "full_response": response,
                });
                let _ = sender.send(Message::Text(done.to_string().into())).await;

                // Broadcast agent_end event
                let _ = state.event_tx.send(serde_json::json!({
                    "type": "agent_end",
                    "provider": provider_label,
                    "model": state.model,
                }));
            }
            Err(e) => {
                let sanitized = crate::providers::sanitize_api_error(&e.to_string());
                let err = serde_json::json!({
                    "type": "error",
                    "message": sanitized,
                });
                let _ = sender.send(Message::Text(err.to_string().into())).await;

                // Broadcast error event
                let _ = state.event_tx.send(serde_json::json!({
                    "type": "error",
                    "component": "ws_chat",
                    "message": sanitized,
                }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_query_token_is_optional() {
        let q: WsQuery = serde_json::from_str("{}").unwrap();
        assert!(q.token.is_none());

        let q: WsQuery = serde_json::from_str(r#"{"token":"abc"}"#).unwrap();
        assert_eq!(q.token.as_deref(), Some("abc"));
    }

    #[test]
    fn parse_ws_message_rejects_invalid_json() {
        let result = parse_ws_message("not json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid JSON"));
    }

    #[test]
    fn parse_ws_message_skips_non_message_types() {
        let result = parse_ws_message(r#"{"type":"ping","content":"hello"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_empty(), "non-message types should be silently skipped");
    }

    #[test]
    fn parse_ws_message_skips_missing_type() {
        let result = parse_ws_message(r#"{"content":"hello"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_empty());
    }

    #[test]
    fn parse_ws_message_skips_empty_content() {
        let result = parse_ws_message(r#"{"type":"message","content":""}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_empty());
    }

    #[test]
    fn parse_ws_message_skips_missing_content() {
        let result = parse_ws_message(r#"{"type":"message"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_empty());
    }

    #[test]
    fn parse_ws_message_extracts_valid_content() {
        let result = parse_ws_message(r#"{"type":"message","content":"Hello world"}"#);
        assert_eq!(result.unwrap(), "Hello world");
    }

    #[test]
    fn parse_ws_message_handles_unicode_content() {
        let result = parse_ws_message(r#"{"type":"message","content":"resonance \u00e9tude"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn protocol_done_response_has_expected_shape() {
        let response = "mock response";
        let done = serde_json::json!({
            "type": "done",
            "full_response": response,
        });
        assert_eq!(done["type"], "done");
        assert_eq!(done["full_response"], "mock response");
    }

    #[test]
    fn protocol_error_response_has_expected_shape() {
        let err = serde_json::json!({
            "type": "error",
            "message": "something went wrong",
        });
        assert_eq!(err["type"], "error");
        assert!(err["message"].as_str().unwrap().contains("something went wrong"));
    }
}
