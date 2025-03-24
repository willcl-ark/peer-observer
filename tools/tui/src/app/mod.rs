#![cfg_attr(feature = "strict", deny(warnings))]

use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::cmp;

use crate::models::{AppMode, Direction, EventType, LogMessage, Statistics};

pub struct App {
    autoscroll: bool,
    command_input: String,
    filter_conn_type: Option<String>,
    filter_direction: Direction,
    filter_event_type: EventType,
    filter_peer_id: Option<u64>,
    filter_text: Option<Vec<String>>,
    max_messages: usize,
    messages: AllocRingBuffer<LogMessage>,
    mode: AppMode,
    scroll_offset: u16,
    show_timestamps: bool,
    statistics: Statistics,
    total_messages_observed: usize,
    selected_message: Option<LogMessage>,
}

impl App {
    pub fn new() -> Self {
        let default_buffer_size = 10000;

        Self {
            autoscroll: true,
            command_input: String::new(),
            filter_conn_type: None,
            filter_direction: Direction::NONE,
            filter_event_type: EventType::NONE,
            filter_peer_id: None,
            filter_text: None,
            max_messages: default_buffer_size,
            messages: AllocRingBuffer::<LogMessage>::new(default_buffer_size),
            mode: AppMode::Normal,
            scroll_offset: 0,
            show_timestamps: false,
            statistics: Statistics::new(),
            total_messages_observed: 0,
            selected_message: None,
        }
    }

    // Getters
    pub fn messages(&self) -> &AllocRingBuffer<LogMessage> {
        &self.messages
    }

    pub fn push_message(&mut self, message: LogMessage) {
        self.messages.push(message);
    }

    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    pub fn command_input(&self) -> &str {
        &self.command_input
    }

    pub fn autoscroll(&self) -> bool {
        self.autoscroll
    }
    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub fn max_messages(&self) -> usize {
        self.max_messages
    }

    pub fn total_messages_observed(&self) -> usize {
        self.total_messages_observed
    }

    pub fn event_type_filter(&self) -> EventType {
        self.filter_event_type
    }

    pub fn direction_filter(&self) -> Direction {
        self.filter_direction
    }

    pub fn peer_id_filter(&self) -> Option<u64> {
        self.filter_peer_id
    }

    pub fn conn_type_filter(&self) -> &Option<String> {
        &self.filter_conn_type
    }

    pub fn text_filter(&self) -> &Option<Vec<String>> {
        &self.filter_text
    }

    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    pub fn statistics(&self) -> &Statistics {
        &self.statistics
    }

    pub fn selected_message(&self) -> &Option<LogMessage> {
        &self.selected_message
    }

    pub fn increment_total_messages(&mut self) {
        self.total_messages_observed += 1;
    }

    pub fn record_statistics(&mut self, message: &LogMessage) {
        self.statistics.record_event(message);
    }

    // Toggle statistics mode
    pub fn toggle_statistics_mode(&mut self) {
        if self.mode == AppMode::Statistics {
            self.mode = AppMode::Normal;
        } else {
            self.mode = AppMode::Statistics;
        }
    }

    pub fn open_detail_view(&mut self) {
        let filtered_messages = self.filtered_messages();
        if !filtered_messages.is_empty() {
            let selected_idx = self.scroll_offset as usize;
            if selected_idx < filtered_messages.len() {
                self.selected_message = Some(filtered_messages[selected_idx].clone());
                self.mode = AppMode::Detail;
            }
        }
    }

    pub fn close_detail_view(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps;
    }

    pub fn toggle_event_type_filter(&mut self, event_type: EventType) {
        if self.filter_event_type.contains(event_type) {
            self.filter_event_type.remove(event_type);
            // If no flags remain, set to NONE
            if self.filter_event_type.is_empty() {
                self.filter_event_type = EventType::NONE;
            }
        } else {
            // If NONE, replace instead of adding (this is first filter)
            if self.filter_event_type == EventType::NONE {
                self.filter_event_type = event_type;
            } else {
                self.filter_event_type.insert(event_type);
            }
        }
    }

    pub fn toggle_direction_filter(&mut self, direction: Direction) {
        if self.filter_direction.contains(direction) {
            self.filter_direction.remove(direction);
            // If no flags remain, set to NONE
            if self.filter_direction.is_empty() {
                self.filter_direction = Direction::NONE;
            }
        } else {
            // If NONE, replace instead of adding (this is first filter)
            if self.filter_direction == Direction::NONE {
                self.filter_direction = direction;
            } else {
                self.filter_direction.insert(direction);
            }
        }
    }

    pub fn set_direction_filter(&mut self, direction: Direction) {
        self.filter_direction = direction;
    }

    pub fn clear_all_filters(&mut self) {
        self.filter_conn_type = None;
        self.filter_direction = Direction::NONE;
        self.filter_event_type = EventType::NONE;
        self.filter_peer_id = None;
        self.filter_text = None;
    }

    // Set or clear the text search filter
    pub fn set_text_filter(&mut self, filter: Option<String>) {
        if let Some(search_text) = filter {
            // Split by commas or spaces
            let terms: Vec<String> = search_text
                .split([',', ' '])
                .filter(|s| !s.is_empty())
                .map(|s| s.to_lowercase())
                .collect();

            if terms.is_empty() {
                self.filter_text = None;
            } else {
                self.filter_text = Some(terms);
            }
        } else {
            self.filter_text = None;
        }

        // Reset to top after changing filter
        self.scroll_offset = 0;
    }

