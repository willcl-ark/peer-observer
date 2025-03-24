#![cfg_attr(feature = "strict", deny(warnings))]

use bitflags::bitflags;

// Use bitflags for slightly more efficient filtering
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct EventType: u8 {
        const NONE      = 0b00000000;
        const MESSAGE   = 0b00000001;
        const CONNECTION = 0b00000010;
        const ADDRMAN   = 0b00000100;
        const MEMPOOL   = 0b00001000;
        const VALIDATION = 0b00010000;
    }
}

impl EventType {
    pub fn prefix(&self) -> &'static str {
        match *self {
            Self::MESSAGE => "[p2p]",
            Self::CONNECTION => "[connection]",
            Self::ADDRMAN => "[addrman]",
            Self::MEMPOOL => "[mempool]",
            Self::VALIDATION => "[validation]",
            _ => "[unknown]",
        }
    }
}
