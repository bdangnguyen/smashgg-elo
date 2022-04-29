use rusqlite::{params, Connection, Error};
use smashgg_elo_rust::clean_string;
use std::collections::HashMap;

// Wrapper struct representing a connection to a sqlite database.
pub struct RusqliteConnection {
    conn: Connection,
}

// Struct that represents a row in the players table. This contains all of the
// data and statistics of a player's performance in a tournament over time.
#[derive(Debug)]
pub struct PlayersRow {
    pub global_id: i32,
    pub name: String,
    pub rank: i32,
    pub elo: f64,
    pub num_games: i32,
    pub wins: i32,
    pub losses: i32,
    pub win_loss_ratio: f64,
    pub num_tournaments: i32,
    pub tournament_wins: i32,
}

// Struct that represents a row in the sets table. This contains all of the
// details of a set that happened between two players in a tournament, and the
// changes to the elo that happened as a result of the set.
pub struct SetsRow {
    pub player_one_global_id: i32,
    pub player_one_name: String,
    pub player_one_elo: f64,
    pub player_one_score: i32,
    pub player_one_elo_delta: f64,
    pub player_two_global_id: i32,
    pub player_two_name: String,
    pub player_two_elo: f64,
    pub player_two_score: i32,
    pub player_two_elo_delta: f64,
    pub tournament_name: String,
    pub game_name: String,
    pub set_time: String,
}

impl Default for SetsRow {
    fn default() -> Self {
        SetsRow {
            player_one_global_id: -1,
            player_one_name: "Player 1".to_string(),
            player_one_elo: 0.0,
            player_one_score: 0,
            player_one_elo_delta: 0.0,
            player_two_global_id: -2,
            player_two_name: "Player 2".to_string(),
            player_two_elo: 0.0,
            player_two_score: 0,
            player_two_elo_delta: 0.0,
            tournament_name: "Default Tournament".to_string(),
            game_name: "Default Game".to_string(),
            set_time: "".to_string(),
        }
    }
}

// Wrapper struct for the ease of working with an iterator when updating elo
// and updating statistics such as tournament count.
#[derive(Debug)]
pub struct ItrStruct {
    pub itr_int: i32,
}

impl Default for RusqliteConnection {
    fn default() -> Self {
        // Initialize connection to the sqlite db.
        let conn =
            Connection::open("./database/smashhgg.db3")
            .expect("Connecting to database failed");

        // Initialize the player table if there is none.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                global_id               INTEGER NOT NULL PRIMARY KEY UNIQUE,
                name             TEXT NOT NULL,
                rank             INTEGER DEFAULT 0 NOT NULL,
                elo              REAL DEFAULT 1500.0 NOT NULL,
                num_games        INTEGER DEFAULT 0 NOT NULL,
                wins             INTEGER DEFAULT 0 NOT NULL,
                losses           INTEGER DEFAULT 0 NOT NULL,
                win_loss_ratio   REAL DEFAULT 0 NOT NULL,
                num_tournaments  INTEGER DEFAULT 0 NOT NULL,
                tournament_wins  INTEGER DEFAULT 0 NOT NULL
            )",
            [],
        )
        .expect("Creating player table failed");

        // Initialize the set history table if there is none.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sets (
                id                      INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                player_one_global_id    INTEGER NOT NULL,
                player_one_name         TEXT NOT NULL,
                player_one_elo          REAL NOT NULL,
                player_one_score        INTEGER NOT NULL,
                player_one_elo_delta    REAL NOT NULL,
                player_two_global_id    INTEGER NOT NULL,
                player_two_name         TEXT NOT NULL,
                player_two_elo          REAL NOT NULL,
                player_two_score        INTEGER NOT NULL,
                player_two_elo_delta    REAL NOT NULL,
                tournament_name         TEXT NOT NULL,
                game_name               TEST NOT NULL,
                set_time                TEXT NOT NULL
            )",
            [],
        )
        .expect("Creating sets table failed");
        println!("Connected to database at database/smashgg.db3");

        RusqliteConnection { conn }
    }
}

