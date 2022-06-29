use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::id;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use uuid::Uuid;
use crate::{Command, Event, EventType, HttpClient, HttpResponse, Log, Logger};
use crate::http::common::{HttpRequest, HttpStatus, HttpVerb};
use crate::io::{UpdateNodeStateRequest, UpdateNodeStateResponse, GetNodeStateResponse};

use std::str::from_utf8;

pub(crate) struct HttpServer {
    thread: JoinHandle<()>,
}

type Connection = Box<dyn FnOnce() + Send + 'static>;

struct ConnectionPool {
    handlers: Vec<ConnectionHandler>,
    sender: Sender<Connection>,
    logger: Logger,
}

struct ConnectionHandler {
    id: usize,
    thread: JoinHandle<()>,
}

struct ConnectionContext {
    id: Uuid,
    slug: String,
    from: String,
    event_sender: Sender<Event>,
    command_sender: Sender<Command>,
    stream: TcpStream,
    logger: Logger,
}

impl HttpServer {
    pub fn create(address: String, event_sender: Sender<Event>, command_sender: Sender<Command>, log: &Log) -> Result<HttpServer, &'static str> {
        let logger = log.get_logger("http_server".to_string());
        let connection_pool = ConnectionPool::new(4, log);
        match TcpListener::bind(address) {
            Ok(listener) => {
                let thread = thread::spawn(move || loop {
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let remote = stream.peer_addr().unwrap();
                                logger.log_info(format!("Request received from {}", remote)).unwrap();
                                let context = ConnectionContext::create(String::from(remote.ip().to_string()), event_sender.clone(), command_sender.clone(), stream, &logger);

                                let es = event_sender.clone();
                                let cs = command_sender.clone();
                                connection_pool.handle_connect(|| { handle_connect(context) });
                            }
                            Err(_) => {
                                logger.log_error("Unable to connection to stream".to_string()).unwrap();
                            }
                        }
                    }
                });

                Ok(HttpServer {
                    thread,
                })
            }
            Err(_) => Err("Could not bind to address.")
        }
    }
}

impl ConnectionPool {
    pub fn new(size: usize, log: &Log) -> ConnectionPool {
        let logger = log.get_logger("connection_pool".to_string());

        let (sender, receiver) = mpsc::channel();

        let mut handlers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let name = format!("worker_{}", id);

            let handler_logger = log.get_logger(name);
            handler_logger.log_info("Starting".to_string()).unwrap();

            handlers.push(ConnectionHandler::new(id, receiver.clone(), handler_logger));
        };

        ConnectionPool {
            handlers,
            sender,
            logger,
        }
    }

    fn handle_connect<F>(&self, f: F) where
        F: FnOnce() + Send + 'static,
    {
        let connection = Box::new(f);

        self.sender.send(connection).unwrap();
    }
}

impl ConnectionHandler {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Connection>>>, logger: Logger) -> ConnectionHandler {
        let thread = thread::spawn(move || loop {
            let connection = receiver.lock().unwrap().recv().unwrap();
            logger.log_info("Connection receiver".to_string()).unwrap();
            connection();
        });

        ConnectionHandler { id, thread }
    }
}

impl ConnectionContext {
    fn create(from: String, event_sender: Sender<Event>, command_sender: Sender<Command>, stream: TcpStream, logger: &Logger) -> ConnectionContext {
        let id = Uuid::new_v4();
        let slug = String::from(Uuid::new_v4().to_string().split_at(6).0);
        let connection_logger = logger.create_from(slug.clone());

        ConnectionContext {
            id,
            slug,
            from,
            event_sender,
            command_sender,
            stream,
            logger: connection_logger,
        }
    }

    fn get_request(&self) -> Result<HttpRequest, &'static str> {
        HttpRequest::from_stream(&self.stream, &self.logger)
    }

    fn send_response(&mut self, mut response: HttpResponse) -> Result<(), &'static str> {
        match self.stream.write(&response.to_bytes()) {
            Ok(_) => {
                match self.stream.flush() {
                    Ok(_) => { Ok(()) }
                    Err(_) => {
                        Err("Flush failed.")
                    }
                }
            }
            Err(_) => {
                Err("Write failed.")
            }
        }
    }
}

fn handle_connect(mut context: ConnectionContext) {
    match context.get_request() {
        Ok(request) => {
            context.logger.log_info(format!("Request for route {}. Type: {}", request.header.route, request.header.verb.get_str())).unwrap();

            let (response, event) = router(request);
            match context.send_response(response) {
                Ok(_) => {
                    context.logger.log_success("Response sent.".to_string()).unwrap();
                }
                Err(message) => {
                    context.logger.log_error(format!("Error sending response - {}", message)).unwrap();
                }
            }
            
            match event {
                None => {}
                Some(event) => {
                    context.logger.log_success(format!("Rising event - id: {}", event.id)).unwrap();
                    context.event_sender.send(event).unwrap();
                }
            }
        }
        Err(message) => {
            context.logger.log_error(format!("Could not get request. Error - {}", message)).unwrap();
        }
    }
}

