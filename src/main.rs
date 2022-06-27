use std::thread;
use std::time::Duration;
use uuid::Uuid;
use crate::logging::logger;
use crate::logging::logger::Log;
use crate::common::{ThreadPool, ActionResult, Action, ActionType, ResultHandler, Operation, Orchestrator, Command, CommandType};
use crate::logger::Logger;

mod logging;
mod common;


fn do_something(action: Action) -> ActionResult {
    let ops = vec![ Operation::Test ];
    
    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}


fn main() {
    
    let log = Log::start().unwrap();
    
    let result_handler = ResultHandler::start(&log);
    
    let orchestrator = Orchestrator::start(result_handler.get_sender(), &log);
    
    loop {
       
        let command = Command { id: Uuid::new_v4(), command_type: CommandType::Test };
        
        orchestrator.queue_command(command);
        
        thread::sleep(Duration::from_secs(1))
    }
}
