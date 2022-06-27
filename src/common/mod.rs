use std::marker::PhantomData;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use uuid::Uuid;
use crate::Log;
use crate::logger::Logger;

pub struct Event {
    pub(crate) id: Uuid,
    pub(crate) event_type: EventType
}

pub enum EventType {
    Test
}

pub struct Command {
    pub(crate) id: Uuid,
    pub(crate) command_type: CommandType,
}

pub enum CommandType {
    Test
}

pub struct Action {
    pub(crate) id: Uuid,
    pub(crate) action_type: ActionType,
}

pub enum ActionType {
    Test
}

pub struct ActionResult {
    pub(crate) id: Uuid,
    pub(crate) successful: bool,
    pub(crate) message: String,
    pub(crate) ops: Vec<Operation>,
}

pub enum Operation {
    Test
}