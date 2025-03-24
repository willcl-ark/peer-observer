#![cfg_attr(feature = "strict", deny(warnings))]

mod direction;
mod event_type;
mod log_message;
mod statistics;

pub use direction::Direction;
pub use event_type::EventType;
pub use log_message::LogMessage;
pub use statistics::Statistics;

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    Command,
    Search,
    Statistics,
    Detail,
}
