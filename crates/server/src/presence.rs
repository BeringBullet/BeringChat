use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct PresenceStore {
    inner: Arc<Mutex<HashMap<Uuid, usize>>>,
    remote: Arc<Mutex<HashSet<String>>>,
}

impl PresenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_remote_users_online(&self, usernames: Vec<String>) {
        let mut remote = self.remote.lock().expect("remote presence mutex");
        // Add all usernames (they already include @server suffix)
        for username in usernames {
            remote.insert(username);
        }
    }
    
    pub fn clear_remote_server(&self, server_name: &str) -> bool {
        let mut remote = self.remote.lock().expect("remote presence mutex");
        let len_before = remote.len();
        remote.retain(|username| !username.ends_with(&format!("@{}", server_name)));
        remote.len() != len_before
    }

    pub fn is_remote_user_online(&self, username: &str) -> bool {
        let remote = self.remote.lock().expect("remote presence mutex");
        remote.contains(username)
    }

    pub fn mark_online(&self, user_id: Uuid) {
        let mut map = self.inner.lock().expect("presence mutex");
        let count = map.entry(user_id).or_insert(0);
        *count += 1;
    }

    pub fn mark_offline(&self, user_id: Uuid) {
        let mut map = self.inner.lock().expect("presence mutex");
        if let Some(count) = map.get_mut(&user_id) {
            if *count <= 1 {
                map.remove(&user_id);
            } else {
                *count -= 1;
            }
        }
    }

    pub fn is_online(&self, user_id: Uuid) -> bool {
        let map = self.inner.lock().expect("presence mutex");
        map.get(&user_id).map(|count| *count > 0).unwrap_or(false)
    }

    pub fn online_user_ids(&self) -> Vec<Uuid> {
        let map = self.inner.lock().expect("presence mutex");
        map.keys().cloned().collect()
    }
}

pub struct PresenceGuard {
    store: PresenceStore,
    pub user_id: Uuid,
}

impl PresenceGuard {
    pub fn new(store: PresenceStore, user_id: Uuid) -> Self {
        store.mark_online(user_id);
        Self { store, user_id }
    }
}

impl Drop for PresenceGuard {
    fn drop(&mut self) {
        self.store.mark_offline(self.user_id);
    }
}
