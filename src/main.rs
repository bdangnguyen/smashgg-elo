// TODO: Maybe edit the tourney query to remove videogame
// Default vs option for Events
// Consider whether querying using requiredConnections for discord ID is good.

use std::collections::HashMap;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::json::construct_json_content;
use crate::reqwest_client::ReqwestClient;

mod json;
mod reqwest_client;


#[derive(Deserialize, Debug)] struct EntrantInfoResponse {
    data: EntrantData
}
#[derive(Deserialize, Debug)] struct EntrantData {
    event: EntrantEvent
}
#[derive(Deserialize, Debug)] struct EntrantEvent {
    entrants: EntrantEntrants
}
#[derive(Deserialize, Debug)] struct EntrantEntrants {
    nodes: Vec<NodeStruct>
}
#[derive(Deserialize,Debug)] struct NodeStruct {
    id: i32,
    participants: Vec<ParticipantStruct>
}
#[derive(Deserialize, Debug)] struct ParticipantStruct {
    gamerTag: String,
    user: UserStruct
}
#[derive(Deserialize, Debug)] struct UserStruct {
    id: i32
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");

    let headers = smashgg_elo_rust::construct_headers();
    let mut json_content = HashMap::new();
    let reqwest_client = ReqwestClient::new();
    let client = reqwest::blocking::Client::new();

    // Does not handle errors yet
    construct_json_content(&mut json_content, json::init_content());
    let mut result = reqwest_client.send_post(headers.clone(), &json_content);
    let json: json::PostResponse = result.json()?;
    let event_id = json.data.tournament.parse_event_id();

    construct_json_content(&mut json_content, json::event_content(event_id));
    result = client.post(smashgg_elo_rust::SMASH_URL)
        .headers(headers.clone())
        .json(&json_content)
        .send()?;
    let json: json::PostResponse = result.json()?;
    let player_map = match json.data.event {
        Some(event) => event.construct_player_map2(headers.clone(), event_id),
        None => panic!("Nothing!"),
    };

    construct_json_content(&mut json_content, json::set_content(event_id));
    result = client.post(smashgg_elo_rust::SMASH_URL)
        .headers(headers.clone())
        .json(&json_content)
        .send()?;
    
    println!("{:?}", result.text());
    Ok(())
}