use std::collections::HashMap;
use std::str::Split;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;
use serde_json::map::Keys;
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
use crate::io::network::{NameResolver, ResolverMessage};

mod logging;
mod common;
mod orchestrating;
mod events;
mod results;
mod http;
mod io;


/*
fn test() {
    let mut client = HttpClient::create("192.168.0.226:80".to_string());
    let request1 = r#"
    {
        "newState": 1
    }
    "#;
    
    let request2 = r#"
    {
        "newState": 2
    }
    "#;

    let request1_parsed: UpdateNodeStateRequest = serde_json::from_str(request1).unwrap();
    let request2_parsed: UpdateNodeStateRequest = serde_json::from_str(request2).unwrap();
    
    let response1 = client.get(format!("/set-state/{}", request1_parsed.new_state), "text/plain".to_string(), HashMap::new()).unwrap();
    
    match response1.body {
        None => {}
        Some(body) => {
            let result: UpdateNodeStateResponse = serde_json::from_slice(&body).unwrap();
            println!("Result: {} - Old state: {} New state: {}", result.result, result.old_state, result.new_state)
        }
    }
    
    thread::sleep(Duration::from_secs(1));

    let  response1_again = client.get(format!("/set-state/{}", request1_parsed.new_state), "text/plain".to_string(), HashMap::new()).unwrap();
    
    match response1_again.body {
        None => {}
        Some(body) => {
            let result: UpdateNodeStateResponse = serde_json::from_slice(&body).unwrap();
            println!("Result: {} - Old state: {} New state: {}", result.result, result.old_state, result.new_state)
        }
    }

    thread::sleep(Duration::from_secs(1));

    let response2 = client.get(format!("/set-state/{}", request2_parsed.new_state), "text/plain".to_string(), HashMap::new()).unwrap();

    match response2.body {
        None => {}
        Some(body) => {
            let result: UpdateNodeStateResponse = serde_json::from_slice(&body).unwrap();
            println!("Result: {} - Old state: {} New state: {}", result.result, result.old_state, result.new_state)
        }
    }

    thread::sleep(Duration::from_secs(1));

    let response3 = client.get("/get-state".to_string(), "text/plain".to_string(), HashMap::new()).unwrap();

    match response3.body {
        None => {}
        Some(body) => {
            let result: GetNodeStateResponse = serde_json::from_slice(&body).unwrap();
            println!("Current state: {}", result.state)
        }
    }

    thread::sleep(Duration::from_secs(1));
}
*/

struct Controller {
    log: Log,
    event_loop: EventLoop,
    orchestrator: Orchestrator,
    result_handler: ResultHandler,
    http_server: HttpServer,
}

impl Controller {
    pub fn start() -> Controller {
        let log = Log::start().unwrap();

        let (event_sender, event_receiver) = channel::<Event>();
        let (command_sender, command_receiver) = channel::<Command>();
        let (result_sender, result_receiver) = channel::<ActionResult>();
        let (nr_sender, nr_receiver) = channel::<ResolverMessage>();

        let mut name_map = HashMap::new();
        
        name_map.insert("node1".to_string(), "192.168.0.226:80".to_string());
        
        let name_resolver = NameResolver::start(name_map, nr_receiver);
        
        let event_loop = EventLoop::start(command_sender.clone(), event_receiver, event_sender.clone(), &log);

        let orchestrator = Orchestrator::start(result_sender, command_receiver, command_sender.clone(), nr_sender.clone(), &log);

        let result_handler = ResultHandler::start(event_sender.clone(), result_receiver, &log);

        let http_server = HttpServer::create("0.0.0.0:61409".to_string(), event_sender, command_sender, nr_sender, &log).unwrap();

        Controller {
            log,
            event_loop,
            orchestrator,
            result_handler,
            http_server,
        }
    }

    pub fn raise_event(&self, event: Event) {
        self.event_loop.raise_event(event);
    }

    pub fn queue_command(&self, command: Command) {
        self.orchestrator.queue_command(command);
    }
}

struct Url {
    parts: Vec<String>,
    query_parameters: HashMap<String, String>,
}

impl Url {
    pub fn parse(value: String) -> Url {
        let split_parts = value.split('/');

        let mut parts: Vec<String> = Vec::new();
        let mut query_parameters: HashMap<String, String> = HashMap::new();

        //let len = &split_parts.count() - 1;
        
        for p in split_parts.skip(1) {
            if p.contains('?') {
                let p_split: Vec<&str> = p.split('?').collect();
                match p_split.first() {
                    None => {}
                    Some(s) => {
                        parts.push(s.to_string())
                    }
                }

                //parts.push(p_split.first().unwrap().to_string());
                match p.split('?').last() {
                    None => {}
                    Some(q) => {
                        for qp in q.split('&') {
                            let qpp: Vec<&str> = qp.split("=").collect();

                            if qpp.len() == 2 {
                                query_parameters.insert(qpp[0].to_string(), qpp[1].to_string());
                            }
                        }
                    }
                }
            }
            else {
                parts.push(p.to_string());
            }
            
        };

        Url {
            parts,
            query_parameters,
        }
    }

    fn print(&self) {
        println!("Parts:");

        for part in &self.parts {
            println!("{}", part)
        }

        println!("Query:");

        for (k, v) in &self.query_parameters {
            println!("Key: {} Value: {}", k, v);
        }
    }
}

fn parse_url() {}

fn test() {
    let url1 = Url::parse("/".to_string());
    let url2 = Url::parse("/home/test?k=v".to_string());
    
    url1.print();
    url2.print();
}

fn main() {
    test();

    /*
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
    */

    let controller = Controller::start();


    loop {}

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
