use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DeviceSession {
    pub device_name: String,
    pub user_agent: String,
    pub last_seen_at: Instant,
}

pub struct SessionManager {
    sessions: HashMap<String, DeviceSession>,
    timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout: Duration) -> Self {
        Self {
            sessions: HashMap::new(),
            timeout,
        }
    }

    pub fn upsert(&mut self, key: String, device_name: String, user_agent: String) {
        let now = Instant::now();

        self.sessions
            .entry(key.clone())
            .and_modify(|session| {
                session.device_name = device_name.clone();
                session.user_agent = user_agent.clone();
                session.last_seen_at = now;
            })
            .or_insert(DeviceSession {
                device_name,
                user_agent,
                last_seen_at: now,
            });
    }

    pub fn touch(&mut self, key: &str) {
        if let Some(session) = self.sessions.get_mut(key) {
            session.last_seen_at = Instant::now();
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.sessions.remove(key).is_some()
    }

    pub fn cleanup_expired(&mut self) {
        let timeout = self.timeout;
        self.sessions
            .retain(|_, session| session.last_seen_at.elapsed() < timeout);
    }

    pub fn len(&mut self) -> usize {
        self.cleanup_expired();
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::SessionManager;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn inserts_and_counts_sessions() {
        let mut manager = SessionManager::new(Duration::from_secs(60));
        manager.upsert("device-1".into(), "Phone".into(), "UA".into());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn removes_expired_sessions() {
        let mut manager = SessionManager::new(Duration::from_millis(10));
        manager.upsert("device-1".into(), "Phone".into(), "UA".into());
        sleep(Duration::from_millis(20));
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn removes_session_explicitly() {
        let mut manager = SessionManager::new(Duration::from_secs(60));
        manager.upsert("device-1".into(), "Phone".into(), "UA".into());
        assert!(manager.remove("device-1"));
        assert_eq!(manager.len(), 0);
    }
}
