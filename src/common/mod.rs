use std::marker::PhantomData;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use uuid::Uuid;
use crate::Log;
use crate::logger::Logger;

pub(crate) struct Command {
    pub(crate) id: Uuid,
    pub(crate) command_type: CommandType,
}

pub(crate) enum CommandType {
    Test
}

pub(crate) struct Orchestrator {
    sender: Sender<Command>,
}

pub(crate) struct Action {
    pub(crate) id: Uuid,
    pub(crate) action_type: ActionType,
}

pub(crate) enum ActionType {
    Test
}

pub(crate) struct ActionResult {
    pub(crate) id: Uuid,
    pub(crate) successful: bool,
    pub(crate) message: String,
    pub(crate) ops: Vec<Operation>,
}

pub enum Operation {
    Test
}

pub(crate) struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
    logger: Logger,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

pub(crate) struct ResultHandler {
    sender: Sender<ActionResult>,
}

type Job = Box<dyn FnOnce() -> ActionResult + Send + 'static>;

impl Orchestrator {
    pub fn start(result_handler: Sender<ActionResult>, log: &Log) -> Orchestrator {
        let (tx, rx) = channel::<Command>();
        let logger = log.get_logger("orchestrator".to_string());

        logger.log_info("Orchestrator starting".to_string()).unwrap();
        let workers = ThreadPool::new(4, result_handler.clone(), log);

        thread::spawn(move || loop {
            let command = rx.recv().unwrap();

            logger.log_info(format!("Command {} received", command.id)).unwrap();

            // Handle turning the command into an action.
            let action =
                match command.command_type {
                    CommandType::Test => {
                        Action { id: command.id, action_type: ActionType::Test }
                    }
                };
            
            let action_logger = logger.create_from(format!("action-{}", action.id));

            workers.execute(|| handle_action(action, action_logger));
        });

        Orchestrator { sender: tx }
    }

    pub fn get_sender(&self) -> Sender<Command> {
        self.sender.clone()
    }

    pub fn queue_command(&self, command: Command) {
        self.sender.send(command).unwrap();
    }
}

impl ThreadPool {
    pub fn new(size: usize, result_handler: Sender<ActionResult>, log: &Log) -> ThreadPool {
        let logger = log.get_logger("thread_pool".to_string());

        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let name = format!("worker-{}", id);

            let worker_logger = log.get_logger(name);

            workers.push(Worker::new(id, Arc::clone(&receiver), result_handler.clone(), worker_logger))
        }

        ThreadPool { workers, sender, logger }
    }

    pub fn execute<F>(&self, f: F) where
        F: FnOnce() -> ActionResult + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, result_handler: Sender<ActionResult>, logger: Logger) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            logger.log_info("Job received".to_string()).unwrap();

            let result = job();

            match result.successful {
                true => logger.log_success("Job completed successfully".to_string()).unwrap(),
                false => logger.log_error("Job failed".to_string()).unwrap(),
            };

            result_handler.send(result).unwrap();
        });

        Worker { id, thread }
    }
}

impl ResultHandler {
    pub fn start(log: &Log) -> ResultHandler {
        let (tx, rx) = channel::<ActionResult>();
        let logger = log.get_logger("result_handler".to_string());

        thread::spawn(move || loop {
            let result = rx.recv().unwrap();
            
            result_handler(result, &logger);
        });

        ResultHandler { sender: tx }
    }

    pub fn get_sender(&self) -> Sender<ActionResult> {
        self.sender.clone()
    }
}

fn handle_action(action: Action, logger: Logger) -> ActionResult {
    let ops = vec![Operation::Test];

    logger.log_info("Action completed".to_string()).unwrap();
    
    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}

fn result_handler(result: ActionResult, logger: &Logger) {
    match result.successful {
        true => logger.log_success(format!("Action {} success. Message - {}", result.id, result.message)).unwrap(),
        false => logger.log_error(format!("Action {} failed. Message - {}", result.id, result.message)).unwrap(),
    };
}