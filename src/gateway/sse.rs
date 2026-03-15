//! Server-Sent Events (SSE) stream for real-time event delivery.
//!
//! Wraps the broadcast channel in AppState to deliver events to web dashboard clients.

use super::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// GET /api/events — SSE event stream
pub async fn handle_sse_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Auth check
    if state.pairing.require_pairing() {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .unwrap_or("");

        if !state.pairing.is_authenticated(token) {
            return (
                StatusCode::UNAUTHORIZED,
                "Unauthorized — provide Authorization: Bearer <token>",
            )
                .into_response();
        }
    }

    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(
        |result: Result<
            serde_json::Value,
            tokio_stream::wrappers::errors::BroadcastStreamRecvError,
        >| {
            match result {
                Ok(value) => Some(Ok::<_, Infallible>(
                    Event::default().data(value.to_string()),
                )),
                Err(_) => None, // Skip lagged messages
            }
        },
    );

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// Broadcast observer that forwards events to the SSE broadcast channel.
pub struct BroadcastObserver {
    inner: Box<dyn crate::observability::Observer>,
    tx: tokio::sync::broadcast::Sender<serde_json::Value>,
}

impl BroadcastObserver {
    pub fn new(
        inner: Box<dyn crate::observability::Observer>,
        tx: tokio::sync::broadcast::Sender<serde_json::Value>,
    ) -> Self {
        Self { inner, tx }
    }
}

impl crate::observability::Observer for BroadcastObserver {
    fn record_event(&self, event: &crate::observability::ObserverEvent) {
        // Forward to inner observer
        self.inner.record_event(event);

        // Broadcast to SSE subscribers
        let json = match event {
            crate::observability::ObserverEvent::LlmRequest {
                provider, model, ..
            } => serde_json::json!({
                "type": "llm_request",
                "provider": provider,
                "model": model,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::ToolCall {
                tool,
                duration,
                success,
            } => serde_json::json!({
                "type": "tool_call",
                "tool": tool,
                "duration_ms": duration.as_millis(),
                "success": success,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::ToolCallStart { tool } => serde_json::json!({
                "type": "tool_call_start",
                "tool": tool,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            crate::observability::ObserverEvent::Error { component, message } => {
                serde_json::json!({
                    "type": "error",
                    "component": component,
                    "message": message,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
            }
            crate::observability::ObserverEvent::AgentStart { provider, model } => {
                serde_json::json!({
                    "type": "agent_start",
                    "provider": provider,
                    "model": model,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
            }
            crate::observability::ObserverEvent::AgentEnd {
                provider,
                model,
                duration,
                tokens_used,
                cost_usd,
            } => serde_json::json!({
                "type": "agent_end",
                "provider": provider,
                "model": model,
                "duration_ms": duration.as_millis(),
                "tokens_used": tokens_used,
                "cost_usd": cost_usd,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            _ => return, // Skip events we don't broadcast
        };

        let _ = self.tx.send(json);
    }

    fn record_metric(&self, metric: &crate::observability::traits::ObserverMetric) {
        self.inner.record_metric(metric);
    }

    fn flush(&self) {
        self.inner.flush();
    }

    fn name(&self) -> &str {
        "broadcast"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::{Observer, ObserverEvent};
    use std::time::Duration;

    #[test]
    fn broadcast_observer_name_is_broadcast() {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );
        assert_eq!(obs.name(), "broadcast");
    }

    #[test]
    fn broadcast_observer_forwards_llm_request_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::LlmRequest {
            provider: "openai".into(),
            model: "gpt-4".into(),
            messages_count: 2,
        });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "llm_request");
        assert_eq!(json["provider"], "openai");
        assert_eq!(json["model"], "gpt-4");
        assert!(json["timestamp"].as_str().is_some());
    }

    #[test]
    fn broadcast_observer_forwards_tool_call_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::ToolCall {
            tool: "shell".into(),
            duration: Duration::from_millis(42),
            success: true,
        });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "tool_call");
        assert_eq!(json["tool"], "shell");
        assert_eq!(json["duration_ms"], 42);
        assert_eq!(json["success"], true);
    }

    #[test]
    fn broadcast_observer_forwards_tool_call_start_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::ToolCallStart { tool: "web_fetch".into() });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "tool_call_start");
        assert_eq!(json["tool"], "web_fetch");
    }

    #[test]
    fn broadcast_observer_forwards_error_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::Error {
            component: "gateway".into(),
            message: "connection refused".into(),
        });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["component"], "gateway");
        assert_eq!(json["message"], "connection refused");
    }

    #[test]
    fn broadcast_observer_forwards_agent_start_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::AgentStart {
            provider: "anthropic".into(),
            model: "claude-3".into(),
        });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "agent_start");
        assert_eq!(json["provider"], "anthropic");
        assert_eq!(json["model"], "claude-3");
    }

    #[test]
    fn broadcast_observer_forwards_agent_end_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::AgentEnd {
            provider: "anthropic".into(),
            model: "claude-3".into(),
            duration: Duration::from_secs(5),
            tokens_used: Some(1500),
            cost_usd: Some(0.03),
        });

        let json = rx.try_recv().unwrap();
        assert_eq!(json["type"], "agent_end");
        assert_eq!(json["provider"], "anthropic");
        assert_eq!(json["duration_ms"], 5000);
        assert_eq!(json["tokens_used"], 1500);
    }

    #[test]
    fn broadcast_observer_skips_heartbeat_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        obs.record_event(&ObserverEvent::HeartbeatTick);

        // Heartbeat events are skipped (fall into the _ => return branch)
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn broadcast_observer_delegates_flush_to_inner() {
        let (tx, _rx) = tokio::sync::broadcast::channel(16);
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );
        // Flush should not panic
        obs.flush();
    }

    #[test]
    fn broadcast_observer_no_panic_when_no_subscribers() {
        let (tx, _) = tokio::sync::broadcast::channel::<serde_json::Value>(16);
        // Drop the receiver — send should silently fail
        let obs = BroadcastObserver::new(
            Box::new(crate::observability::NoopObserver),
            tx,
        );

        // Should not panic even with no subscribers
        obs.record_event(&ObserverEvent::Error {
            component: "test".into(),
            message: "no one listening".into(),
        });
    }
}
