// TODO: Consider whether querying using requiredConnections for discord ID is good.
// Look into the leaky bucket algorithm for traffic shaping.
use crate::json::{construct_json, ContentType, new_content};
use crate::reqwest_wrapper::ReqwestClient;
use crate::rusqlite_wrapper::{RusqliteConnection, SetsRow, PlayersRow};

mod elo;
mod json;
mod reqwest_wrapper;
mod rusqlite_wrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");

    let mut reqwest_client = ReqwestClient::new();
    let rusqlite_connection = RusqliteConnection::new();

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
        let json: json::PostResponse = reqwest_client.send_post().json()?;
        println!("TEST: {:?}", json);

        for set in json.get_sets_info() {
            let player_one_name = &player_map[&set.0].0;
            let player_one_global_id = player_map[&set.0].1;
            let player_two_name = &player_map[&set.2].0;
            let player_two_global_id = player_map[&set.2].1;

            let player_one = rusqlite_connection.select_player(player_one_global_id, player_one_name)?;
            let player_two = rusqlite_connection.select_player(player_two_global_id, player_two_name)?;

            /*let set = rusqlite_wrapper::SetsRow {
                player_one_global_id,
                player_one_name: player_one_name.to_string(),
                player_one_elo: player_one.player_elo,
                player_one_score: set.1,
                player_one_elo_delta: todo!(),
                player_two_global_id,
                player_two_name: player_two_name.to_string(),
                player_two_elo: player_two.player_elo,
                player_two_score: set.3,
                player_two_elo_delta: todo!(),
                tournament_name: todo!(),
                set_time: todo!(),
            };*/
        }

    }
    
    println!("JSON: {:?}", num_set_pages);

    Ok(())
}