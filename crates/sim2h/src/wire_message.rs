//! encapsulates lib3h ghostmessage for sim2h including security challenge
use lib3h_protocol::protocol::*;

pub type Sim2hWireError = String;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WireMessage {
    ClientToLib3h(ClientToLib3h),
    ClientToLib3hResponse(ClientToLib3hResponse),
    Lib3hToClient(Lib3hToClient),
    Lib3hToClientResponse(Lib3hToClientResponse),
    Err(Sim2hWireError),
    SignatureChallenge(String),
    SignatureChallengeResponse(String),
}

#[cfg(test)]
pub mod tests {
    //use super::*;

    #[test]
    pub fn test_wire_message() {
        assert!(true);
    }
}
