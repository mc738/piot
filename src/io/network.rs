use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub struct NameResolver {
    thread: JoinHandle<()>
}

pub enum ResolverMessage {
    AddAddress((String, String)),
    GetAddress(NameRequest),
}

pub struct NameRequest {
    pub(crate) name: String,
    pub(crate) reply_channel: Sender<Option<String>>
}


impl NameResolver {
    
    pub fn start(mut map: HashMap<String, String>, receiver: Receiver<ResolverMessage>) -> NameResolver {
        
        let thread =
            thread::spawn(move || loop {
               let message = receiver.recv().unwrap();
                
                match message {
                    ResolverMessage::AddAddress((k, v)) => {
                        map.insert(k, v);
                    }
                    ResolverMessage::GetAddress(request) => {
                        match map.get(request.name.as_str()) {
                            None => {
                                request.reply_channel.send(None).unwrap();
                            }
                            Some(addr) => {
                                let r = addr.clone();
                                request.reply_channel.send(Some(r)).unwrap();
                            }
                        }
                    }
                }
            });
        
        NameResolver {
            thread
        }
    }
    
}