    // String repr of active filters
    pub fn get_filter_status(&self) -> String {
        let mut filter_parts = Vec::new();

        // Event type
        if self.filter_event_type != EventType::NONE {
            let mut types = Vec::new();
            if self.filter_event_type.contains(EventType::MESSAGE) {
                types.push("P2P");
            }
            if self.filter_event_type.contains(EventType::CONNECTION) {
                types.push("Conn");
            }
            if self.filter_event_type.contains(EventType::ADDRMAN) {
                types.push("Addr");
            }
            if self.filter_event_type.contains(EventType::MEMPOOL) {
                types.push("Mempool");
            }
            if self.filter_event_type.contains(EventType::VALIDATION) {
                types.push("Validation");
            }
            filter_parts.push(format!("Type: {}", types.join(",")));
        }

        // Direction
        if self.filter_direction != Direction::NONE {
            filter_parts.push(format!("Direction: {}", self.filter_direction.name()));
        }

        // Peer ID
        if let Some(peer_id) = self.filter_peer_id {
            filter_parts.push(format!("Peer: {}", peer_id));
        }

        // Connection type
        if let Some(conn_type) = &self.filter_conn_type {
            filter_parts.push(format!("Conn: {}", conn_type));
        }

        // Text search filter
        if let Some(terms) = &self.filter_text {
            if terms.len() == 1 {
                filter_parts.push(format!("Search: '{}'", terms[0]));
            } else {
                filter_parts.push(format!("Search: '{}'", terms.join("' OR '")));
            }
        }

        if filter_parts.is_empty() {
            "No filters active".to_string()
        } else {
            filter_parts.join(" | ")
        }
    }

    pub fn set_peer_id_filter(&mut self, peer_id: Option<u64>) {
        self.filter_peer_id = peer_id;
    }

    pub fn set_conn_type_filter(&mut self, conn_type: Option<String>) {
        self.filter_conn_type = conn_type;
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn push_command_input(&mut self, c: char) {
        self.command_input.push(c);
    }

    pub fn pop_command_input(&mut self) {
        self.command_input.pop();
    }

    pub fn clear_command_input(&mut self) {
        self.command_input.clear();
    }

    pub fn process_command(&mut self) {
        let cmd = self.command_input.trim().to_lowercase();

        // Reset the command input
        self.command_input.clear();

        // Clear all filters with :clear
        if cmd == "clear" {
            self.clear_all_filters();
            return;
        }

        // Set maximum message limit with :buffer NUMBER
        if cmd.starts_with("buffer ") {
            if let Some(limit_str) = cmd.strip_prefix("buffer ") {
                if let Ok(limit) = limit_str.trim().parse::<usize>() {
                    if limit > 0 {
                        // Don't allow less than this, it seems annoying
                        let new_size = cmp::max(limit, 500);

                        // Only resize if the size actually changed
                        if new_size != self.max_messages {
                            // Create a new buffer with the new capacity
                            let mut new_buffer = AllocRingBuffer::<LogMessage>::new(new_size);

                            // Transfer messages from old buffer to new one (keeping most recent)
                            let msg_count = self.messages.len().min(new_size);
                            let start_idx = self.messages.len().saturating_sub(msg_count);

                            for i in 0..msg_count {
                                if let Some(msg) = self.messages.get(start_idx + i) {
                                    new_buffer.push(msg.clone());
                                }
                            }

                            // Replace the old buffer with the new one
                            self.messages = new_buffer;
                            self.max_messages = new_size;
                        }
                    }
                }
            }
            return;
        }

        // Filter by peer ID with :peer ID
        if cmd.starts_with("peer ") {
            if let Some(peer_id_str) = cmd.strip_prefix("peer ") {
                if let Ok(peer_id) = peer_id_str.trim().parse::<u64>() {
                    self.set_peer_id_filter(Some(peer_id));
                } else if peer_id_str.trim() == "none" {
                    self.set_peer_id_filter(None);
                }
            }
            return;
        }

        // Filter by connection type with :conn TYPE
        if cmd.starts_with("conn ") {
            if let Some(conn_type) = cmd.strip_prefix("conn ") {
                if conn_type.trim() == "none" {
                    self.set_conn_type_filter(None);
                } else {
                    self.set_conn_type_filter(Some(conn_type.trim().to_string()));
                }
            }
        }
    }

    // Get messages that pass the current filter settings
    pub fn filtered_messages(&self) -> Vec<&LogMessage> {
        self.messages
            .iter()
            .filter(|msg| msg.passes_filters(self))
            .collect()
    }

    pub fn toggle_autoscroll(&mut self) {
        self.autoscroll = !self.autoscroll;
        if self.autoscroll {
            // When enabling autoscroll, immediately go to bottom
            self.scroll_to_bottom();
        } else {
            // When disabling autoscroll, keep the current position
            // Explicitly set scroll_offset to the current position (helps with UI rendering)
            let filtered_count = self.filtered_messages().len();
            if filtered_count > 0 {
                self.scroll_offset = (filtered_count - 1) as u16;
            }
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let filtered_count = self.filtered_messages().len();
        if filtered_count > 0 {
            self.scroll_offset = filtered_count as u16 - 1;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn manual_scroll(&mut self, delta: i32) {
        if delta != 0 {
            // Any manual scrolling disables autoscroll
            self.autoscroll = false;

            let filtered_count = self.filtered_messages().len() as i32;
            if filtered_count > 0 {
                // Calculate new position
                let new_pos = self.scroll_offset as i32 + delta;
                self.scroll_offset = new_pos.clamp(0, filtered_count - 1) as u16;
            } else {
                self.scroll_offset = 0;
            }
        }
    }
}
