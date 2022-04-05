use rusqlite::Connection;

pub struct RusqliteConnection {
    conn: Connection
}

impl Default for RusqliteConnection {
    fn default() -> Self {
        let conn = match Connection::open("./database/smashhgg.db3") {
            Ok(conn) => conn,
            Err(err) => panic!("Error in connecting to sqlite db: {}", err)
        };

        RusqliteConnection { 
            conn
        }
    }
}

impl RusqliteConnection {
    pub fn new() -> Self {
        RusqliteConnection::default()
    }

    pub fn get_player(self) {
        match self.conn.execute(
            "create table if not exists players (
                global_id               INTEGER NOT NULL PRIMARY KEY,
                player_name             TEXT NOT NULL,
                player_rank             INTEGER NOT NULL,
                player_elo              INTEGER NOT NULL,
                player_num_games        INTEGER NOT NULL,
                player_wins             INTEGER NOT NULL,
                player_losses           INTEGER NOT NULL,
                player_win_loss_ratio   REAL NOT NULL,
                player_num_tournaments  INTEGER NOT NULL,
                player_tournament_wins  INTEGER NOT NULL
            )",
            []
        ) {
            Ok(notification) => println!("{} rows were created", notification),
            Err(err) => println!("Getting player stats failed: {}", err),
        }
    }
}