struct UpdateStateRequest {
    new_state: i8
}


fn router(request: HttpRequest) -> (HttpResponse, Option<Event>) {
    // TODO move.
    let client = HttpClient::create("192.168.0.226:80".to_string());
    
    let set_state_route = String::from("/set-state");
    let get_state_route = String::from("/get-state/test");
    
    match (request.header.route, request.header.verb) {
        (set_state_route, HttpVerb::POST) => {
            match request.body {
                None => {
                    let body = Some("Missing request body.".as_bytes().to_vec());
                    (HttpResponse::create(HttpStatus::BadRequest, "text/plain".to_string(), HashMap::new(), body), None)
                }
                Some(request_body) => {
                    match UpdateNodeStateRequest::from_bytes(request_body) {
                        Ok(parsed_request) => {
                            match set_state(client, parsed_request) {
                                Ok(response) => {
                                    match response.to_bytes() {
                                        Ok(body) => {
                                            let event = Some(Event { id: Uuid::new_v4(), event_type: EventType::Test });
                                            (HttpResponse::create(HttpStatus::Ok, "application/json".to_string(), HashMap::new(), Some(body)), event)
                                        }
                                        Err(_) => {
                                            let body = Some("Could not serialize result.".as_bytes().to_vec());
                                            (HttpResponse::create(HttpStatus::InternalError, "text/plain".to_string(), HashMap::new(), body), None)
                                        }
                                    }
                                }
                                Err(_) => {
                                    let body = Some("Could not set state.".as_bytes().to_vec());
                                    (HttpResponse::create(HttpStatus::InternalError, "text/plain".to_string(), HashMap::new(), body), None)
                                }
                            }
                            
                        }
                        Err(_) => {
                            let body = Some("Invalid request.".as_bytes().to_vec());
                            (HttpResponse::create(HttpStatus::BadRequest, "text/plain".to_string(), HashMap::new(), body), None)
                        }
                    }
                }
            }
            //(HttpResponse::create(HttpStatus::Ok, "text/plain".to_string(), HashMap::new(), body), None)
        },
        (get_state_route, HttpVerb::GET) => {
            match get_state(client) {
                Ok(state) => {
                    //let body
                    let response =
                    match state.to_bytes() {
                        Ok(body) => {
                            HttpResponse::create(HttpStatus::Ok, "application/json".to_string(), HashMap::new(), Some(body))
                        }
                        Err(_) => {
                            let body = Some("Could not serialize result.".as_bytes().to_vec());
                            HttpResponse::create(HttpStatus::InternalError, "text/plain".to_string(), HashMap::new(), body)
                        }
                    };
                    (response, None)
                }
                Err(e) => {
                    println!("{}", e);
                    let body = Some("Could not get state.".as_bytes().to_vec());
                    (HttpResponse::create(HttpStatus::InternalError, "text/plain".to_string(), HashMap::new(), body), None)
                }
            }
        },
        (_, _) => {
            let body = Some("Not found".as_bytes().to_vec());
            (HttpResponse::create(HttpStatus::NotFound, "text/plain".to_string(), HashMap::new(), body), None)
        }
    }
}


fn set_state(mut client: HttpClient, request: UpdateNodeStateRequest) -> Result<UpdateNodeStateResponse, &'static str> {
    let response = client.get(format!("/set-state/{}", request.new_state), "text/plain".to_string(), HashMap::new())?;
    match response.body {
        None => {
            Err("No response body returned")
        }
        Some(body) => {
            match UpdateNodeStateResponse::from_bytes(body) {
                Ok(response) => Ok(response),
                Err(_) => Err("Unable to parse response")
            }
        }
    }
}


fn get_state(mut client: HttpClient) -> Result<GetNodeStateResponse, &'static str> {
    let response = client.get("/get-state".to_string(), "text/plain".to_string(), HashMap::new())?;
    match response.body {
        None => {
            println!("Error - No response body returned");
            Err("No response body returned")
        }
        Some(body) => {
            println!("{}", std::str::from_utf8(&body).unwrap());
            match GetNodeStateResponse::from_bytes(body) {
                Ok(response) => Ok(response),
                Err(_) => Err("Unable to parse response")
            }
        }
    }
}

// TODO remove this - test code
fn fetch() -> Result<String, &'static str> {
    let mut client = HttpClient::create("192.168.0.226:80".to_string());
    let response = client.get("/".to_string(), "text/plain".to_string(), HashMap::new())?;
    match response.body {
        None => Ok("".to_string()),
        Some(raw) => Ok(String::from_utf8(raw).unwrap())
    }
}
 