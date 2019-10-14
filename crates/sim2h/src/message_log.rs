use crate::WireMessage;
use lib3h_protocol::uri::Lib3hUri;
use lib3h_protocol::Address;
use parking_lot::Mutex;
use std::collections::LinkedList;

#[derive(Serialize)]
enum Direction {
    In,
    Out,
}

#[derive(Serialize)]
struct MessageLog {
    uri: Lib3hUri,
    agent: Address,
    direction: Direction,
    message: WireMessage,
}

lazy_static! {
    pub static ref MESSAGE_LOGGER: Mutex<MessageLogger> = Mutex::new(MessageLogger::new());
}

pub struct MessageLogger {
    buffer: LinkedList<MessageLog>,
}

impl MessageLogger {
    pub fn new() -> Self {
        MessageLogger {
            buffer: LinkedList::new(),
        }
    }

    pub fn log_in(&mut self, agent: Address, uri: Lib3hUri, message: WireMessage) {
        self.buffer.push_back(MessageLog {
            uri,
            agent,
            direction: Direction::In,
            message,
        });
    }

    pub fn log_out(&mut self, agent: Address, uri: Lib3hUri, message: WireMessage) {
        self.buffer.push_back(MessageLog {
            uri,
            agent,
            direction: Direction::Out,
            message,
        });
    }
}
