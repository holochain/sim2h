//! encapsulates lib3h gostmessage for sim2h including security challenge
use lib3h_protocol::protocol::*;

pub type Sim2hWireError = String;

enum WireMessage {
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
        assert!(false);
    }
}
