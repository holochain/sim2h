//! represents the state of connected agents
use lib3h_protocol::Address;
pub type AgentId = Address;
pub type SpaceAddress = Address;

#[derive(PartialEq,Debug)]
pub enum ConnectedAgent {
    Limbo,
    RequestedJoiningSpace(AgentId, Address),
    JoinedSpace(AgentId, Address),
}

impl ConnectedAgent {
    fn new() -> ConnectedAgent {
        ConnectedAgent::Limbo
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_connected_agent() {
        let ca = ConnectedAgent::new();
        assert_eq!(ca,ConnectedAgent::Limbo);
    }
}
