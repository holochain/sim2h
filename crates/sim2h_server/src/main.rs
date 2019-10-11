extern crate structopt;

use lib3h::transport::websocket::tls::TlsCertificate;
use lib3h::transport::{
    protocol::DynTransportActor,
    websocket::{actor::GhostTransportWebsocket, tls::TlsConfig},
};
use lib3h_protocol::{uri::Builder, Address};
use log::error;
use sim2h::Sim2h;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        long,
        short,
        help = "The port to run the websocket server at",
        default_value = "9000"
    )]
    port: u16,
}

fn create_websocket_transport() -> DynTransportActor {
    Box::new(GhostTransportWebsocket::new(
        Address::from("sim2h-worker-transport"),
        TlsConfig::SuppliedCertificate(TlsCertificate::build_from_entropy()),
        Address::from("sim2h-network"),
    ))
}

fn main() {
    env_logger::init();
    let transport = create_websocket_transport();

    let args = Cli::from_args();

    let host = "wss://0.0.0.0/";
    let uri = Builder::with_raw_url(host)
        .unwrap_or_else(|e| panic!("with_raw_url: {:?}", e))
        .with_port(args.port)
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
