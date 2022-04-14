// TODO: Consider whether querying using requiredConnections for discord ID is good.
// Look into the leaky bucket algorithm for traffic shaping.
use crate::reqwest_wrapper::{ReqwestClient, Content, ContentType};
use crate::rusqlite_wrapper::{RusqliteConnection, PlayersRow};
use chrono::{DateTime, TimeZone, Utc};


mod elo;
mod json;
mod reqwest_wrapper;
mod rusqlite_wrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");

    let mut reqwest_client = ReqwestClient::new();
    let mut content = Content::new();
    let rusqlite_connection = RusqliteConnection::new();

    content.edit_content(ContentType::InitContent);
    reqwest_client.construct_json(&content);
    let mut json: json::PostResponse = reqwest_client.send_post().json()?;
    let (event_id, event_name) = json.get_event_info();

    content.variables.event_id = Some(event_id);
    content.edit_content(ContentType::EventContent);
    reqwest_client.construct_json(&content);
    json = reqwest_client.send_post().json()?;
    let player_map = json.construct_player_map(&mut reqwest_client, event_id);

    // TODO: Redo the following code. This is just for testing.
    content.edit_content(ContentType::SetContent);
    reqwest_client.construct_json(&content);
    json = reqwest_client.send_post().json()?;

    let num_pages = json.get_total_pages();

    // event id page
    for i in 0..num_pages {
        content.variables.event_id = Some(event_id);
        content.variables.page = Some(i);
        content.edit_content(ContentType::InfoContent);
        reqwest_client.construct_json(&content);
        let json: json::PostResponse = reqwest_client.send_post().json()?;

        // p1 tourney id, p1 score, p2 tourney id, p2 score, time
        for set in json.get_sets_info() {
            let player_one_name = &player_map[&set.0].0;
            let player_one_global_id = player_map[&set.0].1;
            let player_two_name = &player_map[&set.2].0;
            let player_two_global_id = player_map[&set.2].1;

            let player_one = rusqlite_connection.select_player(player_one_global_id, player_one_name)?;
            let player_two = rusqlite_connection.select_player(player_two_global_id, player_two_name)?;

            let elo_calc = elo::Elo {
                player_one,
                player_two
            };

            let (new_elo_one, new_elo_two) = elo_calc.calculate_elo(set.1, set.3);
            let dt = Utc.timestamp(set.4, 0);

            // Check 
            let set_info = rusqlite_wrapper::SetsRow {
                player_one_global_id,
                player_one_name: player_one_name.to_string(),
                player_one_elo: elo_calc.player_one.player_elo,
                player_one_score: set.1,
                player_one_elo_delta: new_elo_one - elo_calc.player_one.player_elo,
                player_two_global_id,
                player_two_name: player_two_name.to_string(),
                player_two_elo: elo_calc.player_two.player_elo,
                player_two_score: set.3,
                player_two_elo_delta: new_elo_two - elo_calc.player_two.player_elo,
                tournament_name: event_name.clone(),
                set_time: dt.to_rfc3339(),
            };
        }

    }
    
    println!("JSON: {:?}", num_pages);

    Ok(())
}