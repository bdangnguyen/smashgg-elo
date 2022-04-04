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
    data: Data
}

impl PostResponse {
    pub fn get_event_id(self) -> i32 {
        loop {
            let mut count = 0;
            let tournament = self.data.tournament();

            println!("List of events found in the tournament:");
            for event in &tournament.events {
                println!("{}: {:?}", count, event.name);
                count += 1;
            }

            let event_input: i32 = get_input(EVNT_PROMPT);
            let num_events = tournament.events.len() - 1;
            match event_input {
                i if i < 0 => continue,
                i if i > (num_events).try_into().unwrap() => continue,
                _ =>  return tournament.events[event_input as usize].id
            };
        }
    }

    pub fn get_total_pages(self) -> i32 {
        self.data.event().sets().page_info.total_pages
    }

    pub fn construct_player_map(self, reqwest_client: &mut ReqwestClient, event_id: i32) -> HashMap<i32, (String, i32)>{
        let mut player_map = HashMap::new();

        let page_info = self.data.event().entrants().page_info();

        for i in 0.. page_info.total_pages {
            let vars = (None, Some(event_id), Some(i));
            let page_content = new_content(ContentType::PageContent, vars);
            construct_json(reqwest_client, page_content);

            let result = reqwest_client.send_post();

            let json: PostResponse = match result.json() {
                Ok(json) => json,
                Err(err) => panic!("Error in converting to json {}", err),
            };

            let nodes = json.data.event().entrants().nodes();

            for player in nodes {
                player_map.insert(
                    player.id,
                    (player.participants[0].gamer_tag.to_owned(), player.participants[0].user.id),
                );
            }
        }

        //println!("Map: {:?}", player_map);

        return player_map;
    }

}
#[derive(Deserialize, Debug)] 
struct Data { 
    tournament: Option<Tournament>,
    event: Option<Event>
}

impl Data {
    fn tournament(&self) -> &Tournament {
        self.tournament.as_ref().expect("Matching error: No tournament found")

    }
    
    fn event(self) -> Event {
        self.event.expect("Matching error: No event found")
    }
}

#[derive(Deserialize, Debug)] 
struct Tournament { 
    events: Vec<Events>, 
}

#[derive(Deserialize, Debug)] 
struct Events {
    id: i32,
    name: String,
}
#[derive(Deserialize, Debug)] 
struct Event {
    entrants: Option<Entrants>,
    sets: Option<Sets>
}

impl Event {
    fn entrants(self) -> Entrants {
        self.entrants.expect("Matching error: No entrants found")
    }

    fn sets(self) -> Sets {
        self.sets.expect("Matching error: No sets found")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct Entrants {
    page_info: Option<PageInfo>,
    nodes: Option<Vec<Nodes>>
}

impl Entrants {
    fn page_info(self) -> PageInfo {
        self.page_info.expect("Matching error: No page info found")
    }

    fn nodes(self) -> Vec<Nodes> {
        self.nodes.expect("Matching error: No nodes found")
    }
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Sets {
    page_info: PageInfo
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
 struct PageInfo {
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
#[derive(Serialize, Debug)] 
pub struct Content {
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