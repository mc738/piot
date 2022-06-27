use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use crate::logging::logger;
use crate::logging::logger::Log;
use crate::common::{ActionResult, Action, ActionType, Operation, Command, CommandType, Event, EventType};
use crate::events::EventLoop;
use crate::logger::Logger;
use crate::orchestrating::Orchestrator;
use crate::results::ResultHandler;

mod logging;
mod common;
mod orchestrating;
mod events;
mod results;

struct Node {
    log: Log,
    event_loop: EventLoop,
    orchestrator: Orchestrator,
    result_handler: ResultHandler
}

impl Node {
    pub fn start() -> Node {
        let log = Log::start().unwrap();

        let (event_sender, event_receiver) = channel::<Event>();
        let (command_sender, command_receiver) = channel::<Command>();
        let (result_sender, result_receiver) = channel::<ActionResult>();

        let event_loop = EventLoop::start(command_sender.clone(), event_receiver, event_sender.clone(), &log);

        let orchestrator = Orchestrator::start(result_sender, command_receiver, command_sender, &log);

        let result_handler = ResultHandler::start(event_sender,  result_receiver, &log);

        Node {
            log,
            event_loop,
            orchestrator,
            result_handler
        }
    }
    
    pub fn raise_event(&self, event: Event) {
        self.event_loop.raise_event(event);
    }
    
    pub fn queue_command(&self, command: Command) {
        self.orchestrator.queue_command(command);
    }
}

fn do_something(action: Action) -> ActionResult {
    let ops = vec![ Operation::Test ];
    
    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}





fn main() {
    
    let node = Node::start();
    
    loop {
       
        let event = Event { id: Uuid::new_v4(), event_type: EventType::Test };
        
        node.raise_event(event);
        
        thread::sleep(Duration::from_secs(1))
    }
}
