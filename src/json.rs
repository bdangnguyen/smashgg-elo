use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use smashgg_elo_rust::get_input;

use crate::reqwest_client::ReqwestClient;

const SLUG_PROMPT: &str = "Enter the tournament slug to read data from: ";
const EVNT_PROMPT: &str = "Enter the id of one of the events to parse: ";

pub enum ContentType {
    InitContent,
    EventContent,
    SetContent,
    PageContent,
}

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
        let mut player_map = HashMap::new();

        match self.entrants.page_info {
            Some(page_info) => for i in 0..page_info.total_pages {
                let vars = (None, Some(event_id), Some(i));
                let page_content = new_content(ContentType::PageContent, vars);
                construct_json(reqwest_client, page_content);

                let result = reqwest_client.send_post();

                let json: PostResponse = match result.json() {
                    Ok(json) => json,
                    Err(err) => panic!("Error in converting to json {}", err),
                };

                let nodes = match json.data.event {
                    Some(event) => event.entrants.nodes,
                    None => None,
                };

                match nodes {
                    Some(node_vec) => for player in node_vec {
                        player_map.insert(
                            player.id,
                            (player.participants[0].gamer_tag.to_owned(), player.participants[0].user.id),
                        );
                    }
                    None => (),
                }
            },
            None => panic!("Error in matching page_info!"),
        }

        println!("Map: {:?}", player_map);

        return player_map;
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct Entrants {
    page_info: Option<Pageinfo>,
    nodes: Option<Vec<Nodes>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct Pageinfo {
    total_pages: i32
}
#[derive(Deserialize, Debug)] struct Nodes {
    id: i32,
    participants: Vec<Participants>
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Participants {
    gamer_tag: String,
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

pub fn init_content() -> Content {
    let tourney_slug: String = get_input(SLUG_PROMPT);
    let vars = (Some(tourney_slug), None, None);
    new_content(ContentType::InitContent, vars)
}

pub fn new_content(enum_type: ContentType, vars: (Option<String>, Option<i32>, Option<i32>)) -> Content {
    let query = match enum_type {
        ContentType::InitContent => include_str!("query/tourney_event_query.graphql"),
        ContentType::EventContent => include_str!("query/entrant_page_query.graphql"),
        ContentType::SetContent => include_str!("query/sets_page_query.graphql"),
        ContentType::PageContent => include_str!("query/entrant_info_query.graphql"),
    };

    let variables = Variables {
        tournament_slug: vars.0,
        event_id: vars.1,
        page: vars.2
    };

    let content = Content {
        query,
        variables
    };

    return content;
}