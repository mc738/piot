use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;
use crate::http::common::{HttpRequest, HttpResponse, HttpVerb};
use crate::{Log, Logger};

pub struct HttpClient {
    stream: TcpStream,
}


impl HttpClient {
    pub fn connect(address: String) -> Result<HttpClient, &'static str> {
        match TcpStream::connect(address) {
            Ok(stream) => {
                Ok(HttpClient { stream })
            }
            Err(_) => {
                Err("Could not connect to server.")
            }
        }
    }

    pub fn get(&mut self, route: String, content_type: String, addition_header: HashMap<String, String>) -> Result<HttpResponse, &'static str> {
        let mut request = HttpRequest::create(route, HttpVerb::GET, content_type, addition_header, None);

        match self.stream.write(&*request.to_bytes()) {
            Ok(_) => {
                HttpResponse::from_stream(&self.stream)
            }
            Err(_) => Err("Could not connect to server, GET request failed.")
        }
    }
}