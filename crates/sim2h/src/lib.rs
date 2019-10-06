//#[macro_use]
extern crate log;
extern crate env_logger;

pub mod connected_agent;
pub mod wire_message;

use std::{
    collections::HashMap,
    result,
};
use lib3h_protocol::{
  //  types::SpaceHash,
    uri::Lib3hUri,
};
use parking_lot::RwLock;

use connected_agent::*;
//use wire_message::*;

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
        Sim2h{
            connection_states: RwLock::new(HashMap::new()),
        }
    }

    // handler for incoming connections
    fn handle_incoming_connect(&self, uri: Lib3hUri) -> Sim2hResult<bool> {
        if let Some(_old) = self.connection_states.write().insert(uri.clone(), ConnectedAgent::new()) {
            println!("should remove {}",uri); //TODO
        };
        Ok(true)
    }

    // handler for messages sent to sim2h
    fn handle_message(&self, uri: &Lib3hUri, _payload: String) -> Sim2hResult<()> {
        match self.connection_states.read().get(uri) {
            None => return Err(format!("no connection for {}",uri)),
            _ => unimplemented!()/*
            ConnectionState::Limbo => self.process_limbo(agent, payload),
            ConnectionState::RequestedJoiningSpace => self.process_join_request(agent),
            ConnectionState::JoinedSpace(agent_id, space_address) => self.proxy(space_address, agent_id, payload),
        }*/
        };
        //Ok(())
    }

    /*
    fn join(&self, space, agent)
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

                // -- Space -- //
                /// Order the engine to be part of the network of the specified space.
                JoinSpace(SpaceData) => {panic!("should have been processed in process_limbo")} //handled in process_limbo
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

                // -- Direct Messaging -- //
                /// Send a message directly to another agent on the network
                SendDirectMessage(dm_data) => {
                    if dm_data.from_agent_id != agent_id {
                        return Err("don't do that")
                    }
                    let other_url = self.lookup_joined(space_address, dm_data.to_agent_id)?;

                    let payload = WireMessage::Lib3hToClient::HandleSendDirectMessage(dm_data).into();

                    self.transport.send(RequestToChild::SendMessage{ uri: other_url, payload });
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

    #[test]
    pub fn test_constructor() {
        let sim2h = Sim2h::new();
        let reader = sim2h.connection_states.read();
        assert_eq!(reader.len(),0)
    }

    #[test]
    pub fn test_incomming_connection() {
        let sim2h = Sim2h::new();

        // incoming connections get added to the map in limbo
        let uri = Lib3hUri::with_memory("addr_1");
        let result = sim2h.handle_incoming_connect(uri.clone());
        assert_eq!(result,Ok(true));

        { // scope for the reader to be dropped
            let reader = sim2h.connection_states.read();
            let result = reader.get(&uri).clone();
            assert_eq!(
                "Some(Limbo)",
                format!("{:?}", result)
            );
        }

        // pretend the agent has joined the space
        let _ = sim2h.connection_states.write().insert(uri.clone(), ConnectedAgent::JoinedSpace("fake_agent".into(),"fake_space".into()));
        // if we get a second incoming connection, the state should be reset.
        let result = sim2h.handle_incoming_connect(uri.clone());
        assert_eq!(result,Ok(true));
        let reader = sim2h.connection_states.read();
        let result = reader.get(&uri).clone();
        assert_eq!(
            "Some(Limbo)",
            format!("{:?}", result)
        );
    }

    #[test]
    pub fn test_message() {
        let sim2h = Sim2h::new();
        let uri = Lib3hUri::with_memory("addr_1");

        let payload = "message".to_string();
        // a message from an unconnected agent should return an error
        let result = sim2h.handle_message(&uri, payload);
        assert_eq!(result, Err(format!("no connection for {}",&uri)));

//        let result = sim2h.handle_incoming_connect(uri.clone());
//        assert_eq!(result,Ok(true));

    }
}
