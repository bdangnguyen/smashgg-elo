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
    let players = json.construct_players(&mut reqwest_client, event_id);

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
        let set_list = json.get_sets_info();
        let mut count = 1;

        // p1 tourney id, p1 score, p2 tourney id, p2 score, time
        for set in &set_list {
            let player_one_name = &players[&set.player_one_id].0;
            let player_one_global_id = players[&set.player_one_id].1;
            let player_two_name = &players[&set.player_two_id].0;
            let player_two_global_id = players[&set.player_two_id].1;
            let dt = Utc.timestamp(set.time, 0);
            
            let mut set_struct = rusqlite_wrapper::SetsRow {
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
                // Select both players from the global players table and the
                // game table in the sqlite database.
                let global_player_one = rusqlite_connection.select_player(
                    player_one_global_id,
                    &player_one_name,
                    &PLAYERS.to_string()
                )?;
                let global_player_two = rusqlite_connection.select_player(
                    player_two_global_id,
                    &player_two_name,
                    &PLAYERS.to_string()
                )?;
                let game_player_one = rusqlite_connection.select_player(
                    player_one_global_id,
                    &player_one_name,
                    &game_name
                )?;
                let game_player_two = rusqlite_connection.select_player(
                    player_two_global_id,
                    &player_two_name,
                    &game_name
                )?;


                let mut global_elo = elo::Elo {
                    player_one: global_player_one,
                    score_one: set.player_one_score,
                    player_two: global_player_two,
                    score_two: set.player_two_score
                };
                let mut game_elo = elo::Elo {
                    player_one: game_player_one,
                    score_one: set.player_one_score,
                    player_two: game_player_two,
                    score_two: set.player_two_score
                };

                // Record the elo before the change
                set_struct.player_one_elo = global_elo.player_one.elo;
                set_struct.player_two_elo = global_elo.player_two.elo;

                // Calculate elo for both players in the global table.
                let (delta_one, delta_two) = global_elo.calc_elo();
                let (_unused_one, _unused_two) = game_elo.calc_elo();

                // Record the change in elo.
                set_struct.player_one_elo_delta = delta_one;
                set_struct.player_two_elo_delta = delta_two;

                // Record the set. Update any changes in the player's stats
                // in both the global and game table.
                rusqlite_connection.insert_set(set_struct);
                rusqlite_connection.update_player(
                    &global_elo.player_one,
                    &PLAYERS.to_string()
                );
                rusqlite_connection.update_player(
                    &game_elo.player_one,
                    &game_name
                );
                rusqlite_connection.update_player(
                    &global_elo.player_two,
                    &PLAYERS.to_string()
                );
                rusqlite_connection.update_player(
                    &game_elo.player_two,
                    &game_name
                );

                println!(
                    "P1: {} - Elo: {:?}, P2: {} - Elo: {:?}",
                    player_one_name,
                    game_elo.player_one.elo,
                    player_two_name,
                    game_elo.player_two.elo
                );
                // If this is the last match, this is grand finals. Therefore
                // whoever has the larger score won the tournament.
                println!("i: {}, num_pages: {}, count: {}, set: {}", i, num_pages, count, set_list.len());
                if i == (num_pages - 1) && count == set_list.len() {
                    println!("Enter");
                    if set.player_one_score > set.player_two_score {
                        rusqlite_connection.assign_winner(
                            player_one_global_id,
                            &PLAYERS.to_string()
                        ).expect("Assigning P1 as winner to players failed");
                        rusqlite_connection.assign_winner(
                            player_one_global_id,
                            &game_name
                        ).expect("Assigning P1 as winner to game failed");
                    } else {
                        rusqlite_connection.assign_winner(
                            player_two_global_id,
                            &PLAYERS.to_string()
                        ).expect("Assigning P2 as winner to players failed");
                        rusqlite_connection.assign_winner(
                            player_two_global_id,
                            &game_name
                        ).expect("Assigning P2 as winner to game failed");
                    }
                }
            }

            count += 1;
        }
    }

    // Update the rankings and increment the relevant counters.
    rusqlite_connection.update_ranking(&PLAYERS.to_string())
        .expect("Updating rankings for players failed");
    rusqlite_connection.update_ranking(&game_name)
        .expect("Updating ranking for game failed");
    rusqlite_connection.increment_count(&players, &PLAYERS.to_string())
        .expect("Incrementing game count for players failed");
    rusqlite_connection.increment_count(&players, &game_name)
        .expect("Incrementing game count for game failed");

    Ok(())
}