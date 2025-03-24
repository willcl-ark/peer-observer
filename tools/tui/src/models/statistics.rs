#![cfg_attr(feature = "strict", deny(warnings))]

use std::collections::HashMap;

use crate::models::{Direction, EventType, LogMessage};

pub struct Statistics {
    pub event_type_counts: HashMap<EventType, usize>,
    pub p2p_message_counts: HashMap<String, usize>, // Mapped by command name
    pub direction_counts: HashMap<Direction, usize>,
    pub p2p_message_bytes: HashMap<String, u64>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            event_type_counts: HashMap::new(),
            p2p_message_counts: HashMap::new(),
            direction_counts: HashMap::new(),
            p2p_message_bytes: HashMap::new(),
        }
    }

    pub fn record_event(&mut self, message: &LogMessage) {
        // Update event type counts
        *self
            .event_type_counts
            .entry(message.event_type)
            .or_insert(0) += 1;

        // Update direction counts for P2P messages
        if message.event_type.contains(EventType::MESSAGE) {
            *self.direction_counts.entry(message.direction).or_insert(0) += 1;

            // Get message type for statistics
            let msg_type = if let Some(msg_type) = &message.p2p_msg_type {
                msg_type.clone()
            } else {
                // Fallback in case p2p_msg_type is None:
                // Extract from content string as before
                let content = &message.content;
                if let Some(cmd_end) = content.find(['(', '[']) {
                    content[..cmd_end].to_string()
                } else {
                    // Fallback for simple messages without parameters
                    content.to_string()
                }
            };

            // Update message count
            *self.p2p_message_counts.entry(msg_type.clone()).or_insert(0) += 1;

            // Update message byte size if available
            if let Some(size) = message.msg_size {
                *self.p2p_message_bytes.entry(msg_type).or_insert(0) += size;
            }
        }
    }
}
