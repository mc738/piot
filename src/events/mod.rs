mod event_handler;

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use crate::{Action, Command, Log};
use crate::common::Event;
use crate::events::event_handler::handle_event;

pub(crate) struct EventLoop {
    sender: Sender<Event>,
    thread: JoinHandle<()>,
}

impl EventLoop {
    pub fn start(command_sender: Sender<Command>, event_receiver: Receiver<Event>, event_sender: Sender<Event>, log: &Log) -> EventLoop {
        let logger = log.get_logger("event-loop".to_string());

        logger.log_info("Starting".to_string()).unwrap();
        
        let thread = thread::spawn(move || loop {
            let event = event_receiver.recv().unwrap();
            // Convert event to command(s).
            let commands = handle_event(event, &logger);
            // Send commands.
            for command in commands {
                command_sender.send(command).unwrap();
            }
        });

        EventLoop { sender: event_sender, thread }
    }
    
    pub fn raise_event(&self, event: Event) {
        self.sender.send(event).unwrap()
    }
}