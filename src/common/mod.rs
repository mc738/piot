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
    Test,
    RunResult(RunResultEvent),
    NodeStateChange(NodeStateChangeEvent),
}

pub struct RunResultEvent {
    pub(crate) successful: bool,
    pub(crate) message: String,
}

pub struct NodeStateChangeEvent {
    pub node: String,
    pub old_state: u8,
    pub new_state: u8,
} 

pub struct Command {
    pub(crate) id: Uuid,
    pub(crate) command_type: CommandType,
}

pub enum CommandType {
    Test,
    Run(RunCommand),
    ChangeNodeState(ChangeNodeStateCommand)
}

pub struct RunCommand {
    pub(crate) message: String
}

pub struct ChangeNodeStateCommand {
    pub node: String,
    pub(crate) new_state: u8
}

pub struct Action {
    pub(crate) id: Uuid,
    pub(crate) action_type: ActionType,
}

pub enum ActionType {
    Test,
    Run(RunAction),
    ChangeNodeState(ChangeNodeStateAction)
}

pub struct RunAction {
    pub(crate) message: String
}

pub struct ChangeNodeStateAction {
    pub node: String,
    pub(crate) new_state: u8
}

pub struct ActionResult {
    pub(crate) id: Uuid,
    pub(crate) successful: bool,
    pub(crate) message: String,
    pub(crate) ops: Vec<Operation>,
}

pub enum Operation {
    Test,
    RaiseEvent(Event),
}