extern crate env_logger;
//#[macro_use]
extern crate log;
#[macro_use]
extern crate detach;
#[macro_use]
extern crate serde;

pub mod connected_agent;
pub mod error;
pub mod wire_message;

use crate::error::*;
use detach::prelude::*;
use holochain_tracing::Span;
use lib3h::transport::protocol::*;
use lib3h_protocol::{data_types::SpaceData, protocol::*, types::SpaceHash, uri::Lib3hUri};
use lib3h_zombie_actor::prelude::*;

use parking_lot::RwLock;
use std::{convert::TryFrom, collections::HashMap};

use connected_agent::*;
pub use wire_message::WireMessage;
use url::Url;
use log::{debug, error, warn};
use lib3h_protocol::data_types::StoreEntryAspectData;

#[allow(dead_code)]
pub struct Sim2h {
    pub bound_uri: Option<Lib3hUri>,
    connection_states: RwLock<HashMap<Lib3hUri, ConnectedAgent>>,
    spaces: HashMap<SpaceHash, RwLock<HashMap<AgentId, Lib3hUri>>>,
    transport: Detach<TransportActorParentWrapperDyn<Self>>,
}

#[allow(dead_code)]
impl Sim2h {
    pub fn new(transport: DynTransportActor) -> Self {
        let t = Detach::new(TransportActorParentWrapperDyn::new(transport, "transport_"));

        let mut sim2h = Sim2h {
            bound_uri: None,
            connection_states: RwLock::new(HashMap::new()),
            spaces: HashMap::new(),
            transport: t,
        };

        let _ = sim2h.transport.request(
            Span::fixme(),
            RequestToChild::Bind {
                spec: Lib3hUri::with_undefined(),
            }, //TODO get spec from a config
            Box::new(|me, response| match response {
                GhostCallbackData::Response(Ok(RequestToChildResponse::Bind(bind_result))) => {
                    me.bound_uri = Some(bind_result.bound_url);
                    Ok(())
                }
                GhostCallbackData::Response(Err(e)) => Err(e.into()),
                GhostCallbackData::Timeout(bt) => Err(format!("timeout: {:?}", bt).into()),
                _ => Err("bad response type".into()),
            }),
        );
        sim2h
    }

    pub fn with_detached_transport(transport: Detach<TransportActorParentWrapperDyn<Self>>) -> Self {
        Sim2h {
            bound_uri: None,
            connection_states: RwLock::new(HashMap::new()),
            spaces: HashMap::new(),
            transport,
        }
    }

    pub fn bind_transport_sync(&mut self, port: u16) -> Result<Lib3hUri, String> {
        let url_string = format!("wss://localhost:{}", port);
        let url = Url::parse(&url_string).expect("can parse url");
        debug!("Trying to bind to {}...", url_string);

        // channel for making an async call sync
        let (tx, rx) = crossbeam_channel::unbounded();
        self.transport.request(
            Span::todo("Find out how to use spans the right way"),
            RequestToChild::Bind {
                spec: url.into(),
            },
            // callback just notifies channel so
            Box::new(move |_owner, response| {
                let result = match response {
                    GhostCallbackData::Timeout(bt) => Err(format!("Bind timed out. Backtrace: {:?}", bt)),
                    GhostCallbackData::Response(Ok(RequestToChildResponse::Bind(bind_result_data))) =>
                        Ok(bind_result_data.bound_url),
                    GhostCallbackData::Response(Err(transport_error)) =>
                        Err(format!("Error during bind: {:?}", transport_error)),
                    _ => Err(String::from("Got unexpected response from transport actor during bind")),
                };
                let _ = tx.send(result);
                Ok(())
            }),
        ).map_err(|ghost_error| format!("GhostError during bind: {:?}", ghost_error))?;

        for _ in 1..3 {
            detach_run!(&mut self.transport, |t| t.process(self)).map_err(|ghost_error| format!("GhostError during bind: {:?}", ghost_error))?;
        }

        rx.recv().expect("local channel to work")
    }

    // adds an agent to a space
    fn join(&mut self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        if let Some(ConnectedAgent::Limbo) = self.get_connection(uri) {
            let _ = self.connection_states.write().insert(
                uri.clone(),
                ConnectedAgent::JoinedSpace(data.space_address.clone(), data.agent_id.clone()),
            );
            if !self.spaces.contains_key(&data.space_address) {
                self.spaces.insert(data.space_address.clone(), RwLock::new(HashMap::new()));
            }
            self.spaces.get(&data.space_address).unwrap().write().insert(data.agent_id.clone(), uri.clone());
            debug!("Agent {:?} joined space {:?}", data.agent_id, data.space_address);
            Ok(())
        } else {
            Err(format!("no agent found in limbo at {} ", uri).into())
        }
    }

