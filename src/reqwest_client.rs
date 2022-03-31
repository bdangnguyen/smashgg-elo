use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::HeaderMap;
use serde_json::Value;
use smashgg_elo_rust::construct_headers;
use std::collections::HashMap;

pub const SMASH_URL: &str = "https://api.smash.gg/gql/alpha";

/// A wrapper struct around a reqwest blocking Client. It contains the headers
/// and the json content needed to make a post request to smash.gg's api.
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
    /// Simple function that just calls the default constructor.
    pub fn new() -> Self {
        ReqwestClient::default()
    }

    /// Sends a HTTP post request using the header and json fields in the 
    /// struct and returns a reqwest response. This reqwest will later be
    /// parsed into json by other methods.
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