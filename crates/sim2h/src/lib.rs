//#[macro_use]
extern crate env_logger;
extern crate log;

pub mod connected_agent;
pub mod wire_message;

use lib3h_protocol::{
    data_types::SpaceData, protocol::ClientToLib3h, types::SpaceHash, uri::Lib3hUri,
};
use parking_lot::RwLock;
use std::{collections::HashMap, result};

use connected_agent::*;
use wire_message::WireMessage;

#[allow(dead_code)]
struct Sim2h {
    connection_states: RwLock<HashMap<Lib3hUri, ConnectedAgent>>,
    //    spaces: HashMap<SpaceAddress, RwLock<HashMap<AgentId, Url>>>,
    //    transport: GhostActorWebsocket<..>,
}

pub type Sim2hError = String;
pub type Sim2hResult<T> = result::Result<T, Sim2hError>;

#[allow(dead_code)]
impl Sim2h {
    pub fn new() -> Self {
        Sim2h {
            connection_states: RwLock::new(HashMap::new()),
        }
    }

    // adds an agent to a space
    fn join(&self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        if let Some(ConnectedAgent::Limbo) = self.get_connection(uri) {
            let _ = self.connection_states.write().insert(
                uri.clone(),
                ConnectedAgent::JoinedSpace(data.space_address.clone(), data.agent_id.clone()),
            );
            Ok(())
        } else {
            Err(format!("no agent found in limbo at {} ", uri))
        }
    }

    // get the connection status of an agent
    fn get_connection(&self, uri: &Lib3hUri) -> Option<ConnectedAgent> {
        let reader = self.connection_states.read();
        reader.get(uri).map(|ca| (*ca).clone())
    }

    // find out if an agent is in a space or not and return its URI
    // TODO get from a cache instead of iterating
    fn lookup_joined(&self, space_address: &SpaceHash, agent_id: &AgentId) -> Option<Lib3hUri> {
        for (key, val) in self.connection_states.read().iter() {
            if let ConnectedAgent::JoinedSpace(item_space, item_agent) = val {
                if item_space == space_address && item_agent == agent_id {
                    return Some(key.clone());
                }
            }
        }
        None
    }

    // handler for incoming connections
    fn handle_incoming_connect(&self, uri: Lib3hUri) -> Sim2hResult<bool> {
        if let Some(_old) = self
            .connection_states
            .write()
            .insert(uri.clone(), ConnectedAgent::new())
        {
            println!("should remove {}", uri); //TODO
        };
        Ok(true)
    }

