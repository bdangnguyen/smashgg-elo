// TODO: Check schema interaction for match table. cumulative id? different set time format? Is provisional needed?
use rusqlite::Connection;
use rusqlite::params;

pub struct RusqliteConnection {
    conn: Connection
}

pub struct PlayersRow {
    pub global_id: i32,
    pub player_name: String,
    pub player_rank: i32,
    pub player_elo: f64,
    pub player_num_games: i32,
    pub player_wins: i32,
    pub player_losses: i32,
    pub player_win_loss_ratio: f64,
    pub player_num_tournaments: i32,
    pub player_tournament_wins: i32,
}

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
    pub set_time: String
}

impl Default for RusqliteConnection {
    fn default() -> Self {
        // Initialize connection to the sqlite db.
        let conn = Connection::open("./database/smashhgg.db3").expect("Connecting to database failed");

        // Initialize the player table if there is none.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
                global_id               INTEGER NOT NULL PRIMARY KEY UNIQUE,
                player_name             TEXT NOT NULL,
                player_rank             INTEGER DEFAULT 0 NOT NULL,
                player_elo              REAL DEFAULT 1500.0 NOT NULL,
                player_num_games        INTEGER DEFAULT 0 NOT NULL,
                player_wins             INTEGER DEFAULT 0 NOT NULL,
                player_losses           INTEGER DEFAULT 0 NOT NULL,
                player_win_loss_ratio   REAL DEFAULT 0 NOT NULL,
                player_num_tournaments  INTEGER DEFAULT 0 NOT NULL,
                player_tournament_wins  INTEGER DEFAULT 0 NOT NULL
            )",
            []
        ).expect("Creating player table failed");

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
                set_time                TEXT NOT NULL
            )",
            []
        ).expect("Creating sets table failed");

        RusqliteConnection { 
            conn
        }
    }
}

impl RusqliteConnection {
    pub fn new() -> Self {
        RusqliteConnection::default()
    }

    pub fn select_player(&self, global_id: i32, player_name: &String) -> Result<PlayersRow, rusqlite::Error> {
        // If the player does not exist in the database, create a default
        // record for the player in the sqlite database.
        self.conn.execute(
            "INSERT OR IGNORE into players (global_id, player_name) VALUES (?1,?2)",
            params![global_id, player_name]
        )?;
        
        // Find the row in the player table that matches to the id. Once found
        // create a PlayerRow object to use.
        let mut stmt = self.conn.prepare("SELECT * FROM players WHERE global_id=?1")?;
        let player_iter = stmt.query_map(params![global_id], |row| {
        Ok(
            PlayersRow {
                global_id,
                player_name: row.get(1)?,
                player_rank: row.get(2)?,
                player_elo: row.get(3)?,
                player_num_games: row.get(4)?,
                player_wins: row.get(5)?,
                player_losses: row.get(6)?,
                player_win_loss_ratio: row.get(7)?,
                player_num_tournaments: row.get(8)?,
                player_tournament_wins: row.get(9)?,
            })
        })?;

        player_iter.last().expect("Getting a player from the database failed")
    }

    pub fn update_player(&self, player: PlayersRow) {
        println!("Player ratio: {}", player.player_win_loss_ratio);
        self.conn.execute(
            "UPDATE players SET 
                    player_elo = ?1, 
                    player_num_games = ?2, 
                    player_wins = ?3,
                    player_losses = ?4,
                    player_win_loss_ratio = ?5
                WHERE global_id = ?6",
            params![player.player_elo,
            player.player_num_games,
            player.player_wins,
            player.player_losses,
            player.player_win_loss_ratio,
            player.global_id]
        ).expect("Updating player info failed");
    }           

    pub fn insert_match(&self, match_info: SetsRow) {
        self.conn.execute(
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
                    set_time)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![match_info.player_one_global_id,
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
            match_info.set_time]
        ).expect("Inserting match into database failed");
    }
}