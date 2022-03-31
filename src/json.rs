use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use smashgg_elo_rust::get_input;

use crate::reqwest_client::ReqwestClient;

const SLUG_PROMPT: &str = "Enter the tournament slug to read data from: ";
const EVNT_PROMPT: &str = "Enter the id of one of the events to parse: ";

#[derive(Deserialize, Debug)] 
pub struct PostResponse { 
    pub data: Data
}
#[derive(Deserialize, Debug)] 
pub struct Data { 
    #[serde(default)]
    pub tournament: Tournament,
    pub event: Option<Event>
}
#[derive(Deserialize, Debug)] 
pub struct Tournament { 
    events: Vec<Events>, 
}

impl Default for Tournament {
    fn default() -> Self {
        Tournament { 
            events: Vec::new(),
         }
    }
}

impl Tournament {
    pub fn parse_event_id(self) -> i32 {
        loop {
            let mut count = 0;

            println!("List of events found in the tournament:");
            for event in &self.events {
                println!("{}: {:?}", count, event.name);
                count += 1;
            }

            let  event_input: i32 = get_input(EVNT_PROMPT);
            match event_input {
                i if i < 0 => continue,
                i if i > (self.events.len()-1).try_into().unwrap() => continue,
                _ =>  return self.events[event_input as usize].id
            };
        }
    }
}
#[derive(Deserialize, Debug)] 
struct Events {
    id: i32,
    name: String,
    videogame: Videogame
}
#[derive(Deserialize, Debug)] 
struct Videogame {
    name: String
}
#[derive(Deserialize, Debug)] 
pub struct Event {
    entrants: Entrants
}

impl Event {
    pub fn construct_player_map(self, reqwest_client: &mut ReqwestClient, event_id: i32) -> HashMap<i32, (String, i32)>{
        let player_map = HashMap::new();

        match self.entrants.pageInfo {
            Some(page_info) => for i in 0..page_info.totalPages {
                construct_json(reqwest_client, page_content(event_id, i));

                let result = reqwest_client.send_post();

                let json: PostResponse = match result.json() {
                    Ok(json) => json,
                    Err(err) => panic!("Error in converting to json {}", err),
                };
                println!("JSON: {:?}", json);
            },
            None => panic!("Error in matching page_info!"),
        }

        return player_map;
    }
}
#[derive(Deserialize, Debug)] struct Entrants {
    pageInfo: Option<Pageinfo>,
    nodes: Option<Vec<Nodes>>
}

#[derive(Deserialize, Debug)] struct Pageinfo {
    totalPages: i32
}
#[derive(Deserialize, Debug)] struct Nodes {
    id: i32,
    participants: Vec<Participants>
}
#[derive(Deserialize, Debug)]
struct Participants {
    gamerTag: String,
    user: User
}
#[derive(Deserialize, Debug)]
struct User {
    id: i32
}
#[derive(Serialize, Debug)] pub struct Content {
    query: &'static str,
    variables: Variables
}

#[derive(Serialize, Debug)] 
pub struct Variables {
    tournament_slug: Option<String>,
    event_id: Option<i32>,
    page: Option<i32>
}

pub fn construct_json(reqwest_client: &mut ReqwestClient, content: Content)  {
    reqwest_client.json_content.insert(
        "query",
        Value::from(content.query)
    );
    reqwest_client.json_content.insert(
        "variables",
        serde_json::json!(content.variables)
    );
}

fn construct_content(query: &'static str, variables: Variables) -> Content {
    Content {
        query,
        variables,
    }
}

fn construct_variables(tournament_slug: Option<String>, event_id: Option<i32>, page: Option<i32>) -> Variables {
    Variables { 
        tournament_slug,
        event_id,
        page
    }
}

pub fn init_content() -> Content {
    let tourney_slug: String = get_input(SLUG_PROMPT);
    let query = include_str!("query/tourney_event_query.graphql");
    let variables = construct_variables(Some(tourney_slug), None, None);

    construct_content(query, variables)
}



pub fn event_content(event_id: i32) -> Content {
    let query = include_str!("query/entrant_page_query.graphql");
    let variables = construct_variables(None, Some(event_id), None);

    construct_content(query, variables)
}

pub fn set_content(event_id: i32) -> Content {
    let query = include_str!("query/sets_page_query.graphql");
    let variables = construct_variables(None, Some(event_id), None);

    construct_content(query, variables)
}

pub fn page_content(event_id: i32, page: i32) -> Content {
    let query = include_str!("query/entrant_info_query.graphql");
    let variables = construct_variables(None, Some(event_id), Some(page));

    construct_content(query, variables)
}