use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct CallParticipant {
    pub username: String,
    pub server_name: String,
    pub user_id: String,
}

#[derive(Clone, Default)]
pub struct ChannelCallStore {
    inner: Arc<Mutex<HashMap<Uuid, HashSet<CallParticipant>>>>,
}

impl ChannelCallStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add participant to channel call, returns all participants in the call
    pub fn join(&self, channel_id: Uuid, participant: CallParticipant) -> Vec<CallParticipant> {
        let mut map = self.inner.lock().expect("channel_call mutex");
        let participants = map.entry(channel_id).or_default();
        participants.insert(participant);
        participants.iter().cloned().collect()
    }

    /// Remove participant from channel call, returns true if they were in the call
    pub fn leave(&self, channel_id: Uuid, participant: &CallParticipant) -> bool {
        let mut map = self.inner.lock().expect("channel_call mutex");
        if let Some(participants) = map.get_mut(&channel_id) {
            let removed = participants.remove(participant);
            if participants.is_empty() {
                map.remove(&channel_id);
            }
            removed
        } else {
            false
        }
    }

    /// Remove user from ALL calls (disconnect cleanup), returns list of channel_ids they were in
    pub fn leave_all(&self, username: &str, server_name: &str) -> Vec<Uuid> {
        let mut map = self.inner.lock().expect("channel_call mutex");
        let mut affected = Vec::new();
        let mut empty_channels = Vec::new();

        for (channel_id, participants) in map.iter_mut() {
            let before = participants.len();
            participants.retain(|p| !(p.username == username && p.server_name == server_name));
            if participants.len() < before {
                affected.push(*channel_id);
            }
            if participants.is_empty() {
                empty_channels.push(*channel_id);
            }
        }

        for ch in empty_channels {
            map.remove(&ch);
        }

        affected
    }

    /// Get participants in a specific channel call
    pub fn participants(&self, channel_id: Uuid) -> Vec<CallParticipant> {
        let map = self.inner.lock().expect("channel_call mutex");
        map.get(&channel_id)
            .map(|p| p.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all active calls with participant counts (for sidebar badges)
    pub fn all_active_calls(&self) -> HashMap<String, usize> {
        let map = self.inner.lock().expect("channel_call mutex");
        map.iter()
            .map(|(id, participants)| (id.to_string(), participants.len()))
            .collect()
    }
}
