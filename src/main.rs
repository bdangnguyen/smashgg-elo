// TODO: Maybe edit the tourney query to remove videogame
// Consider whether querying using requiredConnections for discord ID is good.
// Look into the leaky bucket algorithm for traffic shaping.
use crate::json::{construct_json, ContentType, new_content};
use crate::reqwest_client::ReqwestClient;
//use rusqlite::Connection;

mod json;
mod reqwest_client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");
    //let path = "./database/smashhgg.db3";
    //let conn = Connection::open(path)?;
    //println!("Testing: {}", conn.is_autocommit());

    let mut reqwest_client = ReqwestClient::new();

    construct_json(&mut reqwest_client, json::init_content());
    let mut json: json::PostResponse = reqwest_client.send_post().json()?;
    let event_id = json.get_event_id();

    let vars = (None, Some(event_id), None);
    let mut content = new_content(ContentType::EventContent, vars.clone());
    construct_json(&mut reqwest_client, content);
    json = reqwest_client.send_post().json()?;
    let player_map = json.construct_player_map(&mut reqwest_client, event_id);

    // TODO: Redo the following code. This is just for testing.
    content = new_content(ContentType::SetContent, vars);
    construct_json(&mut reqwest_client, content);
    json = reqwest_client.send_post().json()?;

    let num_set_pages = json.get_total_pages();

    // event id page
    for i in 0..num_set_pages {
        let vars = (None, Some(event_id), Some(i));
        let content = new_content(ContentType::InfoContent, vars.clone());
        construct_json(&mut reqwest_client, content);
        println!("TEST: {:?}", reqwest_client.send_post().text());
    }
    
    println!("JSON: {:?}", num_set_pages);
    Ok(())
}