use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use serde_json::Value;
use smashgg_elo_rust::construct_headers;
use std::collections::HashMap;

pub const SMASH_URL: &str = "https://api.smash.gg/gql/alpha";

pub struct ReqwestClient<'a> {
    client: Client,
    headers: HeaderMap,
    pub json_content: HashMap<&'a str, Value>,
}

impl Default for ReqwestClient<'_> {
    fn default() -> Self {
        ReqwestClient {
            client: reqwest::blocking::Client::new(),
            headers: construct_headers(),
            json_content: HashMap::new(),
        }
    }
}

impl ReqwestClient<'_> {
    pub fn new() -> Self {
        ReqwestClient::default()
    }

    pub fn send_post(&self) -> Response {
        let result = match self
            .client
            .post(SMASH_URL)
            .headers(self.headers.clone())
            .json(&self.json_content)
            .send()
        {
            Ok(response) => response,
            Err(err) => panic!("Error in sending post request: {}", err),
        };

        return result;
    }
}