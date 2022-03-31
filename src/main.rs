// TODO: Maybe edit the tourney query to remove videogame
// Consider whether querying using requiredConnections for discord ID is good.
use crate::json::construct_json;
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

    construct_json(&mut reqwest_client, json::event_content(event_id));
    result = reqwest_client.send_post();
    json = result.json()?;
     let player_map = match json.data.event {
        Some(event) => event.construct_player_map(&mut reqwest_client, event_id),
        None => panic!("Nothing!"),
    };

    construct_json(&mut reqwest_client, json::set_content(event_id));
    result = reqwest_client.send_post();
    
    println!("{:?}", result.text());
    Ok(())
}