impl RusqliteConnection {
    pub fn new() -> Self {
        RusqliteConnection::default()
    }

    /// Given a table name, create a table in the sqlite database if it doesn't
    /// exist. This is primarily used to generate tables for different games.
    pub fn create_table(&self, table_name: &String) {
        // Initialize a game table if there is none.
        let table_stmt = format!("CREATE TABLE IF NOT EXISTS {} (
            global_id        INTEGER NOT NULL PRIMARY KEY UNIQUE,
            name             TEXT NOT NULL,
            rank             INTEGER DEFAULT 0 NOT NULL,
            elo              REAL DEFAULT 1500.0 NOT NULL,
            num_games        INTEGER DEFAULT 0 NOT NULL,
            wins             INTEGER DEFAULT 0 NOT NULL,
            losses           INTEGER DEFAULT 0 NOT NULL,
            win_loss_ratio   REAL DEFAULT 0 NOT NULL,
            num_tournaments  INTEGER DEFAULT 0 NOT NULL,
            tournament_wins  INTEGER DEFAULT 0 NOT NULL
        )", clean_string(table_name));
        self.conn
            .execute(table_stmt.as_str(), [])
            .expect("Creating a game table failed");
    }

    // Given a global id, and the player name, the function searches the
    // database for any existing record of the player participating in a
    // tournament. If no such record exists, it will create one.
    pub fn select_player(
        &self,
        global_id: i32,
        name: &String,
        table_name: &String,
    ) -> Result<PlayersRow, Error> {
        // If the player does not exist in the database, create a default
        // record for the player in the sqlite database.
        let insert_stmt = format!(
            "INSERT OR IGNORE INTO {} (global_id, name) VALUES (?1, ?2)",
            clean_string(table_name)
        );
        self.conn
            .execute(insert_stmt.as_str(), params![global_id, name])?;

        // Find the row in the player table that matches to the id. Once found
        // create a PlayerRow object to use.
        let select_stmt = format!(
            "SELECT * FROM {} WHERE global_id = ?1",
            clean_string(table_name)
        );
        let mut stmt = self.conn.prepare(select_stmt.as_str())?;
        let player_iter = stmt.query_map(params![global_id], |row| {
            Ok(PlayersRow {
                global_id,
                name: row.get(1)?,
                rank: row.get(2)?,
                elo: row.get(3)?,
                num_games: row.get(4)?,
                wins: row.get(5)?,
                losses: row.get(6)?,
                win_loss_ratio: row.get(7)?,
                num_tournaments: row.get(8)?,
                tournament_wins: row.get(9)?,
            })
        })?;

        player_iter
            .last()
            .expect("Getting a player from the database failed")
    }

    // Updates player information in the database after elo calculations have
    // been made.
    pub fn update_player(&self, player: &PlayersRow, table_name: &String) {
        let update_stmt = format!(
            "UPDATE {} SET
                elo = ?1,
                num_games = ?2, 
                wins = ?3,
                losses = ?4,
                win_loss_ratio = ?5
            WHERE global_id = ?6",
            clean_string(table_name)
        );
        self.conn
            .execute(
                update_stmt.as_str(),
                params![
                    player.elo,
                    player.num_games,
                    player.wins,
                    player.losses,
                    player.win_loss_ratio,
                    player.global_id
                ],
            )
            .expect("Updating player info failed");
    }

