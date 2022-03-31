// TODO: Maybe edit the tourney query to remove videogame
// Consider whether querying using requiredConnections for discord ID is good.
use crate::json::{construct_json, ContentType, new_content};
use crate::reqwest_client::ReqwestClient;

mod json;
mod reqwest_client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");

    let mut reqwest_client = ReqwestClient::new();

    construct_json(&mut reqwest_client, json::init_content());
    let mut result = reqwest_client.send_post();
    let mut json: json::PostResponse = result.json()?;
    let event_id = json.data.tournament.parse_event_id();

    let vars = (None, Some(event_id), None);
    let mut content = new_content(ContentType::EventContent, vars.clone());
    construct_json(&mut reqwest_client, content);
    result = reqwest_client.send_post();
    json = result.json()?;
    let player_map = match json.data.event {
        Some(event) => event.construct_player_map(&mut reqwest_client, event_id),
        None => panic!("Nothing!"),
    };

    content = new_content(ContentType::SetContent, vars);
    construct_json(&mut reqwest_client, content);
    result = reqwest_client.send_post();
    json = result.json()?;
    
    println!("JSON: {:?}", json);
    Ok(())
}