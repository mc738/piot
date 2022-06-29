pub mod network;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use crate::HttpResponse;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodeStateRequest {
    pub node: String,
    pub new_state: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodeStateResponse {
    pub result: String,
    pub old_state: u8,
    pub new_state: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeStateResponse {
    pub state: u8,
}

impl UpdateNodeStateRequest {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<UpdateNodeStateRequest> {
        let request: UpdateNodeStateRequest = serde_json::from_slice(&bytes)?;
        Ok(request)
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
}

impl UpdateNodeStateResponse {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<UpdateNodeStateResponse> {
        let response: UpdateNodeStateResponse = serde_json::from_slice(&bytes)?;
        Ok(response)
    }
    
    pub fn from_http_response(response: HttpResponse) ->  std::result::Result<UpdateNodeStateResponse, &'static str> {
        match response.body {
            None => {
                Err("No response body returned")
            }
            Some(body) => {
                match UpdateNodeStateResponse::from_bytes(body) {
                    Ok(response) => Ok(response),
                    Err(_) => Err("Unable to parse response")
                }
            }
        }
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
}

impl GetNodeStateResponse {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<GetNodeStateResponse> {
        let response: GetNodeStateResponse = serde_json::from_slice(&bytes)?;
        Ok(response)
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
}