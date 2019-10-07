//! encapsulates lib3h ghostmessage for sim2h including security challenge
use lib3h_protocol::data_types::Opaque;
use lib3h_protocol::protocol::*;
use std::convert::TryFrom;

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
        serde_json::to_string(&message)
            .expect("wiremessage should serialize")
            .into()
    }
}

impl TryFrom<Opaque> for WireMessage {
    type Error = Sim2hWireError;
    fn try_from(message: Opaque) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&String::from_utf8_lossy(&message))
            .map_err(|e| format!("{:?}", e))?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    //    use std::convert::Try;

    #[test]
    pub fn test_wire_message() {
        let msg = WireMessage::Err("fake_error".into());

        let opaque_msg: Opaque = msg.clone().into();
        assert_eq!(
            "\"{\\\"Err\\\":\\\"fake_error\\\"}\"",
            format!("{}", opaque_msg)
        );
        let roundtrip_msg = WireMessage::try_from(opaque_msg).expect("deserialize should work");
        assert_eq!(roundtrip_msg, msg);
    }
}
