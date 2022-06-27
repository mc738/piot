mod result_handler;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use crate::{ActionResult, Log};
use crate::common::Event;
use crate::results::result_handler::handle_result;

pub(crate) struct ResultHandler {
    thread: JoinHandle<()>
}

impl ResultHandler {
    pub fn start(event_sender: Sender<Event>, result_receiver: Receiver<ActionResult>, log: &Log) -> ResultHandler {
        let logger = log.get_logger("result_handler".to_string());

        logger.log_info("Starting".to_string()).unwrap();
        
        let thread = thread::spawn(move || loop {
            let result = result_receiver.recv().unwrap();

            let events = handle_result(result, &logger);
            
            for event in events {
                event_sender.send(event).unwrap();
            }
        });

        ResultHandler { thread }
    }
}