    // Records the result of the set and any information regarding changes in
    // elo into the database.
    pub fn insert_set(&self, match_info: SetsRow) {
        self.conn
            .execute(
                "INSERT INTO sets (player_one_global_id,
                    player_one_name,
                    player_one_elo,
                    player_one_score,
                    player_one_elo_delta,
                    player_two_global_id,
                    player_two_name,
                    player_two_elo,
                    player_two_score,
                    player_two_elo_delta,
                    tournament_name,
                    game_name,
                    set_time)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    match_info.player_one_global_id,
                    match_info.player_one_name,
                    match_info.player_one_elo,
                    match_info.player_one_score,
                    match_info.player_one_elo_delta,
                    match_info.player_two_global_id,
                    match_info.player_two_name,
                    match_info.player_two_elo,
                    match_info.player_two_score,
                    match_info.player_two_elo_delta,
                    match_info.tournament_name,
                    match_info.game_name,
                    match_info.set_time
                ],
            )
            .expect("Inserting match into database failed");
    }

    // Simply selects all of the players who have at least one completed set
    // and updates the elo rankings in the database.
    pub fn update_ranking(
        &self,
        table_name: &String
    ) -> Result<(), rusqlite::Error> {
        // Select all players in the database and order by elo.
        let mut count = 1;
        let rank_stmt = format!(
            "SELECT global_id FROM {} ORDER BY elo DESC",
            clean_string(table_name)
        );
        let mut stmt = self.conn.prepare(&rank_stmt)?;
        let rank_iter = stmt.query_map([], |row| {
            Ok(ItrStruct {
                itr_int: row.get(0)?,
            })
        })?;

        // Iterate through each player in the database and update rankings.
        let ranking_stmt = format!(
            "UPDATE {} SET rank = ?1 WHERE global_id =?2",
            clean_string(table_name)
        );
        for player_id in rank_iter {
            let mut stmt = self.conn.prepare(&ranking_stmt.to_string())?;

            match player_id {
                Ok(player) => {
                    stmt.execute(params![count, player.itr_int])?;
                    count += 1;
                }
                Err(err) => println!("Error updating rankings: {}", err),
            }
        }

        Ok(())
    }

    pub fn increment_count(
        &self,
        player_map: &HashMap<i32, (String, i32)>,
        table_name: &String,
    ) -> Result<(), rusqlite::Error> {
        // For each player in the tournament, grab the number of tournaments
        // that they participated in.
        for player in player_map.values() {
            let incr_stmt = format!(
                "SELECT num_tournaments FROM {} WHERE global_id = {}",
                clean_string(table_name),
                player.1
            );
            let mut stmt = self.conn.prepare(&incr_stmt.to_string())?;
            let mut incr_iter = stmt
                .query_map([], |row| {
                    Ok(ItrStruct {
                        itr_int: row.get(0)?,
                    })
                })?
                .peekable();

            // If the player exists in the sqlite database, increment the
            // number of tournaments that they have.
            if incr_iter.peek().is_some() {
                match incr_iter
                    .last()
                    .expect("Iterator error: No data for num_tournaments")
                {
                    Ok(num_tour) => {
                        let update_stmt = format!(
                            "UPDATE {} SET num_tournaments = ?1 WHERE global_id = ?2",
                            clean_string(table_name)
                        );
                        stmt = self.conn.prepare(&update_stmt.to_string())?;
                        stmt.execute(params![num_tour.itr_int + 1, player.1])?;
                    }
                    Err(err) => {
                        panic!("Error updating number of tournaments: {}", err)
                    }
                }
            }
        }

        Ok(())
    }

    /// Function that takes the global_id of the winner of the tournament and
    /// the game that the tournament was hosted in and updates the sqlite
    /// database to reflect the win.
    pub fn assign_winner(
        &self,
        global_id: i32,
        table_name: &String,
    ) -> Result<(), rusqlite::Error> {
        // Select the number of wins that the tournament winner has
        let win_stmt = format!(
            "SELECT tournament_wins FROM {} WHERE global_id = {}",
            clean_string(table_name),
            global_id
        );
        let mut stmt = self.conn.prepare(&win_stmt)?;
        let win_iter = stmt.query_map([], |row| {
            Ok(ItrStruct {
                itr_int: row.get(0)?,
            })
        })?;

        // Increment the number of wins that a tournament winner has
        let winner_stmt = format!(
            "UPDATE {} SET tournament_wins = ?1 WHERE global_id = ?2",
            clean_string(table_name)
        );
        let mut stmt = self.conn.prepare(&winner_stmt)?;
        match win_iter
            .last()
            .expect("Iterator error: No data found for tournament_wins")
        {
            Ok(winner_struct) => {
                stmt.execute(params![winner_struct.itr_int + 1, global_id])?;
            }
            Err(err) => panic!("Matching error: {}", err),
        }

        Ok(())
    }
}
