use crate::{Action, ActionResult, Logger, Operation};

pub(crate) fn handle_action(action: Action, logger: Logger) -> ActionResult {
    let ops = vec![Operation::Test];

    logger.log_info("Action completed".to_string()).unwrap();

    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}