    // handler for messages sent to sim2h
    fn handle_message(&self, uri: &Lib3hUri, message: WireMessage) -> Sim2hResult<()> {
        let agent = self
            .get_connection(uri)
            .ok_or_else(|| format!("no connection for {}", uri))?;
        match agent {
            ConnectedAgent::Limbo => {
                if let WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(data)) = message {
                    self.join(uri, &data)
                } else {
                    Err(format!("no agent validated at {} ", uri))
                }
            }
            //ConnectionState::RequestedJoiningSpace => self.process_join_request(agent),
            ConnectedAgent::JoinedSpace(space_address, agent_id) => {
                self.proxy(&space_address, &agent_id, message)
            }
        }
    }

    fn proxy(
        &self,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        message: WireMessage,
    ) -> Sim2hResult<()> {
        match message {
            // -- Space -- //
            WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(_)) => {
                return Err(
                    "join message should have been processed elsewhere and can't be proxied".into(),
                );
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            WireMessage::ClientToLib3h(ClientToLib3h::SendDirectMessage(dm_data)) => {
                if (dm_data.from_agent_id != *agent_id) || (dm_data.space_address != *space_address)
                {
                    return Err("space/agent id mismatch".into());
                }
                let _other_url = self
                    .lookup_joined(space_address, &dm_data.to_agent_id)
                    .ok_or_else(|| format!("unvalidated proxy agent {}", &dm_data.to_agent_id))?;
                /*
                                let payload = WireMessage::Lib3hToClient::HandleSendDirectMessage(dm_data).into();

                                self.transport.send(RequestToChild::SendMessage{ uri: other_url, payload });
                */
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    /*
        fn leave(&self, space, agent)
        fn lookup_joined(&self, space, agent) -> Option<Url> {
             // return Some only if in same space and joined
        }

        // the message better be a join
        fn process_limbo(agent,payload) {}

        // cache messages cus we are waiting for confirmation of join
        fn self.process_join_request(agent,payload) {}

        fn process_next_message(&self) {
            match transport.drain() {
                RequestToParent::ReceivedData{uri, payload} => {
                    self.handle_message(uri,payload)?
                }
                RequestToParent::IncomingConnection{uri} => {
                    self.handle_incominng_connection(uri)?
                }

                RequestToParent::ConnectionClosed{uri} => {
                    self.connection_states.writ } else if let Ok(msg) = Lib3hToClientResponse::try_from(payload) e().remove(uri);  // ignore if we don't have it
                }
            }

        }

        fn proxy(&self, space_address: Address, agent_id: AgentId, payload: Opaque) -> Result<Option<Opaque>, ProxyError> {

            match WireMessage::try_from(payload)? {
                ClientToLib3h(msg) => match msg {
                    // -- Connection -- //
                    /// create an explicit connection to a remote peer
                    Bootstrap(BootstrapData) => {// handled in client}

                    /// Order the engine to leave the network of the specified space.
                    LeaveSpace(SpaceData) => {
                        // remove from map
                        self.spaces
                            .get(space_address)?
                            .write()
                            .take(agent_id)
                            .and_then(|uri| {
                                self.connection_states.write().remove(uri);
                                self.transport.send(RequestToChild::Disconnect(uri));
                            })
                    }

                    // -- Entry -- //
                    /// Request an Entry from the dht network
                    FetchEntry(FetchEntryData), // NOTE: MAY BE DEPRECATED
                    /// Publish data to the dht (event)                HandleGetGossipingEntryListResult(EntryListData) => {}

                    PublishEntry(provided_entry_data) => {
                        for aspect_data in provided_entry_data.entry_data.aspect_list {
                            let broadcast_msg = WireMessage::Lib3hToClient::HandleStoreEntryAspect(
                                StoreEntryAspectData {
                                    request_id: String,
                                    space_address: provided_entry_data.space_address.clone(),
                                    provider_agent_id: provided_entry_data.provider_agent_id.clone(),
                                    entry_address: Address,
                                    entry_aspect: EntryAspectData,
                                });
                            broadcast(space_address, broadcast_msg)?;
                        }

                    }
                    /// Tell Engine that Client is holding this entry (event)
                    HoldEntry(ProvidedEntryData) => {}
                    /// Request some info / data from a Entry
                    QueryEntry(QueryEntryData) => {}
                },
              Lib3hToClientResponse(msg) => msg {
                    /// Our response to a direct message from another agent.
                    HandleSendDirectMessageResult(DirectMessageData) => {}
                    /// Successful data response for a `HandleFetchEntryData` request
                    HandleFetchEntryResult(FetchEntryResultData) => {}
                    HandleStoreEntryAspectResult => {}
                    HandleDropEntryResult => {}
                    /// Response to a `HandleQueryEntry` request
                    HandleQueryEntryResult(QueryEntryResultData) => {}
                    // -- Entry lists -- //
                    HandleGetAuthoringEntryListResult(EntryListData) => {}
                    HandleGetGossipingEntryListResult(EntryListData) => {}
                },
              _ => WireMessage::Err(...)
            }



            let {space, from,to} = msg;
            if from != agent_id_from_ws {
                return Err(...)
            }

            //if do we have a connected agent
            let to_agent = self.lookup(space, to).ok_or_else(|| Err(...))?;

            match msg {
                SendDirectMessage | SendDirectMessageResult => to_agent.forward(msg),
                Publish => self.broadcast(space_address, StoreEntryAspect(...))
            }

        }

        fn broadcast(&self, space: Address, msg: WireMessage) {
            let payload: Opaque = msg.into();
            for (uri) self.spaces.get(space)?.read().values() {
                self.transport.send(RequestToChild::SendMessage{ uri, payload: payload.clone() });
            }
        }
    */
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use lib3h_protocol::data_types::*;

    fn make_test_space_data() -> SpaceData {
        SpaceData {
            request_id: "".into(),
            space_address: "fake_space_address".into(),
            agent_id: "fake_agent_id".into(),
        }
    }

    fn make_test_join_message() -> WireMessage {
        WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(make_test_space_data()))
    }

    fn make_test_dm_data() -> DirectMessageData {
        DirectMessageData {
            request_id: "".into(),
            space_address: "fake_space_address".into(),
            from_agent_id: "fake_agent_id".into(),
            to_agent_id: "fake_to_agent_id".into(),
            content: "foo".into(),
        }
    }

    fn make_test_dm_message() -> WireMessage {
        WireMessage::ClientToLib3h(ClientToLib3h::SendDirectMessage(make_test_dm_data()))
    }

    fn make_test_err_message() -> WireMessage {
        WireMessage::Err("fake_error".into())
    }

    #[test]
    pub fn test_constructor() {
        let sim2h = Sim2h::new();
        let reader = sim2h.connection_states.read();
        assert_eq!(reader.len(), 0)
    }

    #[test]
    pub fn test_incomming_connection() {
        let sim2h = Sim2h::new();

        // incoming connections get added to the map in limbo
        let uri = Lib3hUri::with_memory("addr_1");
        let result = sim2h.handle_incoming_connect(uri.clone());
        assert_eq!(result, Ok(true));

        let result = sim2h.get_connection(&uri).clone();
        assert_eq!("Some(Limbo)", format!("{:?}", result));

        // pretend the agent has joined the space
        let _ = sim2h.connection_states.write().insert(
            uri.clone(),
            ConnectedAgent::JoinedSpace("fake_agent".into(), "fake_space".into()),
        );
        // if we get a second incoming connection, the state should be reset.
        let result = sim2h.handle_incoming_connect(uri.clone());
        assert_eq!(result, Ok(true));
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!("Some(Limbo)", format!("{:?}", result));
    }

    #[test]
    pub fn test_join() {
        let sim2h = Sim2h::new();
        let uri = Lib3hUri::with_memory("addr_1");

        let data = make_test_space_data();
        // you can't join if you aren't in limbo
        let result = sim2h.join(&uri, &data);
        assert_eq!(result, Err(format!("no agent found in limbo at {} ", &uri)));

        // but you can if you are  TODO: real membrane check
        let _result = sim2h.handle_incoming_connect(uri.clone());
        let result = sim2h.join(&uri, &data);
        assert_eq!(result, Ok(()));
        assert_eq!(
            sim2h.lookup_joined(&data.space_address, &data.agent_id),
            Some(uri.clone())
        );
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!(
            "Some(JoinedSpace(SpaceHash(HashString(\"fake_space_address\")), HashString(\"fake_agent_id\")))",
            format!("{:?}", result)
        );
    }

    #[test]
    pub fn test_message() {
        let sim2h = Sim2h::new();
        let uri = Lib3hUri::with_memory("addr_1");

        // a message from an unconnected agent should return an error
        let result = sim2h.handle_message(&uri, make_test_err_message());
        assert_eq!(result, Err(format!("no connection for {}", &uri)));

        // a non-join message from an unvalidated but connected agent should return an error
        let _result = sim2h.handle_incoming_connect(uri.clone());
        let result = sim2h.handle_message(&uri, make_test_err_message());
        assert_eq!(result, Err(format!("no agent validated at {} ", &uri)));

        // a valid join message from a connected agent should update its connection status
        let result = sim2h.handle_message(&uri, make_test_join_message());
        assert_eq!(result, Ok(()));
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!(
            "Some(JoinedSpace(SpaceHash(HashString(\"fake_space_address\")), HashString(\"fake_agent_id\")))",
            format!("{:?}", result)
        );
    }

    #[test]
    pub fn test_proxy() {
        let sim2h = Sim2h::new();
        let _uri = Lib3hUri::with_memory("addr_1");

        let message = make_test_join_message();
        let data = make_test_space_data();

        // you can't proxy a join message
        let result = sim2h.proxy(&data.space_address, &data.agent_id, message);
        assert!(result.is_err());

        // you can't proxy for someone else
        let message = make_test_dm_message();
        let result = sim2h.proxy(&data.space_address, &"fake_other_agent".into(), message);
        assert_eq!("Err(\"space/agent id mismatch\")", format!("{:?}", result));

        // you can't proxy for someone not in the space
        let message = make_test_dm_message();
        let result = sim2h.proxy(&data.space_address, &data.agent_id, message);
        assert_eq!(
            "Err(\"unvalidated proxy agent fake_to_agent_id\")",
            format!("{:?}", result)
        );
    }
}
