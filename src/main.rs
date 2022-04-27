use crate::reqwest_wrapper::{ReqwestClient, Content, ContentType};
use crate::rusqlite_wrapper::{RusqliteConnection, PlayersRow};
use chrono::{TimeZone, Utc};

mod elo;
mod json;
mod reqwest_wrapper;
mod rusqlite_wrapper;

const PLAYERS: &str = "players";

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
    let (event_id, game_name, event_name) = json.get_event_info();

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

    // Create a table for the game rankings if needed.
    rusqlite_connection.create_table(&game_name);

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
            let dt = Utc.timestamp(set.time, 0);
            
            let set_struct = rusqlite_wrapper::SetsRow {
                player_one_global_id,
                player_one_name: player_one_name.to_string(),
                player_one_score: set.player_one_score,
                player_two_global_id,
                player_two_name: player_two_name.to_string(),
                player_two_score: set.player_two_score,
                tournament_name: event_name.clone(),
                game_name: game_name.clone(),
                set_time: dt.to_rfc3339(),
                ..rusqlite_wrapper::SetsRow::default()
            };

            // Detect DQ first. If detected, all we do is record it in the
            // set history.
            if set.player_one_score == -1 || set.player_two_score == -1 {
                rusqlite_connection.insert_set(set_struct);
            } else {
                let global_player_one = rusqlite_connection.select_player(player_one_global_id, &player_one_name, &PLAYERS.to_string())?;
                let global_player_two = rusqlite_connection.select_player(player_two_global_id, &player_two_name, &PLAYERS.to_string())?;
                let game_player_one = rusqlite_connection.select_player(player_one_global_id, &player_one_name, &game_name)?;
                let game_player_two = rusqlite_connection.select_player(player_two_global_id, &player_two_name, &game_name)?;


                let mut global_elo_calc = elo::Elo {
                    player_one: global_player_one,
                    player_one_score: set.player_one_score,
                    player_two: global_player_two,
                    player_two_score: set.player_two_score
                };
                let mut game_elo_calc = elo::Elo {
                    player_one: game_player_one,
                    player_one_score: set.player_one_score,
                    player_two: game_player_two,
                    player_two_score: set.player_two_score
                };

                let (global_player_one_elo_delta, global_player_two_elo_delta) = global_elo_calc.calc_elo();
                let (_game_player_one_elo_delta, _game_player_two_elo_delta) = game_elo_calc.calc_elo();

                let set_struct  = rusqlite_wrapper::SetsRow {
                    player_one_elo: global_elo_calc.player_one.player_elo,
                    player_one_elo_delta: global_player_one_elo_delta,
                    player_two_elo: global_elo_calc.player_two.player_elo,
                    player_two_elo_delta: global_player_two_elo_delta,
                    ..set_struct
                };

                rusqlite_connection.insert_set(set_struct);
                rusqlite_connection.update_player(global_elo_calc.player_one, &PLAYERS.to_string());
                rusqlite_connection.update_player(game_elo_calc.player_one, &game_name);
                rusqlite_connection.update_player(global_elo_calc.player_two, &PLAYERS.to_string());
                rusqlite_connection.update_player(game_elo_calc.player_two, &game_name);
            }
        }
    }

    // Update the rankings and increment the relevant counters.
    rusqlite_connection.update_ranking(&PLAYERS.to_string());
    rusqlite_connection.update_ranking(&game_name);
    rusqlite_connection.increment_count(&player_map, &PLAYERS.to_string());
    rusqlite_connection.increment_count(&player_map, &game_name);

    Ok(())
}