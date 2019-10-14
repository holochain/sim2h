use crate::WireMessage;
use chrono::{DateTime, Utc};
use lib3h_protocol::uri::Lib3hUri;
use lib3h_protocol::Address;
use log::error;
use parking_lot::Mutex;
use std::collections::LinkedList;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Debug)]
enum Direction {
    In,
    Out,
}

#[derive(Serialize)]
struct MessageLog {
    time: String,
    uri: Lib3hUri,
    agent: Address,
    direction: Direction,
    message: WireMessage,
}

lazy_static! {
    pub static ref MESSAGE_LOGGER: Mutex<MessageLogger> = {
        MessageLogger::write_thread();
        Mutex::new(MessageLogger::new())
    };
}

pub struct MessageLogger {
    buffer: LinkedList<MessageLog>,
    file_path: String,
}

impl MessageLogger {
    pub fn new() -> Self {
        MessageLogger {
            buffer: LinkedList::new(),
            file_path: String::from("sim2h_messages.log"),
        }
    }

    fn write_thread() {
        std::thread::Builder::new()
            .name("MessageLogger".into())
            .spawn(|| {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let mut logger = MESSAGE_LOGGER.lock();
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(logger.file_path())
                    {
                        let to_append = logger
                            .buffer
                            .split_off(0)
                            .into_iter()
                            //.drain_filter(|_| true)
                            .map(|log| {
                                format!(
                                    "{}\t{:?}\t{}\t{}\t{}",
                                    log.time,
                                    log.direction,
                                    log.agent,
                                    log.uri,
                                    serde_json::to_string(&log.message)
                                        .expect("Message must be serializable")
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        if let Err(e) = file.write(to_append.as_bytes()) {
                            error!("Error writing log file: {:?}", e);
                        }
                    } else {
                        error!("Could not open log file!")
                    }
                }
            })
            .expect("Could not spaw logger thread");
    }

    fn time() -> String {
        let now: DateTime<Utc> = Utc::now();
        format!("{}", now)
    }

    pub fn log_in(&mut self, agent: Address, uri: Lib3hUri, message: WireMessage) {
        self.buffer.push_back(MessageLog {
            time: Self::time(),
            uri,
            agent,
            direction: Direction::In,
            message,
        });
    }

    pub fn log_out(&mut self, agent: Address, uri: Lib3hUri, message: WireMessage) {
        self.buffer.push_back(MessageLog {
            time: Self::time(),
            uri,
            agent,
            direction: Direction::Out,
            message,
        });
    }

    pub fn set_logfile<T: Into<String>>(&mut self, path: T) {
        self.file_path = path.into();
    }

    pub fn file_path(&self) -> String {
        self.file_path.clone()
    }
}
