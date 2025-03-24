#![cfg_attr(feature = "strict", deny(warnings))]

use jiff::Timestamp;

use crate::app::App;
use crate::models::{Direction, EventType};

#[derive(Clone)]
pub struct LogMessage {
    // Common
    pub content: String,
    pub event_type: EventType,
    pub timestamp: Timestamp,

    // P2P messages-only
    pub direction: Direction,
    pub peer_id: Option<u64>,
    pub conn_type: Option<String>,
    // Message type from the metadata (e.g., "Inv", "Version")
    pub p2p_msg_type: Option<String>,
    // size (when available)
    pub msg_size: Option<u64>,
}

impl LogMessage {
    // Special-case p2p messages as we format them a bit differently
    pub fn new_p2p(
        content: String,
        direction: Direction,
        peer_id: u64,
        conn_type: String,
        msg_type: Option<String>,
        msg_size: Option<u64>,
    ) -> Self {
        Self {
            content,
            event_type: EventType::MESSAGE,
            timestamp: Timestamp::now(),
            direction,
            peer_id: Some(peer_id),
            conn_type: Some(conn_type),
            p2p_msg_type: msg_type,
            msg_size,
        }
    }

    pub fn new_event(content: String, event_type: EventType) -> Self {
        Self {
            content,
            event_type,
            timestamp: Timestamp::now(),
            direction: Direction::NONE,
            peer_id: None,
            conn_type: None,
            p2p_msg_type: None,
            msg_size: None,
        }
    }

    pub fn format(&self, show_timestamp: bool) -> String {
        match (show_timestamp, self.event_type) {
            (true, EventType::MESSAGE) => {
                format!(
                    "[{}] {}{} {} id={} (conn_type={}): {}",
                    self.timestamp.strftime("%T.%3f"),
                    self.event_type.prefix(),
                    self.direction.arrow(),
                    self.direction.text(),
                    self.peer_id.unwrap_or(0),
                    self.conn_type.as_deref().unwrap_or("unknown"),
                    self.content
                )
            }
            (false, EventType::MESSAGE) => {
                format!(
                    "{}{} {} id={} (conn_type={}): {}",
                    self.event_type.prefix(),
                    self.direction.arrow(),
                    self.direction.text(),
                    self.peer_id.unwrap_or(0),
                    self.conn_type.as_deref().unwrap_or("unknown"),
                    self.content
                )
            }
            (true, _) => {
                format!(
                    "[{}] {} {}",
                    self.timestamp.strftime("%T.%3f"),
                    self.event_type.prefix(),
                    self.content
                )
            }
            (false, _) => {
                format!("{} {}", self.event_type.prefix(), self.content)
            }
        }
    }

    pub fn passes_p2p_filters(&self, app: &App) -> bool {
        // The following filters only apply to p2p messages
        if app.direction_filter() != Direction::NONE
            && !app.direction_filter().intersects(self.direction)
        {
            return false;
        }

        if let Some(peer_id_filter) = app.peer_id_filter() {
            if self.peer_id != Some(peer_id_filter) {
                return false;
            }
        }

        if let Some(conn_type_filter) = app.conn_type_filter() {
            if self.conn_type.as_deref() != Some(conn_type_filter) {
                return false;
            }
        }
        true
    }

    pub fn passes_filters(&self, app: &App) -> bool {
        // Event type
        if app.event_type_filter() != EventType::NONE
            && !app.event_type_filter().contains(self.event_type)
        {
            return false;
        }

        // Manual text search
        if let Some(search_terms) = app.text_filter() {
            let content_lower = self.content.to_lowercase();
            // Match against any term
            if !search_terms.iter().any(|term| content_lower.contains(term)) {
                return false;
            }
        }

        // P2P filters
        if self.event_type.contains(EventType::MESSAGE) {
            return self.passes_p2p_filters(app);
        };

        true
    }
}
