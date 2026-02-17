use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AdminSession {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Clone, Debug)]
pub struct UserSession {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: u64,
}

#[derive(Clone)]
pub struct Sessions {
    inner: Arc<Mutex<HashMap<String, AdminSession>>>,
    user_sessions: Arc<Mutex<HashMap<String, UserSession>>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            user_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create(&self, ttl_seconds: u64) -> AdminSession {
        let now = unix_now();
        let token = Uuid::new_v4().to_string();
        let session = AdminSession {
            token: token.clone(),
            expires_at: now + ttl_seconds,
        };
        let mut map = self.inner.lock().expect("sessions mutex");
        map.insert(token.clone(), session.clone());
        session
    }

    pub fn validate(&self, token: &str) -> bool {
        let map = self.inner.lock().expect("sessions mutex");
        if let Some(session) = map.get(token) {
            unix_now() < session.expires_at
        } else {
            false
        }
    }

    pub fn create_user_session(&self, user_id: Uuid, ttl_seconds: u64) -> UserSession {
        let now = unix_now();
        let token = Uuid::new_v4().to_string();
        let session = UserSession {
            token: token.clone(),
            user_id,
            expires_at: now + ttl_seconds,
        };
        let mut map = self.user_sessions.lock().expect("user sessions mutex");
        map.insert(token.clone(), session.clone());
        session
    }

    pub fn validate_user_session(&self, token: &str) -> Option<Uuid> {
        let map = self.user_sessions.lock().expect("user sessions mutex");
        if let Some(session) = map.get(token) {
            if unix_now() < session.expires_at {
                return Some(session.user_id);
            }
        }
        None
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_secs()
}
