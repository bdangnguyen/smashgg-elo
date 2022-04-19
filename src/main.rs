// TODO: Consider whether querying using requiredConnections for discord ID is good.
// Look into the leaky bucket algorithm for traffic shaping.
use crate::reqwest_wrapper::{ReqwestClient, Content, ContentType};
use crate::rusqlite_wrapper::{RusqliteConnection, PlayersRow};
use chrono::{TimeZone, Utc};

mod elo;
mod json;
mod reqwest_wrapper;
mod rusqlite_wrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smash.gg Elo Parser 1.0");

    // Init relevant objects
    let mut reqwest_client = ReqwestClient::new();
    let mut content = Content::new();
    let rusqlite_connection = RusqliteConnection::new();

    // Grab the id and name of the event we want to parse.
    content.edit_content(ContentType::InitContent);
    reqwest_client.construct_json(&content);
    let mut json: json::PostResponse = reqwest_client.send_post().json()?;
    let (event_id, event_name) = json.get_event_info();

    // Create a mapping of players that participated in that event.
    // The map is of the form key: tournament id, value: (name, global id).
    content.variables.event_id = Some(event_id);
    content.edit_content(ContentType::EventContent);
    reqwest_client.construct_json(&content);
    json = reqwest_client.send_post().json()?;
    let player_map = json.construct_player_map(&mut reqwest_client, event_id);

    // Grab the amount of times we need to make a request to parse all sets.
    content.edit_content(ContentType::SetContent);
    reqwest_client.construct_json(&content);
    json = reqwest_client.send_post().json()?;
    let num_pages = json.get_total_pages();

    for i in 0..num_pages {
        // Grab the paginated json for sets.
        content.variables.event_id = Some(event_id);
        content.variables.page = Some(i);
        content.edit_content(ContentType::InfoContent);
        reqwest_client.construct_json(&content);
        json = reqwest_client.send_post().json()?;

        // p1 tourney id, p1 score, p2 tourney id, p2 score, time
        for set in json.get_sets_info() {
            let player_one_name = &player_map[&set.player_one_id].0;
            let player_one_global_id = player_map[&set.player_one_id].1;
            let player_two_name = &player_map[&set.player_two_id].0;
            let player_two_global_id = player_map[&set.player_two_id].1;

            let player_one = rusqlite_connection.select_player(player_one_global_id, &player_one_name)?;
            let player_two = rusqlite_connection.select_player(player_two_global_id, &player_two_name)?;

            let mut elo_calc = elo::Elo {
                player_one,
                player_one_score: set.player_one_score,
                player_two,
                player_two_score: set.player_two_score
            };

            let (player_one_elo_delta, player_two_elo_delta) = elo_calc.calc_elo();
            let dt = Utc.timestamp(set.time, 0);

            rusqlite_connection.insert_match(
                rusqlite_wrapper::SetsRow {
                    player_one_global_id,
                    player_one_name: player_one_name.to_string(),
                    player_one_elo: elo_calc.player_one.player_elo,
                    player_one_score: set.player_one_score,
                    player_one_elo_delta,
                    player_two_global_id,
                    player_two_name: player_two_name.to_string(),
                    player_two_elo: elo_calc.player_two.player_elo,
                    player_two_score: set.player_two_score,
                    player_two_elo_delta,
                    tournament_name: event_name.clone(),
                    set_time: dt.to_rfc3339(),
                }
            );

            // Detect if DQ'd. If so, don't update.
            if set.player_one_score != -1 && set.player_two_score != -1 {
                rusqlite_connection.update_player(elo_calc.player_one);
                rusqlite_connection.update_player(elo_calc.player_two);
            }
        }

    }
    
    println!("JSON: {:?}", num_pages);

    Ok(())
}