    // removes an agent from a space
    fn leave(&self, uri: &Lib3hUri, data: &SpaceData) -> Sim2hResult<()> {
        if let Some(ConnectedAgent::JoinedSpace(space_address, agent_id)) = self.get_connection(uri)
        {
            if (data.agent_id != agent_id) || (data.space_address != space_address) {
                Err(SPACE_MISMATCH_ERR_STR.into())
            } else {
                self.connection_states.write().remove(uri).unwrap();
                Ok(())
            }
        } else {
            Err(format!("no joined agent found at {} ", &uri).into())
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
        debug!("New connection from {:?}", uri);
        if let Some(_old) = self
            .connection_states
            .write()
            .insert(uri.clone(), ConnectedAgent::new())
        {
            println!("TODO should remove {}", uri); //TODO
        };
        Ok(true)
    }

    // handler for messages sent to sim2h
    fn handle_message(&mut self, uri: &Lib3hUri, message: WireMessage) -> Sim2hResult<()> {
        let agent = self
            .get_connection(uri)
            .ok_or_else(|| format!("no connection for {}", uri))?;
        match agent {
            ConnectedAgent::Limbo => {
                if let WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(data)) = message {
                    self.join(uri, &data)
                } else {
                    Err(format!("no agent validated at {} ", uri).into())
                }
            }
            //ConnectionState::RequestedJoiningSpace => self.process_join_request(agent),
            ConnectedAgent::JoinedSpace(space_address, agent_id) => {
                if let Some((is_request, to_uri, message)) =
                    self.prepare_proxy(uri, &space_address, &agent_id, message)?
                {
                    if is_request {
                        let payload = message.into();
                        self.transport
                            .request(
                                Span::fixme(),
                                RequestToChild::SendMessage {
                                    uri: to_uri,
                                    payload,
                                },
                                Box::new(|_me, response| match response {
                                    GhostCallbackData::Response(Ok(
                                        RequestToChildResponse::SendMessageSuccess,
                                    )) => Ok(()),
                                    GhostCallbackData::Response(Err(e)) => Err(e.into()),
                                    GhostCallbackData::Timeout(bt) => {
                                        Err(format!("timeout: {:?}", bt).into())
                                    }
                                    _ => Err("bad response type".into()),
                                }),
                            )
                            .map_err(|e| e.into())
                    } else {
                        unimplemented!()
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn process(&mut self) -> Sim2hResult<()> {
        detach_run!(&mut self.transport, |t| t.process(self)).map_err(|e| format!("{:?}", e))?;
        for mut transport_message in self.transport.drain_messages() {
            match transport_message.take_message().expect("GhostMessage must have a message") {
                RequestToParent::ReceivedData {uri, payload} => {
                    match WireMessage::try_from(&payload) {
                        Ok(wire_message) =>
                            if let Err(error) = self.handle_message(&uri, wire_message) {
                                error!("Error handling message: {:?}", error);
                            },
                        Err(error) =>
                            error!(
                                "Could not deserialize received payload into WireMessage!\nError: {:?}\nPayload was: {:?}",
                                error,
                                payload
                            )
                    }
                }
                RequestToParent::IncomingConnection {uri} =>
                    if let Err(error) = self.handle_incoming_connect(uri) {
                        error!("Error handling incomming connection: {:?}", error);
                    },
                RequestToParent::ErrorOccured {uri, error} =>
                    error!("Transport error occured on connection to {:?}: {:?}", uri, error),
            }
        }
        Ok(())
    }

    // given an incoming messages, prepare a proxy message and whether it's an publish or request
    fn prepare_proxy(
        &mut self,
        uri: &Lib3hUri,
        space_address: &SpaceHash,
        agent_id: &AgentId,
        message: WireMessage,
    ) -> Sim2hResult<Option<(bool, Lib3hUri, WireMessage)>> {
        debug!("Handling message from agent {:?}: {:?}", agent_id, message);
        match message {
            // -- Space -- //
            WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(_)) => {
                Err("join message should have been processed elsewhere and can't be proxied".into())
            }
            WireMessage::ClientToLib3h(ClientToLib3h::LeaveSpace(data)) => {
                self.leave(uri, &data).map(|_| None)
            }

            // -- Direct Messaging -- //
            // Send a message directly to another agent on the network
            WireMessage::ClientToLib3h(ClientToLib3h::SendDirectMessage(dm_data)) => {
                if (dm_data.from_agent_id != *agent_id) || (dm_data.space_address != *space_address)
                {
                    return Err(SPACE_MISMATCH_ERR_STR.into());
                }
                let to_url = self
                    .lookup_joined(space_address, &dm_data.to_agent_id)
                    .ok_or_else(|| format!("unvalidated proxy agent {}", &dm_data.to_agent_id))?;
                Ok(Some((
                    true,
                    to_url,
                    WireMessage::Lib3hToClient(Lib3hToClient::HandleSendDirectMessage(dm_data)),
                )))
            }
            WireMessage::ClientToLib3h(ClientToLib3h::PublishEntry(data)) => {
                debug!("Got Publish - broadcasting all aspects to all agents...");
                for aspect in data.entry.aspect_list {
                    let store_message = WireMessage::Lib3hToClient(
                        Lib3hToClient::HandleStoreEntryAspect(StoreEntryAspectData {
                            request_id: "".into(),
                            space_address: data.space_address.clone(),
                            provider_agent_id: data.provider_agent_id.clone(),
                            entry_address: data.entry.entry_address.clone(),
                            entry_aspect: aspect,
                    }));
                    if let Err(e) = self.broadcast(data.space_address.clone(), &store_message) {
                        error!("Error during broadcast: {:?}", e);
                    }
                }
                Ok(None)
            }
            _ => {
                warn!("Ignoring unimplemented message");
                Err("Message not implemented".into())
            },
        }
    }

    /*
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
*/
    fn broadcast(&mut self, space: SpaceHash, msg: &WireMessage) -> Result<(), String>{
        debug!("Broadcast in space: {:?}", space);
        for uri in self.spaces.get(&space).ok_or("No such space")?.read().values().cloned() {
            debug!("Broadcast: Sending to {:?}", uri);
            let send_result = self.transport
                .request(
                    Span::fixme(),
                    RequestToChild::SendMessage { uri, payload: msg.clone().into() },
                    Box::new(|_me, response| match response {
                        GhostCallbackData::Response(Ok(
                                                        RequestToChildResponse::SendMessageSuccess,
                                                    )) => Ok(()),
                        GhostCallbackData::Response(Err(e)) => Err(e.into()),
                        GhostCallbackData::Timeout(bt) => {
                            Err(format!("timeout: {:?}", bt).into())
                        }
                        _ => Err("bad response type".into()),
                    }),
                );

            if let Err(e) = send_result {
                error!("GhostError during broadcast send: {:?}", e)
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use lib3h::transport::memory_mock::{
        ghost_transport_memory::*, memory_server::get_memory_verse,
    };
    use lib3h_protocol::data_types::*;

    fn make_test_agent() -> AgentId {
        "fake_agent_id".into()
    }

    fn make_test_space_data() -> SpaceData {
        SpaceData {
            request_id: "".into(),
            space_address: "fake_space_address".into(),
            agent_id: make_test_agent(),
        }
    }

    fn make_test_space_data_with_agent(agent_id: AgentId) -> SpaceData {
        SpaceData {
            request_id: "".into(),
            space_address: "fake_space_address".into(),
            agent_id,
        }
    }

    fn make_test_join_message() -> WireMessage {
        make_test_join_message_with_space_data(make_test_space_data())
    }

    fn make_test_join_message_with_space_data(space_data: SpaceData) -> WireMessage {
        WireMessage::ClientToLib3h(ClientToLib3h::JoinSpace(space_data))
    }

    fn make_test_leave_message() -> WireMessage {
        WireMessage::ClientToLib3h(ClientToLib3h::LeaveSpace(make_test_space_data()))
    }

    fn make_test_dm_data() -> DirectMessageData {
        DirectMessageData {
            request_id: "".into(),
            space_address: "fake_space_address".into(),
            from_agent_id: make_test_agent(),
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

    fn make_test_sim2h_nonet() -> Sim2h {
        let transport = Box::new(GhostTransportMemory::new("null".into(), "nullnet".into()));
        Sim2h::new(transport)
    }

    fn make_test_sim2h_memnet(netname: &str) -> Sim2h {
        let transport_id = "test_transport".into();
        let transport = Box::new(GhostTransportMemory::new(transport_id, netname));
        Sim2h::new(transport)
    }

    #[test]
    pub fn test_constructor() {
        let mut sim2h = make_test_sim2h_nonet();
        {
            let reader = sim2h.connection_states.read();
            assert_eq!(reader.len(), 0);
        }
        let result = sim2h.process();
        assert_eq!(result, Ok(()));
        assert_eq!(
            "Some(Lib3hUri(\"mem://addr_1/\"))",
            format!("{:?}", sim2h.bound_uri)
        );
    }

    #[test]
    pub fn test_incomming_connection() {
        let sim2h = make_test_sim2h_nonet();

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
        let mut sim2h = make_test_sim2h_nonet();
        let uri = Lib3hUri::with_memory("addr_1");

        let data = make_test_space_data();
        // you can't join if you aren't in limbo
        let result = sim2h.join(&uri, &data);
        assert_eq!(
            result,
            Err(format!("no agent found in limbo at {} ", &uri).into())
        );

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
    pub fn test_leave() {
        let mut sim2h = make_test_sim2h_nonet();
        let uri = Lib3hUri::with_memory("addr_1");
        let mut data = make_test_space_data();

        // leaving a space not joined should produce an error
        let result = sim2h.leave(&uri, &data);
        assert_eq!(
            result,
            Err(format!("no joined agent found at {} ", &uri).into())
        );
        let _result = sim2h.handle_incoming_connect(uri.clone());
        let result = sim2h.leave(&uri, &data);
        assert_eq!(
            result,
            Err(format!("no joined agent found at {} ", &uri).into())
        );

        let _result = sim2h.join(&uri, &data);

        // a leave on behalf of someone else should fail
        data.agent_id = "someone_else_agent_id".into();
        let result = sim2h.leave(&uri, &data);
        assert_eq!(result, Err(SPACE_MISMATCH_ERR_STR.into()));

        // a valid leave should work
        data.agent_id = make_test_agent();
        let result = sim2h.leave(&uri, &data);
        assert_eq!(result, Ok(()));
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!(result, None);
    }

    #[test]
    pub fn test_prepare_proxy() {
        let mut sim2h = make_test_sim2h_nonet();

        let uri = Lib3hUri::with_memory("addr_1");
        let _ = sim2h.handle_incoming_connect(uri.clone());
        let _ = sim2h.join(&uri, &make_test_space_data());
        let message = make_test_join_message();
        let data = make_test_space_data();

        // you can't proxy a join message
        let result = sim2h.prepare_proxy(&uri, &data.space_address, &data.agent_id, message);
        assert!(result.is_err());

        // you can't proxy for someone else, i.e. the message contents must match the
        // space joined
        let message = make_test_dm_message();
        let result = sim2h.prepare_proxy(
            &uri,
            &data.space_address,
            &"fake_other_agent".into(),
            message,
        );
        assert_eq!(Err("space/agent id mismatch".into()), result);

        // you can't proxy to someone not in the space
        let message = make_test_dm_message();
        let result =
            sim2h.prepare_proxy(&uri, &data.space_address, &data.agent_id, message.clone());
        assert_eq!(
            Err("unvalidated proxy agent fake_to_agent_id".into()),
            result,
        );

        // proxy a dm message
        // first we have to setup the to agent in the space
        let to_agent_data = make_test_space_data_with_agent("fake_to_agent_id".into());
        let to_uri = Lib3hUri::with_memory("addr_2");
        let _ = sim2h.handle_incoming_connect(to_uri.clone());
        let _ = sim2h.join(&to_uri, &to_agent_data);

        let result = sim2h.prepare_proxy(&uri, &data.space_address, &data.agent_id, message);
        assert_eq!(
            "Ok(Some((true, Lib3hUri(\"mem://addr_2/\"), Lib3hToClient(HandleSendDirectMessage(DirectMessageData { space_address: SpaceHash(HashString(\"fake_space_address\")), request_id: \"\", to_agent_id: HashString(\"fake_to_agent_id\"), from_agent_id: HashString(\"fake_agent_id\"), content: \"foo\" })))))",
            format!("{:?}", result)
        );

        // proxy a leave space message should remove the agent from the space
        let message = make_test_leave_message();
        let result = sim2h.prepare_proxy(&uri, &data.space_address, &data.agent_id, message);
        assert_eq!("Ok(None)", format!("{:?}", result));
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!(result, None);
    }

    #[test]
    pub fn test_message() {
        let netname = "test_message";
        let mut sim2h = make_test_sim2h_memnet(netname);
        let uri = Lib3hUri::with_memory("addr_x");

        // a message from an unconnected agent should return an error
        let result = sim2h.handle_message(&uri, make_test_err_message());
        assert_eq!(result, Err(format!("no connection for {}", &uri).into()));

        // a non-join message from an unvalidated but connected agent should return an error
        let _result = sim2h.handle_incoming_connect(uri.clone());
        let result = sim2h.handle_message(&uri, make_test_err_message());
        assert_eq!(
            result,
            Err(format!("no agent validated at {} ", &uri).into())
        );

        // a valid join message from a connected agent should update its connection status
        let result = sim2h.handle_message(&uri, make_test_join_message());
        assert_eq!(result, Ok(()));
        let result = sim2h.get_connection(&uri).clone();
        assert_eq!(
            "Some(JoinedSpace(SpaceHash(HashString(\"fake_space_address\")), HashString(\"fake_agent_id\")))",
            format!("{:?}", result)
        );

        // dm
        // first we have to setup the to agent on the in-memory-network and in the space
        let network = {
            let mut verse = get_memory_verse();
            verse.get_network(netname)
        };
        let to_uri = network.lock().unwrap().bind();
        let _ = sim2h.handle_incoming_connect(to_uri.clone());
        let to_agent_data = make_test_space_data_with_agent("fake_to_agent_id".into());
        let _ = sim2h.join(&to_uri, &to_agent_data);

        // then we can make a message and handle it.
        let message = make_test_dm_message();
        let result = sim2h.handle_message(&uri, message);
        assert_eq!(result, Ok(()));

        // which should result in showing up in the to_uri's inbox in the in-memory netowrk
        let result = sim2h.process();
        assert_eq!(result, Ok(()));
        let mut reader = network.lock().unwrap();
        let server = reader
            .get_server(&to_uri)
            .expect("there should be a server for to_uri");
        if let Ok((did_work, events)) = server.process() {
            assert!(did_work);
            let dm = &events[1];
            assert_eq!(
                "ReceivedData(Lib3hUri(\"mem://addr_2/\"), \"{\\\"Lib3hToClient\\\":{\\\"HandleSendDirectMessage\\\":{\\\"space_address\\\":\\\"fake_space_address\\\",\\\"request_id\\\":\\\"\\\",\\\"to_agent_id\\\":\\\"fake_to_agent_id\\\",\\\"from_agent_id\\\":\\\"fake_agent_id\\\",\\\"content\\\":\\\"Zm9v\\\"}}}\")",
                format!("{:?}", dm))
        } else {
            assert!(false)
        }
    }

    #[test]
    pub fn test_end_to_end() {
        let netname = "test_end_to_end";
        let mut sim2h = make_test_sim2h_memnet(netname);
        let _result = sim2h.process();
        let sim2h_uri = sim2h.bound_uri.clone().expect("should have bound");

        // set up two other agents on the memory-network
        let network = {
            let mut verse = get_memory_verse();
            verse.get_network(netname)
        };
        let agent1_uri = network.lock().unwrap().bind();
        let agent2_uri = network.lock().unwrap().bind();

        // connect them to sim2h with join messages
        let space_data1 = make_test_space_data_with_agent("agent1".into());
        let space_data2 = make_test_space_data_with_agent("agent2".into());
        let join1 : Opaque = make_test_join_message_with_space_data(space_data1.clone()).into();
        let join2 : Opaque = make_test_join_message_with_space_data(space_data2.clone()).into();
        {
        let mut net = network.lock().unwrap();
        let server = net
            .get_server(&sim2h_uri)
            .expect("there should be a server for to_uri");
        server.request_connect(&agent1_uri).expect("can connect");
        let result = server.post(&agent1_uri, &join1.to_vec());
        assert_eq!(result, Ok(()));
        server.request_connect(&agent2_uri).expect("can connect");
        let result = server.post(&agent2_uri, &join2.to_vec());
        assert_eq!(result, Ok(()));
        }

        let _result = sim2h.process();

        assert_eq!(
            sim2h.lookup_joined(&space_data1.space_address, &space_data1.agent_id),
            Some(agent1_uri.clone())
        );
        assert_eq!(
            sim2h.lookup_joined(&space_data2.space_address, &space_data2.agent_id),
            Some(agent2_uri.clone())
        );

    }
}
