use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::collections::HashMap;
use smashgg_elo_rust::get_input;

const SMASH_URL: &str = "https://api.smash.gg/gql/alpha";
const AUTH_PROMPT: &str = "Enter your smash.gg authentication token: ";


/// A wrapper struct around a reqwest blocking Client. It contains the headers
/// and the json content needed to make a post request to smash.gg's api.
pub struct ReqwestClient<'a> {
    client: Client,
    pub json_content: HashMap<&'a str, Value>,
}

impl Default for ReqwestClient<'_> {
    /// Reads in user input to grab their smash.gg authentication token.
    /// Assigns the AUTHORIZATION header to Bearer [auth_token] and assigns the
    /// CONTENT_TYPE header so we're taking in json on our post request.
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        let mut auth_token: String = get_input(AUTH_PROMPT);
        auth_token = "Bearer ".to_owned() + &auth_token;

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_token).unwrap()
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        );

        ReqwestClient {
            client: reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Error in creating the reqwest client"),
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
        self
        .client
        .post(SMASH_URL)
        .json(&self.json_content)
        .send()
        .expect("Error in sending post request")
    }
}