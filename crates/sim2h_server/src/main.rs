use lib3h::transport::{
    protocol::DynTransportActor,
    websocket::{actor::GhostTransportWebsocket, tls::TlsConfig},
};
use lib3h_protocol::{uri::Builder, Address};
use log::error;
use sim2h::Sim2h;
use std::process::exit;

fn create_websocket_transport() -> DynTransportActor {
    Box::new(GhostTransportWebsocket::new(
        Address::from("sim2h-worker-transport"),
        TlsConfig::Unencrypted,
        Address::from("sim2h-network"),
    ))
}

fn main() {
    env_logger::init();
    let transport = create_websocket_transport();

    let host = "wss://127.0.0.1/";
    let port = 9000;
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(port)
        .build();
    let mut sim2h = Sim2h::new(transport, uri);

    loop {
        let result = sim2h.process();
        if let Err(e) = result {
            if e.to_string().contains("Bind error:") {
                println!("{:?}", e);
                exit(1)
            } else {
                error!("{}", e.to_string())
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
