use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, SendError};
use uuid::Uuid;
use crate::{Action, ActionResult, ActionType, Event, EventType, HttpClient, HttpResponse, Log, Logger, NameResolver, Operation, ResolverMessage};
use crate::common::{NodeStateChangeEvent, RunResultEvent};
use crate::io::network::NameRequest;
use crate::io::UpdateNodeStateResponse;

pub(crate) fn handle_action(action: Action, name_resolver: Sender<ResolverMessage>, logger: Logger) -> ActionResult {
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
        ActionType::ChangeNodeState(new_state) => {
            let (rc, rx) = channel();
            name_resolver.send(ResolverMessage::GetAddress(NameRequest { name: new_state.node.clone(), reply_channel: rc })).unwrap();
            
            match rx.recv().unwrap() {
                None => {
                    logger.log_warning("Could not resolve name".to_string()).unwrap();
                }
                Some(addr) => {
                    let mut client = HttpClient::create(addr);
                    match client.get(format!("/set-state/{}", new_state.new_state), "text/plain".to_string(), HashMap::new()) {
                        Ok(response) => {
                            match UpdateNodeStateResponse::from_http_response(response) {
                                Ok(update_response) => if update_response.result == "updated" {
                                    logger.log_success("Node state updated".to_string()).unwrap();
                                    ops.push(Operation::RaiseEvent(Event { id: Uuid::new_v4(), event_type: EventType::NodeStateChange(NodeStateChangeEvent { node: new_state.node, old_state: update_response.old_state, new_state: update_response.new_state }) }))
                                }
                                else {
                                    logger.log_info(format!("Node state not updated. Requested state same as current state")).unwrap();
                                },
                                Err(e) => {
                                    logger.log_error(format!("Failed to update node state. Error - {}", e)).unwrap();
                                }
                            }
                        }
                        Err(e) => {
                            logger.log_error(format!("Failed to connect to node. Error - {}", e)).unwrap();
                        }
                    }
                }
            }
            
            
        }
    }

    ActionResult { id: action.id, successful: true, message: "Hello, World!".to_string(), ops }
}