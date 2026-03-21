pub struct ReconnectPolicy {
    pub enabled: bool,
    pub max_attempts: u32,
    pub backoff_max_secs: u32,
}

impl ReconnectPolicy {
    pub fn backoff_secs(&self, attempt: u32) -> u32 {
        2u32.pow(attempt.saturating_sub(1)).min(self.backoff_max_secs)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        self.enabled && attempt < self.max_attempts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Connected,
    Disconnected,
    Connecting,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user: String,
    pub host: String,
    pub hostname: Option<String>,
    pub port: u16,
    pub state: SessionState,
    pub label: Option<String>,
}

pub struct SessionManager {
    sessions: Vec<(u32, Session)>,
    active_id: Option<u32>,
    next_id: u32,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Vec::new(),
            active_id: None,
            next_id: 1,
        }
    }

    pub fn create(&mut self, session: Session) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        if self.active_id.is_none() {
            self.active_id = Some(id);
        }
        self.sessions.push((id, session));
        id
    }

    pub fn set_active(&mut self, id: u32) {
        self.active_id = Some(id);
    }

    pub fn active_id(&self) -> Option<u32> {
        self.active_id
    }

    pub fn switch_to(&mut self, id: u32) -> bool {
        if self.sessions.iter().any(|(sid, _)| *sid == id) {
            self.active_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn kill(&mut self, id: u32) -> bool {
        let before = self.sessions.len();
        self.sessions.retain(|(sid, _)| *sid != id);
        let killed = self.sessions.len() < before;
        if killed {
            if self.active_id == Some(id) {
                self.active_id = self.sessions.first().map(|(sid, _)| *sid);
            }
        }
        killed
    }

    pub fn rename(&mut self, id: u32, label: String) -> bool {
        if let Some((_, session)) = self.sessions.iter_mut().find(|(sid, _)| *sid == id) {
            session.label = Some(label);
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: u32) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|(sid, _)| *sid == id)
            .map(|(_, s)| s)
    }

    pub fn list(&self) -> &[(u32, Session)] {
        &self.sessions
    }

    pub fn format_list(&self) -> String {
        let mut lines = Vec::new();
        for (id, session) in &self.sessions {
            let display_host = session
                .hostname
                .as_deref()
                .unwrap_or(session.host.as_str());
            let mut line = format!("#{} {}@{}", id, session.user, display_host);
            if session.hostname.is_some() {
                line.push_str(&format!(" ({})", session.host));
            }
            if let Some(lbl) = &session.label {
                line.push_str(&format!(" [{}]", lbl));
            }
            if self.active_id == Some(*id) {
                line.push_str(" \u{25c4} active");
            }
            lines.push(line);
        }
        lines.join("\n")
    }
}
