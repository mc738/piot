use crate::common::Event;
use crate::{Command, CommandType, EventType, Logger};

pub fn handle_event(event: Event, logger: &Logger) -> Vec<Command> {
    logger.log_info(format!("Handling event {}", event.id)).unwrap();
    match event.event_type {
        EventType::Test => {
            vec! [ Command { id: event.id, command_type: CommandType::Test } ]
        }
    }
}