use uuid::Uuid;
use crate::{Action, ActionResult, ActionType, Event, EventType, Logger, Operation};
use crate::common::RunResultEvent;

pub(crate) fn handle_action(action: Action, logger: Logger) -> ActionResult {
    //let ops = vec![];

    let mut ops: Vec<Operation> = Vec::new();

    logger.log_info("Action completed".to_string()).unwrap();

    match action.action_type {
        ActionType::Test => {
            ops.push(Operation::Test)
        }
        ActionType::Run(run) => {
            ops.push(Operation::RaiseEvent(Event { id: Uuid::new_v4(), event_type: EventType::RunResult(RunResultEvent { successful: true, message: run.message }) }))
        }
    }

    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}