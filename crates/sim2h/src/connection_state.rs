//! represents the state of connected agents
use crate::wire_message::{Entropy, WireMessage};
use lib3h_protocol::{data_types::SpaceData, types::SpaceHash, Address};
pub type AgentId = Address;

pub type PendingMessages = Box<Vec<WireMessage>>;

#[derive(PartialEq, Debug, Clone)]
pub enum ConnectionState {
    Limbo(PendingMessages),
    Handshaking(SpaceData, Entropy, PendingMessages),
    Joined(SpaceHash, AgentId),
}

impl ConnectionState {
    pub fn new() -> ConnectionState {
        ConnectionState::Limbo(Box::new(Vec::new()))
    }
    pub fn in_limbo(&self) -> bool {
        match self {
            ConnectionState::Limbo(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_connection_state() {
        let ca = ConnectionState::new();
        assert_eq!(ca, ConnectionState::Limbo(Box::new(Vec::new())));
    }
}
