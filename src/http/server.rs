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
use crate::http::common::{HttpRequest, HttpStatus};

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

            let body = Some("Hello, World!".as_bytes().to_vec());
            thread::sleep(Duration::from_secs(5));

            let response = HttpResponse::create(HttpStatus::Ok, "text/plain".to_string(), HashMap::new(), body);
            match context.send_response(response) {
                Ok(_) => {
                    context.logger.log_success("Response sent.".to_string()).unwrap();
                    match fetch() {
                        Ok(response) => {
                            context.logger.log_success(format!("Response from device - {}", response)).unwrap();
                            context.event_sender.send(Event { id: context.id, event_type: EventType::Test }).unwrap();
                        }
                        Err(_) => {}
                    }
                }
                Err(message) => {
                    context.logger.log_error(format!("Error sending response - {}", message)).unwrap();
                }
            }
        }
        Err(message) => {
            context.logger.log_error(format!("Could not get request. Error - {}", message)).unwrap();
        }
    }
}

// TODO remove this - test code
fn fetch() -> Result<String, &'static str> {
    let mut client = HttpClient::connect("192.168.0.226:80".to_string())?;
    let response = client.get("/".to_string(), "text/plain".to_string(), HashMap::new())?;
    match response.body {
        None => Ok("".to_string()),
        Some(raw) => Ok(String::from_utf8(raw).unwrap())
    }
}