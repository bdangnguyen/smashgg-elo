use std::collections::HashMap;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use smashgg_elo_rust::get_input;


#[derive(Deserialize, Debug)] pub struct PostResponse { 
    pub data: Data
}
#[derive(Deserialize, Debug)] pub struct Data { 
    #[serde(default)]
    pub tournament: Tournament,
    pub event: Option<Event>
}
#[derive(Deserialize, Debug)] pub struct Tournament { 
    events: Vec<EventStruct> 
}

impl Default for Tournament {
    fn default() -> Self {
        Tournament { events: Vec::new() }
    }
}

impl Tournament {
    pub fn get_event_id(self) -> i32 {
        loop {
            let mut count = 0;

            println!("List of events found in the tournament:");
            for event in &self.events {
                println!("{}: {:?}", count, event.name);
                count += 1;
            }

            let  event_input: i32= get_input(smashgg_elo_rust::EVNT_PROMPT);
            match event_input {
                i if i < 0 => continue,
                i if i > (self.events.len()-1).try_into().unwrap() => continue,
                _ =>  return self.events[event_input as usize].id
            };
        }
    }
}
#[derive(Deserialize, Debug)] struct EventStruct {
    id: i32,
    name: String,
    videogame: Videogame
}
#[derive(Deserialize, Debug)] struct Videogame {
    name: String
}
#[derive(Deserialize, Debug)] pub struct Event {
    entrants: Entrants
}

impl Event {
    pub fn construct_player_map2(self, headers: HeaderMap, event_id: i32) -> Result<(), Box<reqwest::Error>>{
        let client = reqwest::blocking::Client::new();
        let mut json_content: HashMap<&str, Value> = HashMap::new();

        for i in 0..self.entrants.pageInfo.totalPages {
            build_player_json(&mut json_content, event_id, i);

            let mut result = client.post(smashgg_elo_rust::SMASH_URL)
                .headers(headers.clone())
                .json(&json_content)
                .send()?;

            //println!("This is the result: {:?}", result.text());

            let json: crate::EntrantInfoResponse = result.json()?;
            println!("JSON: {:?}", json);
        }
    
        Ok(())
    }
}
#[derive(Deserialize, Debug)] struct Entrants {
    pageInfo: Pageinfo,
   // nodes: Option<Nodes>
}

#[derive(Deserialize, Debug)] struct Pageinfo {
    totalPages: i32
}
#[derive(Serialize, Debug)] pub struct Content {
    query: String,
    variables: Variables
}

#[derive(Serialize, Debug)] pub struct Variables {
    tournament_slug: Option<String>,
    event_id: Option<i32>
}

fn build_player_json(json_content: &mut HashMap<&str, Value>, event_id: i32, page: i32) {
    json_content.insert(
        "query",
        Value::from(include_str!("query/entrant_info_query.graphql"))
    );

    json_content.insert(
        "variables",
        serde_json::json!({"eventId": event_id, "page": page})
    );
}

pub fn construct_json_content(json_content: &mut HashMap<&'static str, Value>, content: Content)  {
    json_content.insert(
        "query",
        Value::from((content.query))
    );
    json_content.insert(
        "variables",
        serde_json::json!(content.variables)
    );
}

fn construct_content(query: String, variables: Variables) -> Content {
    Content {
        query: query,
        variables: variables,
    }
}

fn construct_variables(tournament_slug: Option<String>, event_id: Option<i32>) -> Variables {
    Variables { 
        tournament_slug: tournament_slug,
        event_id: event_id,
    }
}

pub fn init_content() -> Content {
    let tourney_slug: String = get_input(smashgg_elo_rust::SLUG_PROMPT);
    let query = include_str!("query/tourney_event_query.graphql").to_string();
    let variables = construct_variables(Some(tourney_slug), None);

    construct_content(query, variables)
}

pub fn event_content(event_id: i32) -> Content {
    let query = include_str!("query/entrant_page_query.graphql").to_string();
    let variables = construct_variables(None, Some(event_id));

    construct_content(query, variables)
}