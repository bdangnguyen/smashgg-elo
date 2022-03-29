
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use serde_json::Value;

pub struct ReqwestClient {
    client: Client
}

impl Default for ReqwestClient {
    fn default() -> Self {
        ReqwestClient { 
            client: reqwest::blocking::Client::new(),
         }
    }
}

impl ReqwestClient {
    pub fn new() -> Self {
        ReqwestClient::default()
    }

    pub fn send_post(self, headers: HeaderMap, json_content: &HashMap<&str, Value>) -> Response {
        let result = match self.client.post(smashgg_elo_rust::SMASH_URL)
        .headers(headers.clone())
        .json(&json_content)
        .send() {
            Ok(response) => response,
            Err(err) => panic!("Error in sending post request: {}", err),
        };

        return result;
    }
}