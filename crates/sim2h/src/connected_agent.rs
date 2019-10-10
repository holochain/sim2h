//! represents the state of connected agents
use crate::wire_message::WireMessage;
use lib3h_protocol::{data_types::SpaceData, types::SpaceHash, Address};
pub type AgentId = Address;

pub type PendingMessages = Box<Vec<WireMessage>>;
pub type Entropy = String;

#[derive(PartialEq, Debug, Clone)]
pub enum ConnectedAgent {
    Limbo(PendingMessages),
    Handshaking(SpaceData, Entropy, PendingMessages),
    JoinedSpace(SpaceHash, AgentId),
}

impl ConnectedAgent {
    pub fn new() -> ConnectedAgent {
        ConnectedAgent::Limbo(Box::new(Vec::new()))
    }
    pub fn in_limbo(&self) -> bool {
        match self {
            ConnectedAgent::Limbo(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_connected_agent() {
        let ca = ConnectedAgent::new();
        assert_eq!(ca, ConnectedAgent::Limbo(Box::new(Vec::new())));
    }
}
