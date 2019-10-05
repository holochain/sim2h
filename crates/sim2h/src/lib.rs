//#[macro_use]
extern crate log;
extern crate env_logger;

pub mod connected_agent;
pub mod wire_message;

use std::{
    collections::HashMap,
    sync::RwLock,
};
use url::Url;

use connected_agent::*;
//use wire_message::*;


struct Sim2h {
    connection_states: RwLock<HashMap<Url, ConnectedAgent>>,
    spaces: HashMap<SpaceAddress, RwLock<HashMap<AgentId, Url>>>,
//    transport: GhostActorWebsocket<..>,
}

//self.transport.send(RequestToChild::SendMessage{ uri: other_url, payload });

impl Sim2h {
    /*
    fn join(&self, space, agent)
    fn leave(&self, space, agent)
    fn lookup_joined(&self, space, agent) -> Option<Url> {
         // return Some only if in same space and joined
    }

    fn process_next_message(&self) {
        match transport.drain() {
            RequestToParent::ReceivedData{uri, payload} => {
                let agent = self.connection_states.get(uri)?;
                    match agent.state {
                        ConnectionState::Limbo => self.process_limbo(agent),
                        ConnectionState::RequestedJoiningSpace => self.process_join_request(agent),
                        ConnectionState::JoinedSpace(agent_id, space_address) => self.proxy(space_address, agent_id, payload),
                    }
            }
            RequestToParent::IncomingConnection{uri} => {
                if self.connection_states.contains(uri) {
                    self.connection_states.write().remove(uri);
                    // We got another incoming connelet maybe_uri = ction from the same
                    // remote URI?! This is strange!
                    // We should tell the transport actor to drop
                    // the connection.
                } else {
                    self.connection_states.write().insert(uri, ConnectedAgent::new(uri))
                }
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
    //use super::*;

    #[test]
    pub fn test_sim2h() {
        assert!(false);
    }
}
