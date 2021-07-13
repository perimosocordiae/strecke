use crate::game::GameManager;
use crate::{board, tiles};
use rusqlite::{Connection, Result};

pub struct AppState {
    pub gm: GameManager,
    conn: Connection,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
             id INTEGER PRIMARY KEY,
             username TEXT NOT NULL UNIQUE,
             hashed_password TEXT,
             num_games INTEGER,
             last_game TIMESTAMP
         )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS games (
             id INTEGER PRIMARY KEY,
             start_time TIMESTAMP,
             end_time TIMESTAMP,
             board_state JSON,
             player_ids JSON
         )",
            [],
        )?;
        let mut rng = rand::thread_rng();
        let mut gm = GameManager::new(&mut rng);

        // Start demo code to exercise the game logic--------------------
        gm.register_player(
            "CJ",
            board::Position {
                row: 2,
                col: 6,
                port: tiles::Port::G,
                alive: true,
            },
        )
        .unwrap();
        gm.register_player(
            "Bob",
            board::Position {
                row: -1,
                col: 3,
                port: tiles::Port::E,
                alive: true,
            },
        )
        .unwrap();
        // End demo code ------------------------------------------------

        Ok(Self { gm, conn })
    }
}
