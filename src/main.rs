use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use crate::logging::logger;
use crate::logging::logger::Log;
use crate::common::{ActionResult, Action, ActionType, Operation, Command, CommandType, Event, EventType, RunCommand};
use crate::events::EventLoop;
use crate::http::client::HttpClient;
use crate::http::common::HttpResponse;
use crate::http::server::HttpServer;
use crate::logger::Logger;
use crate::orchestrating::Orchestrator;
use crate::results::ResultHandler;

mod logging;
mod common;
mod orchestrating;
mod events;
mod results;
mod http;

struct Node {
    log: Log,
    event_loop: EventLoop,
    orchestrator: Orchestrator,
    result_handler: ResultHandler,
    http_server: HttpServer,
}

impl Node {
    pub fn start() -> Node {
        let log = Log::start().unwrap();

        let (event_sender, event_receiver) = channel::<Event>();
        let (command_sender, command_receiver) = channel::<Command>();
        let (result_sender, result_receiver) = channel::<ActionResult>();

        let event_loop = EventLoop::start(command_sender.clone(), event_receiver, event_sender.clone(), &log);

        let orchestrator = Orchestrator::start(result_sender, command_receiver, command_sender.clone(), &log);

        let result_handler = ResultHandler::start(event_sender.clone(),  result_receiver, &log);
        
        let http_server = HttpServer::create("0.0.0.0:61409".to_string(), event_sender, command_sender, &log).unwrap();

        Node {
            log,
            event_loop,
            orchestrator,
            result_handler,
            http_server
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
    
    match HttpClient::connect("192.168.0.226:80".to_string()) {
        Ok(mut client) => {
            match client.get("/".to_string(), "text/plain".to_string(), HashMap::new()){
                Ok(response) => {
                    println!("Header\n\r{}", response.header.get_string());
                    match response.body {
                        None => {}
                        Some(raw) => {
                            println!("Body\n\r{}", String::from_utf8(raw).unwrap())
                        }
                    }
                    
                }
                Err(e) => {
                    println!("{}", e)
                }
            };
        }
        Err(_) => {}
    };
    
    let node = Node::start();



    loop {
        
    }
    
    /*
    loop {
       
        let event = Event { id: Uuid::new_v4(), event_type: EventType::Test };
        
        node.raise_event(event);
        
        thread::sleep(Duration::from_secs(2));
        
        let run_command = Command { id: Uuid::new_v4(), command_type: CommandType::Run(RunCommand { message: "Test run".to_string() }) };
        
        node.queue_command(run_command);
    }
    */
}
