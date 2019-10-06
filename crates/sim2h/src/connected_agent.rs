//! represents the state of connected agents
use lib3h_protocol::{types::SpaceHash, Address};

pub type AgentId = Address;

#[derive(PartialEq, Debug, Clone)]
pub enum ConnectedAgent {
    Limbo,
    RequestedJoiningSpace(AgentId, SpaceHash),
    JoinedSpace(AgentId, SpaceHash),
}

#[allow(dead_code)]
impl ConnectedAgent {
    pub fn new() -> ConnectedAgent {
        ConnectedAgent::Limbo
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_connected_agent() {
        let ca = ConnectedAgent::new();
        assert_eq!(ca, ConnectedAgent::Limbo);
    }
}
