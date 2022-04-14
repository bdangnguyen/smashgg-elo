use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use smashgg_elo_rust::get_input;

const SMASH_URL: &str = "https://api.smash.gg/gql/alpha";
const AUTH_PROMPT: &str = "Enter your smash.gg authentication token: ";
const SLUG_PROMPT: &str = "Enter the tournament slug to read data from: ";
const MAX_ENTRANTS: i32 = 499;
const MAX_SETS: i32 = 70;

pub enum ContentType {
    InitContent,
    EventContent,
    SetContent,
    InfoContent,
    PageContent,
}


// A wrapper struct around a reqwest blocking Client. It contains the headers
// and the json content needed to make a post request to smash.gg's api.
pub struct ReqwestClient<'a> {
    pub client: Client,
    pub json_content: HashMap<&'a str, Value>,
}

impl Default for ReqwestClient<'_> {
    // Reads in user input to grab their smash.gg authentication token.
    // Assigns the AUTHORIZATION header to Bearer [auth_token] and assigns the
    // CONTENT_TYPE header so we're taking in json on our post request.
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
    pub fn new() -> Self {
        ReqwestClient::default()
    }

    // Sends a HTTP post request using the header and json fields in the 
    // struct and returns a reqwest response. This reqwest will later be
    // parsed into json by other methods.
    pub fn send_post(&self) -> Response {
        self
        .client
        .post(SMASH_URL)
        .json(&self.json_content)
        .send()
        .expect("Error in sending post request")
    }

    pub fn construct_json(&mut self, content: &Content)  {
        self.json_content.insert(
            "query",
            Value::from(content.query)
        );
        self.json_content.insert(
            "variables",
            serde_json::json!(content.variables)
        );
    }
}

#[derive(Serialize)] 
pub struct Content {
    pub query: &'static str,
    pub variables: Variables
}

impl Default for Content {
    fn default() -> Self {
        Content {
            query: "",
            variables: Variables::new()
        }
    }
}

impl Content {
    pub fn new() -> Self {
        Content::default()
    }

    pub fn edit_content(&mut self, enum_type: ContentType) {
        let (query, per_page) = match enum_type {
            ContentType::InitContent => {
                self.variables.tournament_slug = Some(get_input(SLUG_PROMPT));
                (include_str!("query/tourney_event_query.graphql"),
                None)
            }
            ContentType::EventContent => {
                (include_str!("query/entrant_page_query.graphql"),
                Some(MAX_ENTRANTS))
            }
            ContentType::SetContent => {
                (include_str!("query/sets_page_query.graphql"),
                Some(MAX_SETS))
            }
            ContentType::InfoContent => {
                (include_str!("query/sets_info_query.graphql"),
                Some(MAX_SETS))
            }
            ContentType::PageContent => {
                (include_str!("query/entrant_info_query.graphql"),
                Some(MAX_ENTRANTS))
            }
        };

        self.variables.per_page = per_page;
        self.query = query;
    }
}

#[derive(Serialize)] 
pub struct Variables {
    pub tournament_slug: Option<String>,
    pub event_id: Option<i32>,
    pub page: Option<i32>,
    pub per_page: Option<i32>
}

impl Default for Variables {
    fn default() -> Self {
        Variables {
            tournament_slug: None,
            event_id: None,
            page: None,
            per_page: None
        }
    }
}

impl Variables {
    pub fn new() -> Self {
        Variables::default()
    }
}