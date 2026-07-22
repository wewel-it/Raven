use crate::agent::runtime::state::{LifecycleState, SessionState};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Minimal session manager storing sessions in memory (thread-safe).
#[derive(Clone)]
pub struct Session {
    pub session_id: String,
    pub conversation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: SessionState,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.into(),
            conversation_id: None,
            created_at: now,
            updated_at: now,
            state: SessionState::new(),
            metadata: HashMap::new(),
        }
    }
}

pub struct SessionManager {
    inner: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(&self, id: impl Into<String>) -> String {
        let id = id.into();
        let s = Session::new(id.clone());
        // Handle poisoned lock by recovering the inner map when possible.
        let mut guard = match self.inner.write() {
            Ok(g) => g,
            Err(poison) => poison.into_inner(),
        };
        guard.insert(id.clone(), s);
        id
    }

    pub fn get(&self, id: &str) -> Option<Session> {
        let r = self.inner.read().unwrap();
        r.get(id).cloned()
    }

    pub fn set_state(&self, id: &str, state: LifecycleState) {
        if let Ok(mut w) = self.inner.write() {
            if let Some(s) = w.get_mut(id) {
                s.state.set(state);
                s.updated_at = Utc::now();
            }
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
