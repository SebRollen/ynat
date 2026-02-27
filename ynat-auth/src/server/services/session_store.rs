use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::common::TokenPair;
use crate::server::models::{OAuthSession, SessionStatus};

pub struct SessionStore {
    sessions: Arc<DashMap<String, OAuthSession>>,
    ttl: Duration,
}

impl SessionStore {
    pub fn new(ttl_seconds: u64) -> Self {
        let store = Self {
            sessions: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        };

        // Spawn background cleanup task
        let sessions_clone = store.sessions.clone();
        let ttl_clone = store.ttl;
        tokio::spawn(async move {
            cleanup_expired_sessions(sessions_clone, ttl_clone).await;
        });

        tracing::info!(
            "Session store initialized with TTL of {} seconds",
            ttl_seconds
        );
        store
    }

    /// Create a new OAuth session
    pub fn create_session(&self, device_id: String, state: String) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = OAuthSession {
            session_id: session_id.clone(),
            device_id: device_id.clone(),
            state,
            status: SessionStatus::Pending,
            created_at: Utc::now(),
            tokens: None,
        };
        self.sessions.insert(session_id.clone(), session);
        tracing::debug!(
            session_id = %session_id,
            device_id = %device_id,
            "Created session"
        );
        session_id
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<OAuthSession> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    /// Update a session using a closure
    pub fn update_session<F>(&self, session_id: &str, update_fn: F) -> bool
    where
        F: FnOnce(&mut OAuthSession),
    {
        self.sessions
            .get_mut(session_id)
            .map(|mut s| {
                update_fn(&mut s);
                true
            })
            .unwrap_or(false)
    }

    /// Complete a session with tokens
    pub fn complete_session(&self, session_id: &str, tokens: TokenPair) -> bool {
        let result = self.update_session(session_id, |s| {
            s.status = SessionStatus::Completed;
            s.tokens = Some(tokens);
        });
        if result {
            tracing::debug!("Session completed: {}", session_id);
        }
        result
    }

    /// Mark a session as errored
    pub fn error_session(&self, session_id: &str, error: String) -> bool {
        let result = self.update_session(session_id, |s| {
            s.status = SessionStatus::Error(error.clone());
        });
        if result {
            tracing::warn!("Session errored: {}: {}", session_id, error);
        }
        result
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) {
        self.sessions.remove(session_id);
        tracing::debug!("Session deleted: {}", session_id);
    }

    /// Validate that device_id matches the session
    pub fn validate_device(&self, session_id: &str, device_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.device_id == device_id)
            .unwrap_or(false)
    }

    /// Get session count (for monitoring)
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Background task that periodically cleans up expired sessions
async fn cleanup_expired_sessions(sessions: Arc<DashMap<String, OAuthSession>>, ttl: Duration) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|session_id, session| {
            let age = now
                .signed_duration_since(session.created_at)
                .to_std()
                .unwrap_or(Duration::ZERO);

            if age >= ttl {
                tracing::debug!(
                    session_id = %session_id,
                    device_id = %session.device_id,
                    "Cleaning up expired session"
                );
                false
            } else {
                true
            }
        });

        let cleaned = initial_count.saturating_sub(sessions.len());
        if cleaned > 0 {
            tracing::info!(
                "Cleaned up {} expired sessions, {} remaining",
                cleaned,
                sessions.len()
            );
        }
    }
}
