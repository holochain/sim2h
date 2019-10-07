//! encapsulates lib3h ghostmessage for sim2h including security challenge
use lib3h_protocol::protocol::*;
use lib3h_protocol::data_types::Opaque;

pub type Sim2hWireError = String;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WireMessage {
    ClientToLib3h(ClientToLib3h),
    ClientToLib3hResponse(ClientToLib3hResponse),
    Lib3hToClient(Lib3hToClient),
    Lib3hToClientResponse(Lib3hToClientResponse),
    Err(Sim2hWireError),
    SignatureChallenge(String),
    SignatureChallengeResponse(String),
}

impl From<WireMessage> for Opaque {
    fn from(message: WireMessage) -> Opaque {
        serde_json::to_string(&message).expect("wiremessage should serialize").into()
    }
}

#[cfg(test)]
pub mod tests {
    //use super::*;

    #[test]
    pub fn test_wire_message() {
        assert!(true);
    }
}
