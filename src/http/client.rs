use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;
use crate::http::common::{HttpRequest, HttpResponse, HttpVerb};
use crate::{Log, Logger};

pub struct HttpClient {
    address: String,
    //stream: TcpStream,
}

impl HttpClient {
    pub fn create(address: String) -> HttpClient {
        HttpClient { address }
    }
    
    /*
    pub fn connect(address: String) -> Result<HttpClient, &'static str> {
        match TcpStream::connect(address) {
            Ok(stream) => {
                Ok(HttpClient { stream })
            }
            Err(_) => {
                Err("Could not connect to server.")
            }
        }
    }*/

    pub fn get(&mut self, route: String, content_type: String, addition_header: HashMap<String, String>) -> Result<HttpResponse, &'static str> {
        match TcpStream::connect(self.address.clone()) {
            Ok(mut stream) => {
                let mut request = HttpRequest::create(route, HttpVerb::GET, content_type, addition_header, None);

                match stream.write(&*request.to_bytes()) {
                    Ok(_) => {
                        HttpResponse::from_stream(&stream)
                    }
                    Err(_) => Err("Could not connect to server, GET request failed.")
                }
            }
            Err(_) => {
                Err("Could not connect to server.")
            }
        }
    }
}