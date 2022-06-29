mod action_handler;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::{Action, ActionResult, ActionType, Command, CommandType, Log, Logger, ResolverMessage};
use crate::common::{ChangeNodeStateAction, RunAction};
use crate::orchestrating::action_handler::handle_action;

type Job = Box<dyn FnOnce() -> ActionResult + Send + 'static>;

pub(crate) struct Orchestrator {
    sender: Sender<Command>,
    thread: JoinHandle<()>,
}

pub(crate) struct WorkerPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
    logger: Logger,
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Orchestrator {
    pub fn start(result_sender: Sender<ActionResult>, command_receiver: Receiver<Command>, command_sender: Sender<Command>, nr_sender: Sender<ResolverMessage>, log: &Log) -> Orchestrator {
        //let (tx, rx) = channel::<Command>();
        let logger = log.get_logger("orchestrator".to_string());

        logger.log_info("Starting".to_string()).unwrap();
        let workers = WorkerPool::new(4, result_sender.clone(), log);

        let thread = thread::spawn(move || loop {
            let command = command_receiver.recv().unwrap();

            logger.log_info(format!("Command {} received", command.id)).unwrap();

            // Handle turning the command into an action.
            let action =
                match command.command_type {
                    CommandType::Test => {
                        Action { id: command.id, action_type: ActionType::Test }
                    }
                    CommandType::Run(run_command) => {
                        Action { id: command.id, action_type: ActionType::Run(RunAction { message: run_command.message }) }
                    }
                    CommandType::ChangeNodeState(new_state) => {
                        Action { id: command.id, action_type: ActionType::ChangeNodeState(ChangeNodeStateAction { node: new_state.node, new_state: new_state.new_state }) }
                    }
                };

            let action_logger = logger.create_from(format!("action_{}", action.id));

            let name_resolver = nr_sender.clone();
            workers.execute(|| handle_action(action, name_resolver, action_logger));
        });

        Orchestrator { sender: command_sender, thread }
    }

    pub fn queue_command(&self, command: Command) {
        self.sender.send(command).unwrap();
    }
}

impl WorkerPool {
    pub fn new(size: usize, result_handler: Sender<ActionResult>, log: &Log) -> WorkerPool {
        let logger = log.get_logger("worker_pool".to_string());

        logger.log_info("Starting".to_string()).unwrap();

        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let name = format!("worker_{}", id);

            let worker_logger = log.get_logger(name);

            worker_logger.log_info("Starting".to_string()).unwrap();
            workers.push(Worker::new(id, Arc::clone(&receiver), result_handler.clone(), worker_logger))
        }

        WorkerPool { workers, sender, logger }
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