use crate::{ActionResult, Logger};
use crate::common::Event;

pub fn handle_result(result: ActionResult, logger: &Logger) -> Vec<Event> {
    match result.successful {
        true => logger.log_success(format!("Action {} success. Message - {}", result.id, result.message)).unwrap(),
        false => logger.log_error(format!("Action {} failed. Message - {}", result.id, result.message)).unwrap(),
    };
    
    vec![]
}