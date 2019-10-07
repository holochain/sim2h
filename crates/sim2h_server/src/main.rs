use detach::{Detach};
use lib3h::transport::protocol::{TransportActorParentWrapperDyn};
use lib3h::transport::websocket::actor::GhostTransportWebsocket;
use lib3h::transport::websocket::tls::TlsConfig;
use lib3h_protocol::Address;
use sim2h::Sim2h;
use log::error;
use std::process::exit;

fn create_websocket_transport() -> Detach<TransportActorParentWrapperDyn<Sim2h>>{
    Detach::new(TransportActorParentWrapperDyn::new(
        Box::new(GhostTransportWebsocket::new(
            Address::from("sim2h-worker-transport"),
            TlsConfig::Unencrypted,
            Address::from("sim2h-network"),
        )),
        "transport_",
    ))
}



fn main() {
    let transport = create_websocket_transport();
    let mut sim2h = Sim2h::with_detached_transport(transport);
    match sim2h.bind_transport_sync(9000) {
        Ok(bound_uri) => println!("Successfully bound to: {:?}", bound_uri),
        Err(error) => {
            error!("Could not bind to network. Error: {:?}", error);
            exit(1);
        }
    }

    loop {
        let _ = sim2h.process();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
