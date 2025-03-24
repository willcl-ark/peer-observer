#![cfg_attr(feature = "strict", deny(warnings))]

use bitflags::bitflags;

// Use bitflags for slightly more efficient filtering
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Direction: u8 {
        const NONE     = 0b00000000;
        const INBOUND  = 0b00000001;
        const OUTBOUND = 0b00000010;
        const BOTH     = Self::INBOUND.bits() | Self::OUTBOUND.bits();
    }
}

impl Direction {
    // User-friendly name
    pub fn name(&self) -> &'static str {
        if self.contains(Direction::INBOUND) && self.contains(Direction::OUTBOUND) {
            "Both"
        } else if self.contains(Direction::INBOUND) {
            "Inbound"
        } else if self.contains(Direction::OUTBOUND) {
            "Outbound"
        } else {
            "None"
        }
    }

    pub fn arrow(&self) -> &'static str {
        if self.contains(Direction::INBOUND) {
            "<--"
        } else if self.contains(Direction::OUTBOUND) {
            "-->"
        } else {
            "---"
        }
    }

    pub fn text(&self) -> &'static str {
        if self.contains(Direction::INBOUND) {
            "from"
        } else if self.contains(Direction::OUTBOUND) {
            "to"
        } else {
            "with"
        }
    }
}
